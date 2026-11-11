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
