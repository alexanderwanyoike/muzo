use iced::Size;

use crate::appearance::sidebar::SidebarWidth;

use super::{
    cover_layout_tests::paint,
    queue_tests::{setup, snapshot},
};

#[test]
fn it_draws_the_source_selection_across_the_whole_row_and_reserves_its_accent_edge() {
    const FRAME: Size<u32> = Size::new(850, 800);
    const FIRST_SOURCE_Y: usize = 380;
    const SECOND_SOURCE_Y: usize = FIRST_SOURCE_Y + 32;
    const ACCENT: [u8; 4] = [0, 217, 237, 255];

    let (mut desktop, _) = setup();
    let mut catalogue = snapshot(&["Selected library", "Other library"]);
    catalogue.selected = Some(catalogue.libraries[0].id.clone());
    desktop.sources.snapshot = Some(catalogue);
    desktop.sidebar.set(SidebarWidth::new(312.0));
    desktop.sidebar.layout_width(FRAME.width as f32);
    desktop.appearance.accent = "glow-cyan".into();

    for (material, mode, selected) in [
        ("walnut", "dark", [30, 53, 51, 255]),
        ("pulse", "light", [216, 216, 216, 255]),
        ("pulse", "dark", [42, 42, 42, 255]),
    ] {
        desktop.appearance.theme = material.into();
        desktop.appearance.mode = mode.into();
        let panel = desktop.appearance.surfaces().raised.into_rgba8();
        let pixels = paint(&desktop, desktop.windows.library_id(), FRAME);
        let at = |x, y| {
            let start = (y * FRAME.width as usize + x) * 4;
            &pixels[start..start + 4]
        };

        for x in [9, 10] {
            assert_eq!(
                at(x, FIRST_SOURCE_Y),
                ACCENT,
                "the selected {material} row's 2px accent edge"
            );
            assert_eq!(
                at(x, SECOND_SOURCE_Y),
                panel,
                "the unselected {material} row reserves an empty edge"
            );
        }
        for x in [11, 150, 300] {
            assert_eq!(
                at(x, FIRST_SOURCE_Y),
                selected,
                "the {material} selection at x={x}"
            );
        }
    }
}
