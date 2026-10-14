#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]
use chrono::{SecondsFormat, Utc};
use codex_quota_windows_core::ble::native::BleWorker;
use codex_quota_windows_core::quota::{QuotaCollector, load_cached_snapshot, save_cached_snapshot};
use codex_quota_windows_core::*;
use std::{
    mem::size_of,
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{Arc, Mutex, OnceLock, RwLock},
    time::Duration,
};
use tokio::{runtime::Runtime, task::JoinHandle};
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::GetModuleHandleW, Registry::*, Threading::CreateMutexW},
    UI::{Shell::*, WindowsAndMessaging::*},
};
const APP_NAME: &str = "Codex额度";
const TRAY_ICON_ID: u32 = 1;
const TRAY_MESSAGE: u32 = WM_APP + 1;
const MENU_REFRESH: u32 = 1001;
const MENU_BLE: u32 = 1002;
const MENU_STARTUP: u32 = 1003;
const MENU_EXIT: u32 = 1004;
const MENU_HELP: u32 = 1005;
const TRAY_ICON_ICO: &[u8] = include_bytes!("../../assets/tray-icon.ico");
static APP: OnceLock<App> = OnceLock::new();
struct App {
    ble: BleWorker,
    setting: PathBuf,
    quota: Arc<RwLock<QuotaSnapshot>>,
    refresh: tokio::sync::mpsc::Sender<()>,
    task: Mutex<Option<JoinHandle<()>>>,
    runtime: Runtime,
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--migrate-legacy-setup") {
        if migration::cleanup().is_err() {
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|a| a == "--smoke-test") {
        match unsafe { load_tray_icon() } {
            Ok(icon) => unsafe {
                DestroyIcon(icon);
            },
            Err(_) => std::process::exit(1),
        }
        return;
    }
    if args.iter().any(|a| a == "--ble-probe") {
        println!("{}", ble::native::probe());
        return;
    }
    // Obsolete Hook invocations from older installations exit without reading stdin.
    if args.iter().any(|a| {
        matches!(
            a.as_str(),
            "--hook-event" | "--install-hooks" | "--remove-hooks"
        )
    }) {
        return;
    }
    if let Err(error) = run(args.iter().any(|a| a == "--show-onboarding")) {
        show_message(APP_NAME, &error, MB_ICONERROR);
    }
}
fn run(onboarding: bool) -> Result<(), String> {
    unsafe { SetLastError(0) };
    let mutex = unsafe {
        CreateMutexW(
            null(),
            0,
            wide("Local\\CodexQuotaWindowsNative040").as_ptr(),
        )
    };
    if mutex.is_null() {
        return Err("无法创建单实例锁".into());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(mutex) };
        return Ok(());
    }
    let directory = data_directory()?;
    std::fs::create_dir_all(&directory).map_err(|_| "无法创建额度缓存目录")?;
    let path = directory.join("last-snapshot-v1.json");
    let collector = QuotaCollector::discover().ok();
    let initial = collector
        .as_ref()
        .and_then(|c| c.collect(Utc::now()).ok())
        .or_else(|| load_cached_snapshot(&path, Utc::now()))
        .unwrap_or_else(unavailable_quota);
    let quota = Arc::new(RwLock::new(initial));
    let setting = directory.join("huangshan-ble-enabled-v1");
    let enabled = std::env::args().any(|a| a == "--huangshan-ble")
        || std::fs::read(&setting).map_or(true, |v| v != b"0");
    let ble = BleWorker::start(quota.clone(), enabled).map_err(|_| "无法启动黄山派蓝牙")?;
    let runtime = Runtime::new().map_err(|_| "无法启动额度采集")?;
    let (refresh, mut requests) = tokio::sync::mpsc::channel(1);
    let state = quota.clone();
    let task = runtime.spawn(async move {
        let mut local = tokio::time::interval(Duration::from_secs(5));
        let mut upstream = tokio::time::interval(Duration::from_secs(45));
        let mut collector = collector;
        loop {
            let confirm = tokio::select! {
                _=local.tick()=>false,
                _=upstream.tick()=>true,
                request=requests.recv()=>{if request.is_none(){break;}true}
            };
            if collector.is_none() {
                collector = QuotaCollector::discover().ok();
            }
            let Some(current) = collector.clone() else {
                continue;
            };
            let collected = tokio::task::spawn_blocking(move || {
                if confirm {
                    current.refresh_upstream_freshness(Utc::now());
                }
                current.collect(Utc::now())
            })
            .await;
            match collected {
                Ok(Ok(snapshot)) => {
                    if snapshot.link.codex == CodexLinkStatus::Ok {
                        let _ = save_cached_snapshot(&path, &snapshot);
                    }
                    if let Ok(mut latest) = state.write() {
                        *latest = snapshot;
                    }
                }
                _ => {
                    if let Ok(mut latest) = state.write() {
                        latest.link.codex = CodexLinkStatus::Stale;
                        latest.source_status = QuotaSourceStatus::Partial;
                    }
                }
            }
        }
    });
    APP.set(App {
        ble,
        setting,
        quota,
        refresh,
        task: Mutex::new(Some(task)),
        runtime,
    })
    .map_err(|_| "重复初始化")?;
    let _ = ensure_default_startup_enabled();
    unsafe {
        let instance = GetModuleHandleW(null());
        let class = wide("CodexQuotaTrayWindow");
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(tray_window_proc),
            hInstance: instance,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            lpszClassName: class.as_ptr(),
            ..Default::default()
        };
        if RegisterClassExW(&wc) == 0 {
            return Err("无法注册托盘窗口".into());
        }
        let window = create_tray_window()?;
        let icon = add_tray_icon(window)?;
        if std::env::args().any(|a| a == "--menu-probe") {
            let menu = create_tray_menu();
            for index in 0..GetMenuItemCount(menu) {
                let mut text = [0u16; 256];
                let count = GetMenuStringW(
                    menu,
                    index as u32,
                    text.as_mut_ptr(),
                    text.len() as i32,
                    MF_BYPOSITION,
                );
                if count > 0 {
                    println!("{}", String::from_utf16_lossy(&text[..count as usize]));
                }
            }
            DestroyMenu(menu);
        } else {
            if onboarding {
                show_help();
            }
            let mut message = MSG::default();
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        remove_tray_icon(window);
        DestroyIcon(icon);
    }
    if let Some(app) = APP.get() {
        app.ble.stop();
        if let Some(task) = app.task.lock().ok().and_then(|mut t| t.take()) {
            task.abort();
            let _ = app.runtime.block_on(task);
        }
    }
    unsafe { CloseHandle(mutex) };
    Ok(())
}
fn show_help() {
    show_message(
        "连接黄山派",
        "在 Windows 蓝牙设置中添加 CodexQuota-HSPI，输入黄山派显示的6位配对码。\n\n配对后托盘中的「黄山派蓝牙同步」保持开启。\n\nKEY2切换布局，离线3分钟息屏；触摸或KEY2唤醒。",
        MB_ICONINFORMATION,
    );
}
unsafe fn create_tray_menu() -> HMENU {
    let menu = CreatePopupMenu();
    if let Some(app) = APP.get() {
        append_menu(menu, MF_STRING | MF_DISABLED, 0, app.ble.status().label());
        if let Ok(quota) = app.quota.read() {
            let fresh = match quota.upstream_freshness.usage.status {
                UpstreamFreshnessStatus::Current => "已同步",
                UpstreamFreshnessStatus::Cached => "缓存",
                _ => "等待同步",
            };
            append_menu(menu, MF_STRING | MF_DISABLED, 0, fresh);
            for label in tray_quota_labels(&quota) {
                append_menu(menu, MF_STRING | MF_DISABLED, 0, &label);
            }
        }
        append_menu(
            menu,
            MF_STRING | if app.ble.enabled() { MF_CHECKED } else { 0 },
            MENU_BLE as usize,
            "黄山派蓝牙同步",
        );
    }
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_menu(menu, MF_STRING, MENU_REFRESH as usize, "刷新额度");
    append_menu(menu, MF_STRING, MENU_HELP as usize, "连接黄山派…");
    append_menu(
        menu,
        MF_STRING | if startup_enabled() { MF_CHECKED } else { 0 },
        MENU_STARTUP as usize,
        "开机启动",
    );
    append_menu(menu, MF_STRING, MENU_EXIT as usize, "退出");
    menu
}
unsafe fn show_tray_menu(window: HWND) {
    let menu = create_tray_menu();
    let mut point = POINT::default();
    GetCursorPos(&mut point);
    SetForegroundWindow(window);
    let command = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON,
        point.x,
        point.y,
        0,
        window,
        null(),
    );
    PostMessageW(window, WM_NULL, 0, 0);
    DestroyMenu(menu);
    match command as u32 {
        MENU_REFRESH => {
            if let Some(app) = APP.get() {
                let _ = app.refresh.try_send(());
            }
        }
        MENU_BLE => {
            if let Some(app) = APP.get() {
                let enabled = !app.ble.enabled();
                if std::fs::write(&app.setting, if enabled { b"1" } else { b"0" }).is_ok() {
                    app.ble.set_enabled(enabled);
                } else {
                    show_message(APP_NAME, "无法保存蓝牙设置", MB_ICONERROR);
                }
            }
        }
        MENU_HELP => show_help(),
        MENU_STARTUP => {
            if let Err(e) = set_startup_enabled(!startup_enabled()) {
                show_message(APP_NAME, &e, MB_ICONERROR);
            }
        }
        MENU_EXIT => {
            DestroyWindow(window);
        }
        _ => {}
    }
}
fn unavailable_quota() -> QuotaSnapshot {
    QuotaSnapshot {
        window_presence: QuotaWindowPresence::default(),
        protocol_version: 1,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        source_status: QuotaSourceStatus::Unavailable,
        limits_collected_at: None,
        windows: vec![],
        reset_inventory: ResetInventorySnapshot {
            status: ResetInventoryStatus::Unavailable,
            available_count: None,
            cached_at: None,
            items: vec![],
        },
        link: QuotaLink {
            computer: ComputerLinkStatus::Online,
            codex: CodexLinkStatus::Unavailable,
        },
        upstream_freshness: UpstreamFreshness::default(),
    }
}

