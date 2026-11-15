use std::io;

#[cfg(target_os = "linux")]
pub(super) fn open_default(url: &str) -> io::Result<()> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    // KDE and Xfce can keep their preferred browser separately from MIME associations.
    if desktop
        .split(':')
        .any(|name| name.eq_ignore_ascii_case("kde") || name.eq_ignore_ascii_case("xfce"))
    {
        return launch("xdg-open", &[url]).or_else(|_| launch("gio", &["open", url]));
    }

    match launch("gio", &["open", url]) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => launch("xdg-open", &[url]),
        result => result,
    }
}

#[cfg(target_os = "linux")]
fn launch(executable: &str, args: &[&str]) -> io::Result<()> {
    use std::process::{Command, Stdio};

    let status = Command::new(executable)
        .args(args)
        .env_remove("BROWSER")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "The desktop link handler exited with {status}."
        )))
    }
}

#[cfg(not(target_os = "linux"))]
pub(super) fn open_default(url: &str) -> io::Result<()> {
    open::that_detached(url)
}
