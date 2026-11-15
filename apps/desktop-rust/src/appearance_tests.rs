use crate::appearance::{Appearance, load, save};

#[test]
fn it_restores_the_chosen_material_mode_and_accent() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("appearance.json");
    assert_eq!(load(&path).unwrap(), Appearance::default());
    let appearance = Appearance {
        theme: "walnut".into(),
        mode: "dark".into(),
        accent: "glow-cyan".into(),
    };
    save(&path, &appearance).unwrap();
    assert_eq!(load(&path).unwrap(), appearance);
    assert_ne!(
        appearance.theme().palette().background,
        Appearance::default().theme().palette().background
    );
}

#[test]
fn it_imports_the_electron_appearance_without_changing_its_storage() {
    let root = tempfile::tempdir().unwrap();
    let legacy = root.path().join("leveldb");
    let mut database = rusty_leveldb::DB::open(&legacy, rusty_leveldb::Options::default()).unwrap();
    let key = b"_app://muzo\0\x01muzo.appearance.v1";
    let json = b"\x01{\"theme\":\"paper\",\"mode\":\"dark\",\"accent\":\"plum\"}";
    database.put(key, json).unwrap();
    database.flush().unwrap();
    let before = database.get(key).unwrap();
    let path = root.path().join("appearance-rust.json");
    let imported = crate::appearance::load_or_import(&path, &legacy).unwrap();
    assert_eq!(imported.theme, "paper");
    assert_eq!(imported.mode, "dark");
    assert_eq!(load(&path).unwrap(), imported);
    assert_eq!(database.get(key).unwrap(), before);
}

#[test]
fn it_imports_a_valid_legacy_sidebar_width_and_keeps_the_rust_choice() {
    use crate::appearance::sidebar::{SidebarWidth, load_or_import, save};
    let root = tempfile::tempdir().unwrap();
    let legacy = root.path().join("leveldb");
    let mut database = rusty_leveldb::DB::open(&legacy, rusty_leveldb::Options::default()).unwrap();
    database
        .put(b"_app://muzo\0\x01muzo.sidebar-width.v1", b"\x01312")
        .unwrap();
    database.flush().unwrap();
    let path = root.path().join("sidebar-rust.json");

    assert_eq!(
        load_or_import(&path, Some(&legacy)).unwrap(),
        SidebarWidth::new(312.0)
    );
    save(&path, SidebarWidth::new(275.0)).unwrap();
    assert_eq!(
        load_or_import(&path, Some(&legacy)).unwrap(),
        SidebarWidth::new(275.0)
    );
}

#[test]
fn it_narrows_sidebar_on_a_small_window_without_losing_the_preferred_width() {
    use crate::appearance::sidebar::SidebarWidth;
    let width = SidebarWidth::new(310.0);

    assert_eq!(width.visible(500.0), 192.0);
    assert_eq!(width.visible(750.0), 230.0);
    assert_eq!(width.visible(900.0), 310.0);
}

#[test]
fn it_keeps_a_distinct_material_preview_for_each_textured_surface() {
    use crate::desktop::material;

    for material_id in ["aluminium", "carbon", "walnut"] {
        assert!(material::has_texture(material_id));
    }
    assert!(!material::has_texture("pulse"));
}
