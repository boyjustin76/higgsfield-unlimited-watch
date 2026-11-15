use anyhow::{Result, bail};
use std::{
    io::{Read, Seek, SeekFrom},
    ops::{Deref, DerefMut},
    process::{Child, Command, Stdio},
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
/// A spawned local helper must be reaped on every return path.
pub struct OwnedChild(pub Child);
impl OwnedChild {
    pub fn terminate(&mut self) -> std::io::Result<()> {
        if self.0.try_wait()?.is_none() {
            self.0.kill()?;
        }
        self.0.wait()?;
        Ok(())
    }
}
impl Deref for OwnedChild {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}
impl DerefMut for OwnedChild {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
impl Runner for Process {
    fn run(&self, args: &[String], timeout: f64) -> Result<Output> {
        if args.is_empty() || !timeout.is_finite() || timeout <= 0.0 {
            bail!("invalid process invocation");
        }
        // An anonymous temporary file avoids waiting forever for EOF from a
        // descendant that inherited stdout after the direct helper has exited.
        // Only the direct helper's bounded-lifetime output is consumed.
        let mut stdout = tempfile::tempfile()?;
        let mut c = Command::new(&args[0]);
        c.args(&args[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.try_clone()?))
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x08000000);
        }
        let mut child = OwnedChild(
            c.spawn()
                .map_err(|_| anyhow::anyhow!("transport spawn failed"))?,
        );
        let status = child.wait_timeout(Duration::from_secs_f64(timeout))?;
        if status.is_none() {
            bail!("transport timeout");
        }
        // Bound the snapshot read, not the amount a trusted child could write
        // during its lifetime. Asynchronous descendant output is unsupported.
        const MAX_OUTPUT: u64 = 16 * 1024 * 1024;
        let length = stdout.metadata()?.len();
        if length > MAX_OUTPUT {
            bail!("transport output too large");
        }
        stdout.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        stdout.take(length).read_to_end(&mut bytes)?;
        Ok(Output {
            success: status.unwrap().success(),
            stdout: String::from_utf8_lossy(&bytes).trim().into(),
        })
    }
}
