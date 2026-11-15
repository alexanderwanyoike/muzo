fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        const ICON: &str = "assets/icons/muzo.ico";
        println!("cargo:rerun-if-changed={ICON}");
        winresource::WindowsResource::new()
            .set_icon(ICON)
            .compile_for("muzo-desktop")?;
    }
    Ok(())
}
