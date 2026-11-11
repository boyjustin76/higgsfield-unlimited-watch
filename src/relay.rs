use crate::{monitor, storage, transport::Runner};
pub fn deferred(
    dir: &Path,
    hc: &str,
    target: &str,
    app: &str,
    runner: &dyn Runner,
    wait: f64,
) -> Result<Value> {
    if !wait.is_finite() || wait < 0.0 {
        bail!("invalid relay wait");
    }
    if wait == 0.0 {
        return relay(dir, hc, target, app, runner);
    }
    let start = std::time::Instant::now();
    loop {
        let r = runner.run(&[hc.into(), "get".into(), target.into()], 10.0)?;
        let info: Value = serde_json::from_str(&r.stdout)?;
        if !r.success
            || info["address"] != target
            || info["state"] != "live"
            || info["status"]["app"] != app
            || matches!(info["status"]["state"].as_str(), Some("blocked" | "error"))
        {
            bail!("recipient unavailable; event retained");
        }
        if matches!(info["status"]["state"].as_str(), Some("idle" | "done")) {
            return relay(dir, hc, target, app, runner);
        }
        let left = wait - start.elapsed().as_secs_f64();
        if left <= 0.0 {
            bail!("relay wait expired; event retained");
        }
        runner.run(
            &[
                hc.into(),
                "wait".into(),
                target.into(),
                "--timeout".into(),
                format!("{}", left.min(60.0).ceil() as u64),
            ],
            70.0,
        )?;
    }
}
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::Path;
pub fn relay(dir: &Path, hc: &str, target: &str, app: &str, runner: &dyn Runner) -> Result<Value> {
    if !target.starts_with('t')
        || target.len() < 2
        || !target[1..]
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        bail!("notify target must be local terminal ID");
    }
    if !matches!(app, "codex" | "copilot") {
        bail!("unsupported notify app");
    }
    storage::private_dir(dir)?;
    let event = storage::read(&dir.join("event.json"))?;
    if !monitor::terminal(&event)
        && !matches!(event["state"].as_str(), Some("timeout" | "cancelled"))
    {
        bail!("not terminal monitor event");
    }
    let aid = monitor::asset(event["asset_id"].as_str().unwrap_or(""))?;
    let id = event["event_id"].as_str().unwrap_or("");
    if id.len() != 24
        || !id
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    {
        bail!("invalid event identity");
    }
    let path = dir.join(format!("relay-{target}-{id}.json"));
    let _lock = storage::Lock::acquire(dir, "relay.lock")?;
    if path.exists() {
        let v = storage::read(&path)?;
        if v["expected_app"] != app {
            bail!("different or legacy app binding; inspect before recovery");
        }
        return Ok(v);
    }
    let r = runner.run(&[hc.into(), "get".into(), target.into()], 10.0)?;
    let info: Value = if r.success {
        serde_json::from_str(&r.stdout)?
    } else {
        Value::Null
    };
    if info["address"] != target
        || info["state"] != "live"
        || info["status"]["app"] != app
        || !matches!(info["status"]["state"].as_str(), Some("idle" | "done"))
    {
        bail!("relay target must match idle/done agent app; no send");
    }
    let mut receipt = json!({"state":"attempting","event_id":id,"target":target,"expected_app":app,"attempted_at":storage::now()});
    storage::atomic(&path, &receipt)?;
    let message = format!(
        "[HUW_EVENT] asset_id={aid} state={} event_id={id}. Recheck the Higgsfield web session and review the result. This event does not authorize another generation. Concurrency remains one.",
        event["state"].as_str().unwrap()
    );
    let accepted = match runner.run(&[hc.into(), "send".into(), target.into(), message], 30.0) {
        Ok(r) if r.success => serde_json::from_str::<Value>(&r.stdout)
            .map(|v| v["started"] != false)
            .unwrap_or(true),
        _ => false,
    };
    receipt["state"] = json!(if accepted {
        "accepted"
    } else {
        "delivery_unknown"
    });
    storage::atomic(&path, &receipt)?;
    Ok(receipt)
}
