use huw::transport::{Process, Runner};
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
#[ignore = "subprocess fixture, invoked explicitly"]
fn fixture_hold_stdout() {
    std::thread::sleep(Duration::from_secs(3));
}

#[test]
#[ignore = "subprocess fixture, invoked explicitly"]
fn fixture_spawns_descendant() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--ignored", "--exact", "fixture_hold_stdout", "--nocapture"])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let _child = command.spawn().unwrap();
    println!("parent finished");
}

#[test]
fn inherited_stdout_cannot_extend_transport_deadline() {
    let args = vec![
        std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        "--ignored".into(),
        "--exact".into(),
        "fixture_spawns_descendant".into(),
        "--nocapture".into(),
    ];
    let start = Instant::now();
    let result = Process.run(&args, 1.0).unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("parent finished"));
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "descendant stdout extended the deadline"
    );
}
