use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::Path,
};

const MANAGED: &str = "X-Muzo-AppImage-Integration";
const SHORTCUT: &str = "muzo.desktop";
const CINNAMON: &str = "Muzo.cinnamon-generated.desktop";

pub fn register(image: &Path, data: &Path) -> io::Result<()> {
    let image = desktop_path(image)?;
    desktop_path(data)?;
    if image.contains('=') {
        return Err(io::Error::other(
            "Cannot use an AppImage path containing '=' in a desktop launcher.",
        ));
    }
    let directory = data.join("applications");
    fs::create_dir_all(&directory)?;
    let mut managed = None;
    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| extension != "desktop")
        {
            continue;
        }
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if entry.file_name() == SHORTCUT || entry.file_name() == CINNAMON => {
                return Err(error);
            }
            Err(_) => continue,
        };
        let fields = desktop_entry_fields(&contents);
        let temporary_cinnamon = entry.file_name() == CINNAMON
            && fields.get("Name") == Some(&"Muzo")
            && fields.get("Exec").is_some_and(|exec| {
                exec.strip_prefix("/tmp/.mount_").is_some_and(|exec| {
                    exec.split_whitespace()
                        .next()
                        .is_some_and(|binary| binary.ends_with("/muzo"))
                })
            });
        if temporary_cinnamon || fields.get(MANAGED) == Some(&"true") {
            managed = Some(path);
        } else if entry.file_name() == SHORTCUT || fields.get("StartupWMClass") == Some(&"muzo") {
            return Ok(());
        }
    }

    let icon = data.join("muzo/appimage.png");
    fs::create_dir_all(data.join("muzo"))?;
    fs::write(
        &icon,
        include_bytes!("../../../desktop/assets/icons/muzo.png"),
    )?;
    let icon = desktop_path(&icon)?.replace('\\', "\\\\");
    let escaped = image
        .chars()
        .map(|character| match character {
            '%' => "%%".into(),
            '\\' => "\\\\\\\\".into(),
            '"' | '`' | '$' => format!("\\\\{character}"),
            _ => character.to_string(),
        })
        .collect::<String>();
    let command = if image.contains('%') { "env " } else { "" };
    let mut file = tempfile::NamedTempFile::new_in(&directory)?;
    write!(
        file,
        "[Desktop Entry]\nType=Application\nName=Muzo\nExec={command}\"{escaped}\"\nIcon={icon}\nTerminal=false\nStartupWMClass=muzo\nCategories=AudioVideo;Audio;Player;\n{MANAGED}=true\n"
    )?;
    match managed {
        Some(path) => {
            file.persist(path).map_err(|error| error.error)?;
        }
        None => {
            if let Err(error) = file.persist_noclobber(directory.join(SHORTCUT))
                && error.error.kind() != io::ErrorKind::AlreadyExists
            {
                return Err(error.error);
            }
        }
    }
    Ok(())
}

fn desktop_entry_fields(contents: &str) -> HashMap<&str, &str> {
    let mut fields = HashMap::new();
    let mut in_desktop_entry = false;
    for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
        } else if in_desktop_entry
            && !line.starts_with('#')
            && let Some((key, value)) = line.split_once('=')
        {
            fields.insert(key.trim(), value.trim());
        }
    }
    fields
}

fn desktop_path(path: &Path) -> io::Result<&str> {
    path.to_str()
        .filter(|text| path.is_absolute() && !text.chars().any(char::is_control))
        .ok_or_else(|| {
            io::Error::other(
                "Desktop launcher paths must be absolute text without control characters.",
            )
        })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    #[test]
    fn it_refreshes_a_managed_shortcut_and_preserves_external_launchers() {
        let root = tempfile::tempdir().unwrap();
        let image = root.path().join("Muzo 50%.AppImage");
        super::register(&image, root.path()).unwrap();
        let shortcut = root.path().join("applications/muzo.desktop");
        assert!(
            fs::read_to_string(&shortcut)
                .unwrap()
                .contains("50%%.AppImage")
        );
        let moved = root.path().join("New.AppImage");
        super::register(&moved, root.path()).unwrap();
        assert!(
            fs::read_to_string(&shortcut)
                .unwrap()
                .contains("New.AppImage")
        );

        fs::write(&shortcut, "StartupWMClass=muzo\n").unwrap();
        super::register(Path::new("/another.AppImage"), root.path()).unwrap();
        assert_eq!(
            fs::read_to_string(shortcut).unwrap(),
            "StartupWMClass=muzo\n"
        );
    }

    #[test]
    fn it_preserves_an_external_launcher_with_space_around_the_desktop_entry_key() {
        let root = tempfile::tempdir().unwrap();
        let applications = root.path().join("applications");
        fs::create_dir(&applications).unwrap();
        let external = applications.join("appimagekit-muzo.desktop");
        let contents = "[Desktop Entry]\nType=Application\nName=Muzo\nExec=/usr/bin/true\nStartupWMClass = muzo\n";
        fs::write(&external, contents).unwrap();

        super::register(Path::new("/portable/Muzo.AppImage"), root.path()).unwrap();

        assert_eq!(fs::read_to_string(external).unwrap(), contents);
        assert!(!applications.join("muzo.desktop").exists());
    }

    #[test]
    fn it_does_not_consider_actions_as_launcher_ownership() {
        let root = tempfile::tempdir().unwrap();
        let applications = root.path().join("applications");
        fs::create_dir(&applications).unwrap();
        let external = applications.join("another-player.desktop");
        let contents = "[Desktop Entry]\nName=Another player\n[Desktop Action Play]\nX-Muzo-AppImage-Integration=true\n";
        fs::write(&external, contents).unwrap();

        super::register(Path::new("/portable/Muzo.AppImage"), root.path()).unwrap();

        assert_eq!(fs::read_to_string(external).unwrap(), contents);
        assert!(applications.join("muzo.desktop").exists());
    }
}
