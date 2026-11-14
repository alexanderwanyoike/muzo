#![cfg(windows)]

use std::{ptr, slice};

use windows_sys::Win32::{
    Foundation::FreeLibrary,
    System::LibraryLoader::{
        FindResourceW, LOAD_LIBRARY_AS_DATAFILE, LoadLibraryExW, LoadResource, LockResource,
        SizeofResource,
    },
};

#[test]
fn it_embeds_small_and_large_shell_icons_in_the_application_binary() {
    let executable = env!("CARGO_BIN_EXE_muzo-desktop")
        .encode_utf16()
        .chain([0])
        .collect::<Vec<_>>();
    // Loading the PE as a data file reads its resources without running the app.
    unsafe {
        let module = LoadLibraryExW(
            executable.as_ptr(),
            ptr::null_mut(),
            LOAD_LIBRARY_AS_DATAFILE,
        );
        assert!(!module.is_null(), "could not read the desktop executable");
        let group = FindResourceW(
            module,
            ptr::without_provenance(1),
            ptr::without_provenance(14),
        );
        assert!(!group.is_null(), "the desktop executable has no icon group");
        let len = SizeofResource(module, group) as usize;
        let resource = LockResource(LoadResource(module, group));
        assert!(!resource.is_null() && len >= 6);
        let entries = slice::from_raw_parts(resource.cast::<u8>(), len)[6..].chunks_exact(14);
        let sizes: Vec<_> = entries.map(|entry| (entry[0], entry[1])).collect();
        FreeLibrary(module);
        assert!(sizes.contains(&(16, 16)));
        assert!(sizes.contains(&(32, 32)));
        assert!(sizes.contains(&(0, 0)), "256 px icon is missing");
    }
}
