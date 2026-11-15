use crate::{dom, storage};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
pub fn asset(value: &str) -> Result<String> {
    let parsed = uuid::Uuid::parse_str(value)
        .map_err(|_| anyhow::anyhow!("asset_id must be canonical UUID"))?;
    let lower = value.to_lowercase();
    if parsed.to_string() != lower {
        bail!("asset_id must be canonical UUID");
    }
    Ok(lower)
}
pub trait Clock {
    fn now(&self) -> f64;
    fn sleep(&mut self, seconds: f64);
    fn cancelled(&self) -> bool {
        false
    }
}
pub struct RealClock<'a> {
    pub start: Instant,
    pub cancel: &'a AtomicBool,
}
impl Clock for RealClock<'_> {
    fn now(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
    fn sleep(&mut self, s: f64) {
        let end = Instant::now() + Duration::from_secs_f64(s);
        while Instant::now() < end && !self.cancelled() {
            std::thread::sleep(
                Duration::from_millis(100).min(end.saturating_duration_since(Instant::now())),
            );
        }
    }
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}
pub fn terminal(v: &Value) -> bool {
    matches!(v["state"].as_str(), Some("completed" | "failed"))
}
pub fn watch(
    mut observe: impl FnMut(&str) -> Result<Value>,
    aid: &str,
    dir: &Path,
    interval: f64,
    timeout: f64,
    resume: bool,
    clock: &mut dyn Clock,
) -> Result<Value> {
    let aid = asset(aid)?;
    if !interval.is_finite() || !timeout.is_finite() || interval < 5.0 || timeout <= 0.0 {
        bail!("interval >=5 and finite timeout >0 required");
    }
    let _lock = storage::Lock::acquire(dir, "monitor.lock")?;
    let target = dir.join("target.json");
    if target.exists() && storage::read(&target)?["asset_id"] != aid {
        bail!("state directory belongs to different asset");
    }
    storage::atomic(&target, &json!({"asset_id":aid}))?;
    let event_path = dir.join("event.json");
    let previous = if event_path.exists() {
        Some(storage::read(&event_path)?)
    } else {
        None
    };
    if let Some(ref event) = previous {
        if terminal(event) || !resume {
            return Ok(event.clone());
        }
    }
    let attempt = previous
        .as_ref()
        .and_then(|v| v["attempt"].as_u64())
        .unwrap_or(0)
        + 1;
    let start = clock.now();
    let mut polls = 0;
    let mut consecutive = 0;
    let mut last = Value::Null;
    let mut prior = Value::Null;
    let mut event = loop {
        if clock.cancelled() {
            break json!({"state":"cancelled","reason":"monitor_stopped_job_not_cancelled","asset_id":aid,"attempt":attempt,"polls":polls,"observed_at":storage::now()});
        }
        polls += 1;
        let mut current = observe(&aid).unwrap_or_else(
            |_| json!({"state":"unknown","reason":"browser_unavailable_or_changed"}),
        );
        if !current.is_object() {
            current = json!({"state":"unknown","reason":"invalid_observation"});
        }
        let sig = json!([current["state"], current["dom_status"], current["reason"]]);
        consecutive = if sig == last { consecutive + 1 } else { 1 };
        last = sig.clone();
        current["asset_id"] = json!(aid);
        current["observed_at"] = json!(storage::now());
        current["polls"] = json!(polls);
        current["consecutive_observations"] = json!(consecutive);
        current["attempt"] = json!(attempt);
        current["elapsed_seconds"] = json!(((clock.now() - start) * 1000.0).round() / 1000.0);
        storage::atomic(&dir.join("status.json"), &current)?;
        if sig != prior {
            storage::journal(&dir.join("transitions.jsonl"), &current)?;
            prior = sig;
        }
        if terminal(&current) && consecutive >= 2 {
            break current;
        }
        if clock.now() - start >= timeout {
            current["state"] = json!("timeout");
            current["reason"] = json!("deadline_not_job_failure");
            break current;
        }
        clock.sleep(interval.min((timeout - (clock.now() - start)).max(0.0)));
    };
    event["schema_version"] = json!(1);
    event["event_id"] = json!(
        &dom::digest(&format!(
            "{aid}:{attempt}:{}",
            event["state"].as_str().unwrap()
        ))[..24]
    );
    event["monitor_llm_calls"] = json!(0);
    storage::atomic(&event_path, &event)?;
    Ok(event)
}
