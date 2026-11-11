#![cfg(windows)]
use huw::storage;
use serde_json::json;
use std::{fs, os::windows::fs::OpenOptionsExt, time::Duration};
#[test]
fn transient_sharing_lock_keeps_old_until_atomic_replace() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("event.json");
    fs::write(&p, "old").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&p)
        .unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        drop(held);
    });
    storage::atomic(&p, &json!({"new":true})).unwrap();
    release.join().unwrap();
    assert_eq!(storage::read(&p).unwrap()["new"], true);
}
#[test]
fn permanent_sharing_failure_preserves_old() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("event.json");
    fs::write(&p, "old").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&p)
        .unwrap();
    assert!(storage::atomic(&p, &json!({"new":true})).is_err());
    drop(held);
    assert_eq!(fs::read_to_string(&p).unwrap(), "old");
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 1);
}
