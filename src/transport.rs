use anyhow::{Result, bail};
use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use wait_timeout::ChildExt;
#[derive(Debug)]
pub struct Output {
    pub success: bool,
    pub stdout: String,
}
pub trait Runner {
    fn run(&self, args: &[String], timeout: f64) -> Result<Output>;
}
pub struct Process;
impl Runner for Process {
    fn run(&self, args: &[String], timeout: f64) -> Result<Output> {
        if args.is_empty() || !timeout.is_finite() || timeout <= 0.0 {
            bail!("invalid process invocation");
        }
        let mut c = Command::new(&args[0]);
        c.args(&args[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x08000000);
        }
        let mut child = c
            .spawn()
            .map_err(|_| anyhow::anyhow!("transport spawn failed"))?;
        let mut stdout = child.stdout.take().unwrap();
        let reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        });
        let status = child.wait_timeout(Duration::from_secs_f64(timeout))?;
        if status.is_none() {
            let _ = child.kill();
            let _ = child.wait();
            bail!("transport timeout");
        }
        let bytes = reader
            .join()
            .map_err(|_| anyhow::anyhow!("transport read failed"))?;
        Ok(Output {
            success: status.unwrap().success(),
            stdout: String::from_utf8_lossy(&bytes).trim().into(),
        })
    }
}
