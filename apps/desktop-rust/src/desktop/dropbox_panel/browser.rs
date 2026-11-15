use std::io;

#[cfg(target_os = "linux")]
pub(super) fn open_default(url: &str) -> io::Result<()> {
    use std::process::{Command, Stdio};

    match Command::new("gio")
        .args(["open", url])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(io::Error::other(format!(
            "The desktop link handler exited with {status}."
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => open::that_detached(url),
        Err(error) => Err(error),
    }
}

#[cfg(not(target_os = "linux"))]
pub(super) fn open_default(url: &str) -> io::Result<()> {
    open::that_detached(url)
}
