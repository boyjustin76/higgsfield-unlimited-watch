use anyhow::{Result, bail};
use huw::{
    monitor::{self, Clock},
    relay, storage,
    transport::{Output, Runner},
};
use serde_json::{Value, json};
use std::{cell::RefCell, path::Path};
const A: &str = "00000000-0000-4000-8000-000000000001";
const B: &str = "00000000-0000-4000-8000-000000000002";
struct Tick(f64, bool);
impl Clock for Tick {
    fn now(&self) -> f64 {
        self.0
    }
    fn sleep(&mut self, s: f64) {
        self.0 += s;
    }
    fn cancelled(&self) -> bool {
        self.1
    }
}
fn watch(dir: &Path, states: Vec<&str>, timeout: f64, resume: bool) -> Result<Value> {
    let mut seq = states.into_iter();
    monitor::watch(
        |_| {
            let s = seq.next().expect("unexpected observation");
            if s == "lost" {
                bail!("lost");
            }
            Ok(json!({"state":s}))
        },
        A,
        dir,
        20.0,
        timeout,
        resume,
        &mut Tick(0.0, false),
    )
}
#[test]
fn confirmations_and_replay() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    let e = watch(&d, vec!["pending", "completed", "completed"], 100.0, false).unwrap();
    assert_eq!(e["polls"], 3);
    assert_eq!(e["monitor_llm_calls"], 0);
    assert_eq!(watch(&d, vec![], 100.0, false).unwrap(), e);
    assert!(!d.join("monitor.lock").exists());
}
#[test]
fn disruption_resets() {
    let t = tempfile::tempdir().unwrap();
    let e = watch(
        &t.path().join("job"),
        vec!["completed", "lost", "completed", "completed"],
        100.0,
        false,
    )
    .unwrap();
    assert_eq!(e["polls"], 4);
}
#[test]
fn timeout_resume() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    let e = watch(&d, vec!["unknown", "unknown"], 5.0, false).unwrap();
    assert_eq!(e["state"], "timeout");
    assert_eq!(watch(&d, vec![], 100.0, false).unwrap(), e);
    assert_eq!(
        watch(&d, vec!["completed", "completed"], 100.0, true).unwrap()["attempt"],
        2
    );
}
#[test]
fn failure() {
    let t = tempfile::tempdir().unwrap();
    assert_eq!(
        watch(
            &t.path().join("job"),
            vec!["failed", "failed"],
            100.0,
            false
        )
        .unwrap()["state"],
        "failed"
    );
}
#[test]
fn immutable_asset() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    watch(&d, vec!["completed", "completed"], 100.0, false).unwrap();
    assert!(
        monitor::watch(
            |_| panic!(),
            B,
            &d,
            20.0,
            100.0,
            false,
            &mut Tick(0.0, false)
        )
        .is_err()
    );
}
#[test]
fn exclusive() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    let _guard = storage::Lock::acquire(&d, "monitor.lock").unwrap();
    assert!(watch(&d, vec![], 100.0, false).is_err());
}
#[test]
fn cancelled_not_job_cancel() {
    let t = tempfile::tempdir().unwrap();
    let e = monitor::watch(
        |_| panic!(),
        A,
        &t.path().join("job"),
        20.0,
        100.0,
        false,
        &mut Tick(0.0, true),
    )
    .unwrap();
    assert_eq!(e["state"], "cancelled");
    assert!(e["reason"].as_str().unwrap().contains("job_not_cancelled"));
}
#[test]
fn invalid_asset() {
    assert!(monitor::asset("../secret").is_err());
    assert!(monitor::asset(&A.replace('-', "")).is_err());
}
#[test]
fn finite_deadlines() {
    let t = tempfile::tempdir().unwrap();
    for v in [f64::NAN, f64::INFINITY] {
        assert!(watch(&t.path().join("job"), vec![], v, false).is_err());
        assert!(
            monitor::watch(
                |_| panic!(),
                A,
                &t.path().join("job"),
                v,
                100.0,
                false,
                &mut Tick(0.0, false)
            )
            .is_err()
        );
    }
}
struct Fake {
    calls: RefCell<usize>,
    target: Value,
    send: u8,
}
impl Runner for Fake {
    fn run(&self, args: &[String], _: f64) -> Result<Output> {
        *self.calls.borrow_mut() += 1;
        if args[1] == "get" {
            return Ok(Output {
                success: true,
                stdout: self.target.to_string(),
            });
        }
        match self.send {
            1 => bail!("timeout"),
            2 => panic!("simulated process crash"),
            3 => Ok(Output {
                success: true,
                stdout: json!({"started":false}).to_string(),
            }),
            _ => Ok(Output {
                success: true,
                stdout: json!({"started":true}).to_string(),
            }),
        }
    }
}
fn fake(send: u8) -> Fake {
    Fake {
        calls: RefCell::new(0),
        target: json!({"address":"t1","state":"live","status":{"app":"copilot","state":"idle"}}),
        send,
    }
}
fn completed(d: &Path) -> Value {
    watch(d, vec!["completed", "completed"], 100.0, false).unwrap()
}
#[test]
fn relay_once() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    completed(&d);
    let f = fake(0);
    let a = relay::relay(&d, "hc", "t1", "copilot", &f).unwrap();
    assert_eq!(a["state"], "accepted");
    assert_eq!(a, relay::relay(&d, "hc", "t1", "copilot", &f).unwrap());
    assert_eq!(*f.calls.borrow(), 2);
}
#[test]
fn ambiguous_never_retry() {
    for mode in [1, 3] {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("job");
        completed(&d);
        let f = fake(mode);
        assert_eq!(
            relay::relay(&d, "hc", "t1", "copilot", &f).unwrap()["state"],
            "delivery_unknown"
        );
        relay::relay(&d, "hc", "t1", "copilot", &f).unwrap();
        assert_eq!(*f.calls.borrow(), 2);
    }
}
#[test]
fn receipt_before_crash() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    completed(&d);
    let f = fake(2);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| relay::relay(
            &d, "hc", "t1", "copilot", &f
        )))
        .is_err()
    );
    assert_eq!(
        relay::relay(&d, "hc", "t1", "copilot", &f).unwrap()["state"],
        "attempting"
    );
    assert_eq!(*f.calls.borrow(), 2);
}
#[test]
fn wrong_recipient_without_receipt() {
    for (app, address, state) in [
        ("bash", "t1", "idle"),
        ("vim", "t1", "idle"),
        ("codex", "t1", "idle"),
        ("copilot", "t2", "idle"),
        ("copilot", "t1", "blocked"),
        ("copilot", "t1", "working"),
    ] {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("job");
        let e = completed(&d);
        let mut f = fake(0);
        f.target = json!({"address":address,"state":"live","status":{"app":app,"state":state}});
        assert!(relay::relay(&d, "hc", "t1", "copilot", &f).is_err());
        assert_eq!(*f.calls.borrow(), 1);
        assert!(
            !d.join(format!("relay-t1-{}.json", e["event_id"].as_str().unwrap()))
                .exists()
        );
    }
}
#[test]
fn no_agent_status() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    completed(&d);
    let mut f = fake(0);
    f.target = json!({"address":"t1","state":"live","status":null});
    assert!(relay::relay(&d, "hc", "t1", "copilot", &f).is_err());
}
#[test]
fn unsupported_app() {
    let t = tempfile::tempdir().unwrap();
    assert!(relay::relay(t.path(), "hc", "t1", "bash", &fake(0)).is_err());
}
#[test]
fn legacy_receipt() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    let e = completed(&d);
    storage::atomic(
        &d.join(format!("relay-t1-{}.json", e["event_id"].as_str().unwrap())),
        &json!({"state":"attempting"}),
    )
    .unwrap();
    let f = fake(0);
    assert!(relay::relay(&d, "hc", "t1", "copilot", &f).is_err());
    assert_eq!(*f.calls.borrow(), 0);
}
#[test]
fn cli_notify_pair() {
    use clap::Parser;
    assert!(
        huw::cli::Cli::try_parse_from([
            "huw",
            "watch",
            "--config",
            "unused",
            "--asset-id",
            A,
            "--state-dir",
            "unused",
            "--notify-terminal",
            "t1"
        ])
        .is_err()
    );
}
#[test]
fn offline_demo() {
    let t = tempfile::tempdir().unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_huw"))
        .args([
            "demo",
            "--state-dir",
            t.path().join("job").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["synthetic"], true);
    assert_eq!(v["event"]["state"], "completed");
}
