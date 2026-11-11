use anyhow::{Result, bail};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
}
pub fn private_dir(p: &Path) -> Result<()> {
    if !p.exists() {
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                private_dir(parent)?;
            }
        }
        #[allow(unused_mut)]
        let mut b = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            b.mode(0o700);
        }
        match b.create(p) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
    }
    let m = fs::symlink_metadata(p)?;
    if !m.is_dir() || m.file_type().is_symlink() {
        bail!("state directory must be real directory, not symlink");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.mode() & 0o777 != 0o700 || m.uid() != unsafe { libc::geteuid() } {
            bail!("state directory must be owned by user with mode 0700");
        }
    }
    Ok(())
}
pub fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
pub fn atomic(p: &Path, v: &Value) -> Result<()> {
    let parent = p
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent"))?;
    private_dir(parent)?;
    let mut tmp = tempfile::Builder::new()
        .prefix(".huw-")
        .suffix(".tmp")
        .tempfile_in(parent)?;
    serde_json::to_writer_pretty(&mut tmp, v)?;
    tmp.write_all(b"\n")?;
    tmp.as_file().sync_all()?;
    // Close our handle before the rename, matching the Python write/close/replace order.
    // Windows sharing/access failures can be transient; never delete the destination
    // or alter ACLs. Retain the old file and bound retries to 350 ms.
    let mut path = tmp.into_temp_path();
    for attempt in 0..4 {
        match path.persist(p) {
            Ok(()) => return Ok(()),
            Err(e) => {
                let transient =
                    cfg!(windows) && matches!(e.error.raw_os_error(), Some(5 | 32 | 33));
                if !transient || attempt == 3 {
                    return Err(e.error.into());
                }
                path = e.path;
                std::thread::sleep(std::time::Duration::from_millis(50 * (1 << attempt)));
            }
        }
    }
    unreachable!()
}
pub fn journal(p: &Path, v: &Value) -> Result<()> {
    private_dir(p.parent().unwrap())?;
    if let Ok(m) = fs::symlink_metadata(p) {
        if m.file_type().is_symlink() || !m.is_file() {
            bail!("journal must be regular file");
        }
    }
    let mut o = OpenOptions::new();
    o.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut f = o.open(p)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    serde_json::to_writer(&mut f, v)?;
    f.write_all(b"\n")?;
    Ok(())
}
pub struct Lock(PathBuf);
impl Lock {
    pub fn acquire(dir: &Path, name: &str) -> Result<Self> {
        private_dir(dir)?;
        let p = dir.join(name);
        let mut o = OpenOptions::new();
        o.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            o.mode(0o600);
        }
        let mut f = o.open(&p).map_err(|_| {
            anyhow::anyhow!("{name} exists or cannot be created; inspect PID before recovery")
        })?;
        let guard = Self(p);
        serde_json::to_writer(
            &mut f,
            &serde_json::json!({"pid":std::process::id(),"created_at":now(),"supervisor_token":std::env::var("HUW_SUPERVISOR_TOKEN").ok()}),
        )?;
        f.sync_all()?;
        Ok(guard)
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