unsafe fn create_tray_window() -> Result<HWND, String> {
    let class_name = wide("CodexQuotaTrayWindow");
    let title = wide(APP_NAME);
    let window = CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        null_mut(),
        null_mut(),
        GetModuleHandleW(null()),
        null(),
    );
    if window.is_null() {
        Err("无法创建托盘窗口".to_string())
    } else {
        Ok(window)
    }
}

unsafe fn add_tray_icon(window: HWND) -> Result<HICON, String> {
    let mut data = tray_icon_data(window);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = TRAY_MESSAGE;
    let icon = load_tray_icon()?;
    data.hIcon = icon;
    copy_wide(&mut data.szTip, APP_NAME);
    if Shell_NotifyIconW(NIM_ADD, &data) == 0 {
        DestroyIcon(icon);
        Err("无法创建通知区域图标".to_string())
    } else {
        Ok(icon)
    }
}

unsafe fn load_tray_icon() -> Result<HICON, String> {
    if TRAY_ICON_ICO.len() < 22 || TRAY_ICON_ICO[0..6] != [0, 0, 1, 0, 1, 0] {
        return Err("托盘图标资源无效".to_string());
    }
    let image_size = u32::from_le_bytes(
        TRAY_ICON_ICO[14..18]
            .try_into()
            .map_err(|_| "托盘图标资源无效".to_string())?,
    );
    let image_offset = u32::from_le_bytes(
        TRAY_ICON_ICO[18..22]
            .try_into()
            .map_err(|_| "托盘图标资源无效".to_string())?,
    );
    let image_end = image_offset
        .checked_add(image_size)
        .ok_or_else(|| "托盘图标资源无效".to_string())? as usize;
    if image_offset < 22 || image_end > TRAY_ICON_ICO.len() {
        return Err("托盘图标资源无效".to_string());
    }
    let icon = CreateIconFromResourceEx(
        TRAY_ICON_ICO[image_offset as usize..image_end].as_ptr(),
        image_size,
        TRUE,
        0x0003_0000,
        32,
        32,
        LR_DEFAULTSIZE,
    );
    if icon.is_null() {
        Err("无法加载托盘图标".to_string())
    } else {
        Ok(icon)
    }
}

