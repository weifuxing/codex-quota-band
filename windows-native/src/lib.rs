//! Quota collection and authenticated Huangshan BLE transport.
pub mod ble;
pub mod migration;
pub mod quota;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaSourceStatus {
    Ok,
    Partial,
    Unavailable,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaWindowStatus {
    Current,
    PendingSync,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetInventoryStatus {
    Cached,
    CachedDerived,
    Missing,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerLinkStatus {
    Online,
    Offline,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexLinkStatus {
    Ok,
    Unavailable,
    Stale,
    FormatChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub id: String,
    pub name: String,
    pub window_minutes: u32,
    pub remaining_percent: Option<u8>,
    pub resets_at: String,
    pub status: QuotaWindowStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetInventoryItem {
    pub id: String,
    pub title: String,
    pub status: ResetItemStatus,
    /// Source-only timing. Snapshot v1 never serializes this field; negotiated
    /// quota v2 carries it in the reduced reset-card shape below.
    #[serde(skip_serializing, default)]
    pub granted_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetItemStatus {
    Available,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetInventorySnapshot {
    pub status: ResetInventoryStatus,
    pub available_count: Option<u32>,
    pub cached_at: Option<String>,
    pub items: Vec<ResetInventoryItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaLink {
    pub computer: ComputerLinkStatus,
    pub codex: CodexLinkStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamFreshnessStatus {
    Current,
    Cached,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamDatasetFreshness {
    pub status: UpstreamFreshnessStatus,
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamFreshness {
    pub usage: UpstreamDatasetFreshness,
    pub reset_inventory: UpstreamDatasetFreshness,
}

impl Default for UpstreamFreshness {
    fn default() -> Self {
        let unavailable = UpstreamDatasetFreshness {
            status: UpstreamFreshnessStatus::Unavailable,
            last_attempt_at: None,
            last_success_at: None,
        };
        Self {
            usage: unavailable.clone(),
            reset_inventory: unavailable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    /// Source evidence retained locally for BLE window applicability.
    #[serde(skip)]
    pub window_presence: QuotaWindowPresence,
    pub protocol_version: u8,
    pub generated_at: String,
    pub source_status: QuotaSourceStatus,
    pub limits_collected_at: Option<String>,
    pub windows: Vec<QuotaWindow>,
    pub reset_inventory: ResetInventorySnapshot,
    pub link: QuotaLink,
    #[serde(skip_serializing, default)]
    pub upstream_freshness: UpstreamFreshness,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindowPresence {
    pub five_hour: Option<bool>,
    pub weekly: Option<bool>,
}
