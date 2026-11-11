#![cfg(unix)]
use huw::storage;
use serde_json::json;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};
#[test]
fn artifacts_private() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("new-parent/job");
    storage::atomic(&d.join("event.json"), &json!({})).unwrap();
    storage::journal(&d.join("transitions.jsonl"), &json!({})).unwrap();
    let _l = storage::Lock::acquire(&d, "monitor.lock").unwrap();
    for e in fs::read_dir(&d).unwrap() {
        assert_eq!(
            e.unwrap().metadata().unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    for p in [d.as_path(), d.parent().unwrap()] {
        assert_eq!(fs::metadata(p).unwrap().permissions().mode() & 0o777, 0o700);
    }
}
#[test]
fn broad_directory_rejected_without_chmod() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("broad");
    fs::create_dir(&d).unwrap();
    fs::set_permissions(&d, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(storage::atomic(&d.join("x"), &json!({})).is_err());
    assert_eq!(
        fs::metadata(&d).unwrap().permissions().mode() & 0o777,
        0o755
    );
}
#[test]
fn tighten_only_targets() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path();
    for name in ["event.json", "journal", "unrelated"] {
        fs::write(d.join(name), "{}").unwrap();
        fs::set_permissions(d.join(name), fs::Permissions::from_mode(0o644)).unwrap();
    }
    storage::atomic(&d.join("event.json"), &json!({})).unwrap();
    storage::journal(&d.join("journal"), &json!({})).unwrap();
    assert_eq!(
        fs::metadata(d.join("journal"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(d.join("unrelated"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );
}
#[test]
fn symlinks_rejected() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path();
    symlink(d, d.join("link")).unwrap();
    assert!(storage::private_dir(&d.join("link")).is_err());
    fs::write(d.join("original"), "unchanged").unwrap();
    symlink(d.join("original"), d.join("journal")).unwrap();
    assert!(storage::journal(&d.join("journal"), &json!({})).is_err());
    assert_eq!(fs::read_to_string(d.join("original")).unwrap(), "unchanged");
}
#[test]
fn atomic_failure_cleans_temp() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path();
    fs::create_dir(d.join("event.json")).unwrap();
    assert!(storage::atomic(&d.join("event.json"), &json!({})).is_err());
    assert_eq!(fs::read_dir(d).unwrap().count(), 1);
}
