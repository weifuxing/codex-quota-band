//! Windows GATT client. Only explicitly paired CodexQuota-HSPI devices are used.
use super::{DEVICE_NAME, QUOTA_UUID, SERVICE_UUID, encode_quota};
use crate::QuotaSnapshot;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristic, GattCommunicationStatus, GattDeviceService, GattProtectionLevel,
    GattSession, GattWriteOption,
};
use windows::Devices::Bluetooth::{
    BluetoothAdapter, BluetoothCacheMode, BluetoothConnectionStatus, BluetoothLEDevice,
};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Storage::Streams::DataWriter;
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
use windows::core::GUID;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BleStatus {
    Disabled,
    Connecting,
    Connected,
    NotPaired,
    Ambiguous,
    Reconnecting,
    Unavailable,
}
impl BleStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "黄山派 未启用",
            Self::Connecting => "黄山派 连接中",
            Self::Connected => "黄山派 已连接",
            Self::NotPaired => "黄山派 等待 Windows 配对",
            Self::Ambiguous => "黄山派 存在多个配对设备",
            Self::Reconnecting => "黄山派 正在重连",
            Self::Unavailable => "黄山派 蓝牙不可用",
        }
    }
}
struct Control {
    enabled: AtomicBool,
    stopped: AtomicBool,
    status: AtomicU8,
}
pub struct BleWorker {
    control: Arc<Control>,
    thread: JoinHandle<()>,
}
impl BleWorker {
    pub fn start(quota: Arc<RwLock<QuotaSnapshot>>, enabled: bool) -> std::io::Result<Self> {
        let control = Arc::new(Control {
            enabled: AtomicBool::new(enabled),
            stopped: AtomicBool::new(false),
            status: AtomicU8::new(BleStatus::Disabled as u8),
        });
        let worker_control = control.clone();
        let thread = thread::Builder::new()
            .name("huangshan-ble".into())
            .spawn(move || {
                let initialized = unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.is_ok();
                if initialized {
                    run(quota, worker_control);
                    unsafe { RoUninitialize() };
                } else {
                    worker_control
                        .status
                        .store(BleStatus::Unavailable as u8, Ordering::Release);
                }
            })?;
        Ok(Self { control, thread })
    }
    pub fn enabled(&self) -> bool {
        self.control.enabled.load(Ordering::Acquire)
    }
    pub fn set_enabled(&self, enabled: bool) {
        self.control.enabled.store(enabled, Ordering::Release);
        self.thread.thread().unpark();
    }
    pub fn status(&self) -> BleStatus {
        match self.control.status.load(Ordering::Acquire) {
            0 => BleStatus::Disabled,
            1 => BleStatus::Connecting,
            2 => BleStatus::Connected,
            3 => BleStatus::NotPaired,
            4 => BleStatus::Ambiguous,
            5 => BleStatus::Reconnecting,
            _ => BleStatus::Unavailable,
        }
    }
    pub fn stop(&self) {
        self.control.stopped.store(true, Ordering::Release);
        self.thread.thread().unpark();
    }
}
impl Drop for BleWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
struct Connection {
    device: BluetoothLEDevice,
    service: GattDeviceService,
    session: GattSession,
    quota: GattCharacteristic,
}
impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.service.Close();
        let _ = self.device.Close();
    }
}
impl Connection {
    fn send(&self, packet: &[u8]) -> windows::core::Result<bool> {
        let writer = DataWriter::new()?;
        writer.WriteBytes(packet)?;
        let result = self
            .quota
            .WriteValueWithResultAndOptionAsync(
                &writer.DetachBuffer()?,
                GattWriteOption::WriteWithResponse,
            )?
            .join()?;
        Ok(result.Status()? == GattCommunicationStatus::Success)
    }
}
fn connect() -> Result<Connection, BleStatus> {
    let inner = || -> windows::core::Result<Result<Connection, BleStatus>> {
        let adapter = BluetoothAdapter::GetDefaultAsync()?.join()?;
        if !adapter.IsLowEnergySupported()? || !adapter.IsCentralRoleSupported()? {
            return Ok(Err(BleStatus::Unavailable));
        }
        let selector = BluetoothLEDevice::GetDeviceSelectorFromPairingState(true)?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.join()?;
        let mut candidate = None;
        for i in 0..devices.Size()? {
            let d = devices.GetAt(i)?;
            if d.Name()?.to_string() == DEVICE_NAME {
                if candidate.is_some() {
                    return Ok(Err(BleStatus::Ambiguous));
                }
                candidate = Some(d);
            }
        }
        let Some(info) = candidate else {
            return Ok(Err(BleStatus::NotPaired));
        };
        let device = BluetoothLEDevice::FromIdAsync(&info.Id()?)?.join()?;
        let result = device
            .GetGattServicesForUuidWithCacheModeAsync(
                GUID::from_u128(SERVICE_UUID),
                BluetoothCacheMode::Uncached,
            )?
            .join()?;
        if result.Status()? != GattCommunicationStatus::Success {
            return Ok(Err(BleStatus::Reconnecting));
        }
        let services = result.Services()?;
        if services.Size()? != 1 {
            return Ok(Err(BleStatus::Reconnecting));
        }
        let service = services.GetAt(0)?;
        let result = service
            .GetCharacteristicsForUuidWithCacheModeAsync(
                GUID::from_u128(QUOTA_UUID),
                BluetoothCacheMode::Uncached,
            )?
            .join()?;
        if result.Status()? != GattCommunicationStatus::Success {
            return Ok(Err(BleStatus::Reconnecting));
        }
        let chars = result.Characteristics()?;
        if chars.Size()? != 1 {
            return Ok(Err(BleStatus::Reconnecting));
        }
        let quota = chars.GetAt(0)?;
        quota.SetProtectionLevel(GattProtectionLevel::EncryptionAndAuthenticationRequired)?;
        let session = GattSession::FromDeviceIdAsync(&device.BluetoothDeviceId()?)?.join()?;
        session.SetMaintainConnection(true)?;
        Ok(Ok(Connection {
            device,
            service,
            session,
            quota,
        }))
    };
    inner().map_err(|_| BleStatus::Unavailable)?
}
fn run(quota: Arc<RwLock<QuotaSnapshot>>, control: Arc<Control>) {
    let mut connection: Option<Connection> = None;
    let mut due = Instant::now();
    let mut retry = 2u64;
    loop {
        if control.stopped.load(Ordering::Acquire) {
            return;
        }
        if !control.enabled.load(Ordering::Acquire) {
            connection = None;
            retry = 2;
            due = Instant::now();
            control
                .status
                .store(BleStatus::Disabled as u8, Ordering::Release);
            thread::park_timeout(Duration::from_secs(1));
            continue;
        }
        if Instant::now() < due {
            thread::park_timeout(Duration::from_secs(1));
            continue;
        }
        if connection.is_none() {
            control
                .status
                .store(BleStatus::Connecting as u8, Ordering::Release);
            match connect() {
                Ok(c) => connection = Some(c),
                Err(status) => {
                    control.status.store(status as u8, Ordering::Release);
                    due = Instant::now() + Duration::from_secs(retry);
                    retry = (retry * 2).min(30);
                    continue;
                }
            }
        }
        // Recheck the switch after potentially slow Windows device discovery.
        if !control.enabled.load(Ordering::Acquire) || control.stopped.load(Ordering::Acquire) {
            connection = None;
            continue;
        }
        let packet = quota.read().ok().and_then(|q| encode_quota(&q, Utc::now()));
        let ok = packet.is_some_and(|p| {
            connection
                .as_ref()
                .is_some_and(|c| c.send(&p).unwrap_or(false))
        });
        if ok {
            control
                .status
                .store(BleStatus::Connected as u8, Ordering::Release);
            retry = 2;
            due = Instant::now() + Duration::from_secs(10);
        } else {
            connection = None;
            control
                .status
                .store(BleStatus::Reconnecting as u8, Ordering::Release);
            due = Instant::now() + Duration::from_secs(retry);
            retry = (retry * 2).min(30);
        }
    }
}
/// Read-only diagnostic: enumerate paired boards and perform service discovery.
pub fn probe() -> &'static str {
    if unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.is_err() {
        return "蓝牙初始化失败";
    }
    let result = match connect() {
        Ok(c) => {
            let _ = c
                .device
                .ConnectionStatus()
                .map(|s| s == BluetoothConnectionStatus::Connected);
            "黄山派 GATT 服务可用"
        }
        Err(status) => status.label(),
    };
    unsafe { RoUninitialize() };
    result
}
