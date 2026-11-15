use std::io;

pub(super) async fn open_default(url: String) -> io::Result<()> {
    let (result, opened) = iced::futures::channel::oneshot::channel();
    // A desktop handler can wait for the browser to close. Keep it out of Iced's shutdown path.
    std::thread::Builder::new()
        .name("dropbox-browser".into())
        .spawn(move || {
            let _ = result.send(launch_default(&url));
        })?;
    opened.await.map_err(io::Error::other)?
}

#[cfg(target_os = "linux")]
fn launch_default(url: &str) -> io::Result<()> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let kde = desktop
        .split(':')
        .any(|name| name.eq_ignore_ascii_case("kde"));
    if kde && std::env::var_os("KDE_SESSION_VERSION").is_some_and(|version| version == "6") {
        // Older xdg-open dispatchers can exit successfully without opening anything on Plasma 6.
        return launch("kde-open", &[url]).or_else(|_| launch("gio", &["open", url]));
    }
    // KDE and Xfce can keep their preferred browser separately from MIME associations.
    if kde
        || desktop
            .split(':')
            .any(|name| name.eq_ignore_ascii_case("xfce"))
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
fn launch_default(url: &str) -> io::Result<()> {
    open::that_detached(url)
}