unsafe fn remove_tray_icon(window: HWND) {
    let data = tray_icon_data(window);
    Shell_NotifyIconW(NIM_DELETE, &data);
}

fn tray_icon_data(window: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: window,
        uID: TRAY_ICON_ID,
        ..Default::default()
    }
}

unsafe extern "system" fn tray_window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        TRAY_MESSAGE => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP || event == WM_CONTEXTMENU || event == WM_LBUTTONDBLCLK {
                show_tray_menu(window);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

unsafe fn append_menu(menu: HMENU, flags: MENU_ITEM_FLAGS, id: usize, label: &str) {
    let label = wide(label);
    AppendMenuW(menu, flags, id, label.as_ptr());
}

fn tray_quota_labels(quota: &QuotaSnapshot) -> [String; 2] {
    [
        tray_quota_label(quota, "five_hour", 300, "5小时额度"),
        tray_quota_label(quota, "weekly", 10_080, "周额度"),
    ]
}

fn tray_quota_label(
    quota: &QuotaSnapshot,
    window_name: &str,
    window_minutes: u32,
    label: &str,
) -> String {
    let remaining = quota
        .windows
        .iter()
        .find(|window| window.name == window_name && window.window_minutes == window_minutes)
        .filter(|window| {
            window.status == codex_quota_windows_core::QuotaWindowStatus::Current
                && chrono::DateTime::parse_from_rfc3339(&window.resets_at)
                    .is_ok_and(|resets_at| resets_at.with_timezone(&Utc) > Utc::now())
        })
        .and_then(|window| window.remaining_percent);
    match remaining {
        Some(percent) => format!("{label} {percent}%"),
        None => format!("{label} --"),
    }
}

fn startup_enabled() -> bool {
    unsafe {
        let mut key = HKEY::default();
        let path = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }
        let name = wide("CodexQuota");
        let result = RegQueryValueExW(
            key,
            name.as_ptr(),
            null(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
        RegCloseKey(key);
        result == ERROR_SUCCESS
    }
}

fn set_startup_enabled(enabled: bool) -> Result<(), String> {
    unsafe {
        let path = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let mut key = HKEY::default();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            null(),
            &mut key,
            null_mut(),
        ) != ERROR_SUCCESS
        {
            return Err("无法打开当前用户启动项".to_string());
        }
        let name = wide("CodexQuota");
        let result = if enabled {
            let executable = std::env::current_exe().map_err(|error| error.to_string())?;
            let command = wide(&format!("\"{}\"", executable.display()));
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                REG_SZ,
                command.as_ptr().cast(),
                (command.len() * size_of::<u16>()) as u32,
            )
        } else {
            RegDeleteValueW(key, name.as_ptr())
        };
        RegCloseKey(key);
        if result == ERROR_SUCCESS || (!enabled && result == ERROR_FILE_NOT_FOUND) {
            mark_startup_preference_configured()
        } else {
            Err(format!("Windows 注册表返回错误 {result}"))
        }
    }
}

