use chrono::{DateTime, Utc};
use codex_quota_windows_core::{ble::encode_quota, *};

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-10-14T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}
fn snapshot() -> QuotaSnapshot {
    QuotaSnapshot {
        window_presence: codex_quota_windows_core::QuotaWindowPresence::default(),
        protocol_version: 1,
        generated_at: "2026-10-14T10:00:00Z".into(),
        source_status: QuotaSourceStatus::Ok,
        limits_collected_at: Some("2026-10-14T09:59:30Z".into()),
        windows: vec![
            QuotaWindow {
                id: "codex:primary:300".into(),
                name: "five_hour".into(),
                window_minutes: 300,
                remaining_percent: Some(68),
                resets_at: "2026-10-14T12:00:00Z".into(),
                status: QuotaWindowStatus::Current,
            },
            QuotaWindow {
                id: "codex:secondary:10080".into(),
                name: "weekly".into(),
                window_minutes: 10080,
                remaining_percent: Some(42),
                resets_at: "2026-10-18T10:00:00Z".into(),
                status: QuotaWindowStatus::Current,
            },
        ],
        reset_inventory: ResetInventorySnapshot {
            status: ResetInventoryStatus::Missing,
            available_count: None,
            cached_at: None,
            items: vec![],
        },
        link: QuotaLink {
            computer: ComputerLinkStatus::Online,
            codex: CodexLinkStatus::Ok,
        },
        upstream_freshness: UpstreamFreshness {
            usage: UpstreamDatasetFreshness {
                status: UpstreamFreshnessStatus::Current,
                last_attempt_at: None,
                last_success_at: Some("2026-10-14T09:59:30Z".into()),
            },
            ..UpstreamFreshness::default()
        },
    }
}
#[test]
fn fits_default_mtu_and_carries_source_confirmation() {
    let p = encode_quota(&snapshot(), now()).unwrap();
    assert_eq!(p.len(), 20);
    assert_eq!(&p[..4], &[2, 1, 68, 42]);
    assert_eq!(
        u32::from_le_bytes(p[4..8].try_into().unwrap()),
        now().timestamp() as u32
    );
    assert_eq!(
        u32::from_le_bytes(p[8..12].try_into().unwrap()),
        now().timestamp() as u32 - 30
    );
}
#[test]
fn sending_cached_data_does_not_refresh_confirmation() {
    let s = snapshot();
    let a = encode_quota(&s, now()).unwrap();
    let b = encode_quota(&s, now() + chrono::Duration::seconds(121)).unwrap();
    assert_eq!(&a[8..12], &b[8..12]);
    assert_eq!(b[1], 2);
}
#[test]
fn unknown_or_expired_windows_are_never_guessed() {
    let mut s = snapshot();
    s.windows[0].remaining_percent = None;
    s.windows[1].resets_at = "2026-10-14T09:00:00Z".into();
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(&p[2..4], &[255, 255]);
    assert_eq!(&p[16..20], &[0, 0, 0, 0]);
}
#[test]
fn pending_and_invalid_values_are_unknown() {
    let mut s = snapshot();
    s.windows[0].status = QuotaWindowStatus::PendingSync;
    s.windows[1].remaining_percent = Some(101);
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(&p[2..4], &[255, 255]);
}
#[test]
fn unavailable_confirmation_preserves_cache_without_green_status() {
    let mut s = snapshot();
    s.upstream_freshness.usage = UpstreamDatasetFreshness {
        status: UpstreamFreshnessStatus::Unavailable,
        last_attempt_at: None,
        last_success_at: None,
    };
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(p[1], 2);
    assert_eq!(p[2], 68);
    assert_eq!(&p[8..12], &[0, 0, 0, 0]);
}
#[test]
fn future_confirmation_cannot_claim_freshness() {
    let mut s = snapshot();
    s.upstream_freshness.usage.last_success_at = Some("2026-10-14T11:00:00Z".into());
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(p[1], 2);
    assert_eq!(&p[8..12], &[0, 0, 0, 0]);
}
#[test]
fn malformed_reset_and_out_of_range_clock_are_safe() {
    let mut s = snapshot();
    s.windows[0].resets_at = "invalid".into();
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(p[2], 255);
    assert_eq!(&p[12..16], &[0, 0, 0, 0]);
    assert!(encode_quota(&s, DateTime::from_timestamp(-1, 0).unwrap()).is_none());
}

#[test]
fn source_slot_ids_do_not_determine_window_role() {
    let mut s = snapshot();
    s.windows[0].id = "codex:secondary:300".into();
    s.windows[1].id = "codex:primary:10080".into();
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(&p[..4], &[2, 1, 68, 42]);
}

#[test]
fn confirmed_weekly_only_plan_has_not_applicable_five_hour() {
    let mut s = snapshot();
    s.windows.remove(0);
    s.window_presence = QuotaWindowPresence {
        five_hour: Some(false),
        weekly: Some(true),
    };
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(p[0], 2);
    assert_eq!(p[2], 254);
    assert_eq!(p[3], 42);
}
#[test]
fn missing_window_without_plan_evidence_remains_unknown() {
    let mut s = snapshot();
    s.windows.remove(0);
    s.window_presence = QuotaWindowPresence::default();
    let p = encode_quota(&s, now()).unwrap();
    assert_eq!(p[2], 255);
}
