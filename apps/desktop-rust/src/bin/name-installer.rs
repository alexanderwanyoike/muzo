use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha512};
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use muzo_desktop::distribution::{Target, filename};

fn main() -> io::Result<()> {
    let target = Target::current()
        .ok_or_else(|| io::Error::other("Unsupported Muzo distribution target"))?;
    let argument = std::env::args_os()
        .nth(1)
        .ok_or_else(|| io::Error::other("Pass the package output directory"))?;
    if argument == "--target-id" {
        println!("{}", target.id());
        return Ok(());
    }
    let directory = PathBuf::from(argument);
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
    if matches!(target, Target::LinuxX64 | Target::WindowsX64) {
        let mut source = fs::File::open(&destination)?;
        let mut sha = Sha512::new();
        let mut bytes = [0u8; 32 * 1024];
        let mut count = 0;
        loop {
            let read = source.read(&mut bytes)?;
            if read == 0 {
                break;
            }
            sha.update(&bytes[..read]);
            count += read;
        }
        let version = env!("CARGO_PKG_VERSION");
        let digest = STANDARD.encode(sha.finalize());
        let name = filename(version, target);
        let metadata_name = if matches!(target, Target::LinuxX64) {
            "latest-linux.yml"
        } else {
            "latest.yml"
        };
        fs::write(
            directory.join(metadata_name),
            format!(
                "version: {version}\npath: {name}\nsha512: {digest}\nfiles:\n  - url: {name}\n    sha512: {digest}\n    size: {count}\n"
            ),
        )?;
    }
    println!("{}", destination.display());
    Ok(())
}
