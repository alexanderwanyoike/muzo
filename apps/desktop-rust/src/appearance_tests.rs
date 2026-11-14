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
