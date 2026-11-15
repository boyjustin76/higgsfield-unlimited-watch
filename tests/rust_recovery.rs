use anyhow::Result;
use huw::{
    relay, storage, supervisor,
    transport::{Output, Runner},
};
use serde_json::json;
use std::cell::RefCell;
#[test]
fn owned_lock_cleanup_only() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    storage::private_dir(&d).unwrap();
    let p = d.join("monitor.lock");
    for v in [
        json!({"pid":10}),
        json!({"pid":11,"supervisor_token":"ours"}),
        json!({"pid":10,"supervisor_token":"foreign"}),
    ] {
        storage::atomic(&p, &v).unwrap();
        assert!(supervisor::recover_owned_lock(&d, 10, "ours").is_err());
        assert!(p.exists());
    }
    storage::atomic(&p, &json!({"pid":10,"supervisor_token":"ours"})).unwrap();
    supervisor::recover_owned_lock(&d, 10, "ours").unwrap();
    assert!(!p.exists());
}
#[test]
fn real_child_failure_bounded() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    let r = std::process::Command::new(env!("CARGO_BIN_EXE_huw"))
        .args([
            "supervise",
            "--config",
            t.path().join("absent.json").to_str().unwrap(),
            "--state-dir",
            d.to_str().unwrap(),
            "--asset-id",
            "00000000-0000-4000-8000-000000000001",
            "--max-restarts",
            "1",
            "--backoff",
            "1",
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    let v = storage::read(&d.join("supervisor.json")).unwrap();
    assert_eq!(v["state"], "restart_budget_exhausted");
    assert_eq!(
        std::fs::read_to_string(d.join("supervisor-transitions.jsonl"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    assert!(!d.join("supervisor.lock").exists());
}
#[test]
fn existing_event_survives_supervision_without_browser_calls() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    let bin = env!("CARGO_BIN_EXE_huw");
    assert!(
        std::process::Command::new(bin)
            .args(["demo", "--state-dir"])
            .arg(&d)
            .output()
            .unwrap()
            .status
            .success()
    );
    let before = storage::read(&d.join("event.json")).unwrap();
    let config = t.path().join("config.json");
    std::fs::write(
        &config,
        r#"{"command":["must-not-launch-browser"],"browser":"synthetic"}"#,
    )
    .unwrap();
    let result = std::process::Command::new(bin)
        .args(["supervise", "--config"])
        .arg(config)
        .args(["--state-dir"])
        .arg(&d)
        .args([
            "--asset-id",
            "00000000-0000-4000-8000-000000000001",
            "--max-restarts",
            "0",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(before, storage::read(&d.join("event.json")).unwrap());
    assert_eq!(
        storage::read(&d.join("supervisor.json")).unwrap()["state"],
        "finished"
    );
}

struct RefusedSend(std::cell::Cell<usize>);
impl Runner for RefusedSend {
    fn run(&self, args: &[String], _: f64) -> Result<Output> {
        self.0.set(self.0.get() + 1);
        Ok(Output {
            success: true,
            stdout: if args[1] == "get" {
                json!({"address":"t1","state":"live","status":{"app":"codex","state":"idle"}})
                    .to_string()
            } else {
                json!({"started":false}).to_string()
            },
        })
    }
}
#[test]
fn refused_send_receipt_never_retried() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("job");
    storage::atomic(&d.join("event.json"), &json!({"state":"completed","asset_id":"00000000-0000-4000-8000-000000000001","event_id":"aaaaaaaaaaaaaaaaaaaaaaaa"})).unwrap();
    let runner = RefusedSend(std::cell::Cell::new(0));
    let first = relay::relay(&d, "hc", "t1", "codex", &runner).unwrap();
    assert_eq!(first["state"], "delivery_unknown");
    assert_eq!(
        relay::relay(&d, "hc", "t1", "codex", &runner).unwrap(),
        first
    );
    assert_eq!(runner.0.get(), 2);
}
struct Deferred {
    calls: RefCell<Vec<String>>,
    working: RefCell<bool>,
    blocked: bool,
}
impl Runner for Deferred {
    fn run(&self, args: &[String], _: f64) -> Result<Output> {
        self.calls.borrow_mut().push(args[1].clone());
        let data = match args[1].as_str() {
            "get" => {
                json!({"address":"t1","state":"live","status":{"app":"codex","state":if self.blocked{"blocked"}else if *self.working.borrow(){"working"}else{"idle"}}})
            }
            "wait" => {
                *self.working.borrow_mut() = false;
                json!({"reason":"status"})
            }
            "send" => json!({"started":true}),
            _ => panic!(),
        };
        Ok(Output {
            success: true,
            stdout: data.to_string(),
        })
    }
}
#[test]
fn deferred_wait_then_send() {
    for blocked in [false, true] {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("job");
        storage::atomic(&d.join("event.json"),&json!({"state":"completed","asset_id":"00000000-0000-4000-8000-000000000001","event_id":"aaaaaaaaaaaaaaaaaaaaaaaa"})).unwrap();
        let f = Deferred {
            calls: RefCell::new(vec![]),
            working: RefCell::new(true),
            blocked,
        };
        let r = relay::deferred(&d, "hc", "t1", "codex", &f, 10.0);
        if blocked {
            assert!(r.is_err());
            assert_eq!(*f.calls.borrow(), vec!["get"]);
        } else {
            assert_eq!(r.unwrap()["state"], "accepted");
            assert_eq!(*f.calls.borrow(), vec!["get", "wait", "get", "get", "send"]);
        }
    }
}
