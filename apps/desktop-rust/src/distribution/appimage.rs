use std::{
    fs,
    io::{self, Write},
    path::Path,
};

const MANAGED: &str = "X-Muzo-AppImage-Integration=true";
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
        let lines = contents.lines().map(str::trim);
        let temporary_cinnamon = entry.file_name() == CINNAMON
            && lines.clone().any(|line| line == "Name=Muzo")
            && lines.clone().any(|line| {
                line.strip_prefix("Exec=/tmp/.mount_").is_some_and(|exec| {
                    exec.split_whitespace()
                        .next()
                        .is_some_and(|binary| binary.ends_with("/muzo"))
                })
            });
        if temporary_cinnamon || lines.clone().any(|line| line == MANAGED) {
            managed = Some(path);
        } else if entry.file_name() == SHORTCUT
            || lines.clone().any(|line| line == "StartupWMClass=muzo")
        {
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
        "[Desktop Entry]\nType=Application\nName=Muzo\nExec={command}\"{escaped}\"\nIcon={icon}\nTerminal=false\nStartupWMClass=muzo\nCategories=AudioVideo;Audio;Player;\n{MANAGED}\n"
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
}
