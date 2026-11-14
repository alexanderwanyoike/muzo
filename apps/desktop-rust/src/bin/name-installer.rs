use std::{fs, io, path::PathBuf};

use muzo_desktop::distribution::{Target, filename};

fn main() -> io::Result<()> {
    let target = Target::current()
        .ok_or_else(|| io::Error::other("Unsupported Muzo distribution target"))?;
    let directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("Pass the package output directory"))?;
    let mut candidates = fs::read_dir(&directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case(target.extension()))
        });
    let source = candidates
        .next()
        .ok_or_else(|| io::Error::other("The installer was not found"))?;
    if candidates.next().is_some() {
        return Err(io::Error::other(
            "The directory contains multiple installers",
        ));
    }
    let destination = directory.join(filename(env!("CARGO_PKG_VERSION"), target));
    if source != destination {
        fs::rename(source, &destination)?;
    }
    println!("{}", destination.display());
    Ok(())
}
