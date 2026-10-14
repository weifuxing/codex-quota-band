//! One-time removal of this product's retired Hook registrations.
use serde_json::Value;
pub fn without_legacy_hooks(existing: &str) -> Result<Option<String>, serde_json::Error> {
    let mut root: Value = serde_json::from_str(existing)?;
    let mut changed = false;
    if let Some(events) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        for groups in events.values_mut().filter_map(Value::as_array_mut) {
            for group in groups.iter_mut() {
                if let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                    let before = handlers.len();
                    handlers.retain(|h| {
                        !["command", "commandWindows"].iter().any(|key| {
                            h.get(*key)
                                .and_then(Value::as_str)
                                .is_some_and(|c| c.contains("--owner codex-quota"))
                        })
                    });
                    changed |= before != handlers.len();
                }
            }
            groups.retain(|g| {
                !g.get("hooks")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            });
        }
    }
    if changed {
        Ok(Some(serde_json::to_string_pretty(&root)?))
    } else {
        Ok(None)
    }
}
pub fn cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::var_os("CODEX_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|p| std::path::PathBuf::from(p).join(".codex"))
        });
    let Some(directory) = directory else {
        return Ok(());
    };
    let path = directory.join("hooks.json");
    let existing = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    if let Some(updated) = without_legacy_hooks(&existing)? {
        use std::io::Write;
        let backup = directory.join(format!(
            "hooks.codex-quota-backup-{}.json",
            chrono::Utc::now().timestamp_millis()
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(backup)?;
        file.write_all(existing.as_bytes())?;
        file.sync_all()?;
        std::fs::write(path, updated)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_other_hooks_and_settings() {
        let input = r#"{"other":7,"hooks":{"Stop":[{"matcher":"x","hooks":[{"command":"a --owner codex-quota"},{"command":"another"}]}]}}"#;
        let result: Value =
            serde_json::from_str(&without_legacy_hooks(input).unwrap().unwrap()).unwrap();
        assert_eq!(result["other"], 7);
        assert_eq!(result["hooks"]["Stop"][0]["matcher"], "x");
        assert_eq!(
            result["hooks"]["Stop"][0]["hooks"],
            serde_json::json!([{"command":"another"}])
        );
    }
    #[test]
    fn unchanged_config_is_not_rewritten() {
        assert!(
            without_legacy_hooks(r#"{"hooks":{"Stop":[{"hooks":[{"command":"another"}]}]}}"#)
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn invalid_json_is_never_replaced() {
        assert!(without_legacy_hooks("invalid").is_err());
    }
}