fn startup_preference_configured() -> bool {
    unsafe {
        let mut key = HKEY::default();
        let path = wide("Software\\CodexQuota");
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }
        let name = wide("StartupPreferenceConfigured");
        let result = RegQueryValueExW(
            key,
            name.as_ptr(),
            null(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
        RegCloseKey(key);
        result == ERROR_SUCCESS
    }
}

fn mark_startup_preference_configured() -> Result<(), String> {
    unsafe {
        let path = wide("Software\\CodexQuota");
        let mut key = HKEY::default();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            null(),
            &mut key,
            null_mut(),
        ) != ERROR_SUCCESS
        {
            return Err("无法保存自动启动偏好".to_string());
        }
        let name = wide("StartupPreferenceConfigured");
        let configured = 1_u32;
        let result = RegSetValueExW(
            key,
            name.as_ptr(),
            0,
            REG_DWORD,
            (&configured as *const u32).cast(),
            size_of::<u32>() as u32,
        );
        RegCloseKey(key);
        if result == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("Windows 注册表返回错误 {result}"))
        }
    }
}

fn ensure_default_startup_enabled() -> Result<(), String> {
    if startup_preference_configured() {
        Ok(())
    } else {
        set_startup_enabled(true)
    }
}

fn data_directory() -> Result<PathBuf, String> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("CodexQuota").join("0.4.0"))
        .ok_or_else(|| "无法定位 LOCALAPPDATA".to_string())
}

fn show_message(title: &str, body: &str, flags: MESSAGEBOX_STYLE) {
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(body).as_ptr(),
            wide(title).as_ptr(),
            MB_OK | flags,
        );
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn copy_wide<const N: usize>(target: &mut [u16; N], value: &str) {
    let encoded = wide(value);
    let count = encoded.len().min(N);
    target[..count].copy_from_slice(&encoded[..count]);
}
