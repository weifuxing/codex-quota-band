//! Fixed-size, privacy-bounded Huangshan BLE quota summary.
use crate::{
    ComputerLinkStatus, QuotaSnapshot, QuotaSourceStatus, QuotaWindowStatus,
    UpstreamFreshnessStatus,
};
use chrono::{DateTime, Utc};
pub const PACKET_LEN: usize = 20;
pub const SERVICE_UUID: u128 = 0xf984c790_37ca_4d8b_a140_58d6a1b25f10;
pub const QUOTA_UUID: u128 = 0xf984c791_37ca_4d8b_a140_58d6a1b25f10;
pub const DEVICE_NAME: &str = "CodexQuota-HSPI";

fn unix(value: &str) -> Option<u32> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|d| u32::try_from(d.timestamp()).ok())
        .filter(|v| *v > 0)
}
/// Version, freshness, two percentages, then four LE u32 Unix timestamps.
/// Unknown percentages are 255; unknown timestamps are zero. No source labels,
/// reset card identities, tasks or credentials enter this transport.
pub fn encode_quota(snapshot: &QuotaSnapshot, now: DateTime<Utc>) -> Option<[u8; PACKET_LEN]> {
    let sent = u32::try_from(now.timestamp()).ok().filter(|t| *t > 0)?;
    let confirmed = snapshot
        .upstream_freshness
        .usage
        .last_success_at
        .as_deref()
        .and_then(unix)
        .filter(|t| *t <= sent)
        .unwrap_or(0);
    let mut packet = [0; PACKET_LEN];
    packet[0] = 2;
    packet[2] = 255;
    packet[3] = 255;
    let paused = snapshot.source_status == QuotaSourceStatus::Paused
        || snapshot.link.computer == ComputerLinkStatus::Paused;
    for (index, name, minutes) in [(0, "five_hour", 300), (1, "weekly", 10080)] {
        if paused {
            continue;
        }
        if let Some(w) = snapshot.windows.iter().find(|w| {
            w.name == name && w.window_minutes == minutes && w.status == QuotaWindowStatus::Current
        }) {
            if let Some(reset) = unix(&w.resets_at).filter(|t| *t > sent) {
                packet[12 + index * 4..16 + index * 4].copy_from_slice(&reset.to_le_bytes());
                packet[2 + index] = w.remaining_percent.filter(|p| *p <= 100).unwrap_or(255);
            }
        }
    }
    if snapshot.window_presence.five_hour == Some(false) {
        packet[2] = 254;
    }
    if snapshot.window_presence.weekly == Some(false) {
        packet[3] = 254;
    }
    packet[1] = if packet[2] > 100 && packet[3] > 100 {
        0
    } else if confirmed > 0
        && sent - confirmed <= 120
        && snapshot.upstream_freshness.usage.status == UpstreamFreshnessStatus::Current
    {
        1
    } else {
        2
    };
    packet[4..8].copy_from_slice(&sent.to_le_bytes());
    packet[8..12].copy_from_slice(&confirmed.to_le_bytes());
    Some(packet)
}

#[cfg(windows)]
#[path = "ble_windows.rs"]
pub mod native;
