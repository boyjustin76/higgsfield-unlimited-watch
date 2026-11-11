use crate::{cli::WatchArgs, dom, monitor, relay, storage, transport::Process};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
pub fn recover_owned_lock(dir: &Path, pid: u32, token: &str) -> Result<()> {
    let p = dir.join("monitor.lock");
    if !p.exists() {
        return Ok(());
    }
    let v = storage::read(&p)?;
    if v["pid"] != pid || v["supervisor_token"] != token {
        bail!("foreign or legacy monitor lock; manual recovery required");
    }
    std::fs::remove_file(p)?;
    Ok(())
}
pub fn supervise(a: WatchArgs, max: u32, backoff: f64, cancel: &AtomicBool) -> Result<Value> {
    monitor::asset(&a.asset_id)?;
    if max > 10 || !backoff.is_finite() || !(1.0..=300.0).contains(&backoff) {
        bail!("restart budget <=10 and backoff 1..300 required");
    }
    let _guard = storage::Lock::acquire(&a.state_dir, "supervisor.lock")?;
    let executable = std::env::current_exe()?;
    for restart in 0..=max {
        if cancel.load(Ordering::Relaxed) {
            bail!("supervisor cancelled; remote job not cancelled");
        }
        let token = dom::digest(&format!(
            "{}:{}:{restart}",
            std::process::id(),
            storage::now()
        ));
        let mut c = Command::new(&executable);
        c.args(["watch", "--config"])
            .arg(&a.config)
            .args(["--asset-id", &a.asset_id, "--state-dir"])
            .arg(&a.state_dir)
            .args([
                "--interval",
                &a.interval.to_string(),
                "--timeout",
                &a.timeout.to_string(),
            ])
            .env("HUW_SUPERVISOR_TOKEN", &token)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if restart > 0 || a.resume {
            c.arg("--resume");
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x08000000);
        }
        let mut child = c.spawn()?;
        let pid = child.id();
        storage::atomic(
            &a.state_dir.join("supervisor.json"),
            &json!({"state":"running","restart":restart,"child_pid":pid,"asset_id":a.asset_id,"started_at":storage::now()}),
        )?;
        let status = loop {
            if let Some(s) = child.try_wait()? {
                break s;
            }
            if cancel.load(Ordering::Relaxed) {
                child.kill()?;
                child.wait()?;
                recover_owned_lock(&a.state_dir, pid, &token)?;
                bail!("supervisor cancelled; remote job not cancelled");
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        // Only a child handle that we spawned and have reaped permits token-bound lock cleanup.
        recover_owned_lock(&a.state_dir, pid, &token)?;
        if status.success() {
            let event = storage::read(&a.state_dir.join("event.json"))?;
            if event["asset_id"] != a.asset_id {
                bail!("supervised asset changed");
            }
            storage::atomic(
                &a.state_dir.join("supervisor.json"),
                &json!({"state":"finished","restarts":restart,"asset_id":a.asset_id}),
            )?;
            return if let (Some(target), Some(app)) = (&a.notify_terminal, &a.notify_app) {
                Ok(
                    json!({"event":event,"relay":relay::deferred(&a.state_dir,&a.hc,target,app,&Process,a.relay_wait)?}),
                )
            } else {
                Ok(event)
            };
        }
        storage::journal(
            &a.state_dir.join("supervisor-transitions.jsonl"),
            &json!({"state":"child_failed","restart":restart,"exit_code":status.code(),"observed_at":storage::now()}),
        )?;
        if restart < max {
            let end = Instant::now()
                + Duration::from_secs_f64((backoff * 2_f64.powi(restart as i32)).min(300.0));
            while Instant::now() < end && !cancel.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
    storage::atomic(
        &a.state_dir.join("supervisor.json"),
        &json!({"state":"restart_budget_exhausted","asset_id":a.asset_id,"max_restarts":max}),
    )?;
    bail!("watcher restart budget exhausted; inspect supervisor.json; remote job unchanged")
}
