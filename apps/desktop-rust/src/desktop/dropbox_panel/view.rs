use iced::{
    Alignment, Element, Fill, Shrink, border,
    widget::{Column, button, checkbox, column, container, row, scrollable, text, tooltip},
};

use crate::appearance::Surfaces;

use super::{
    Action, Control, Message, Settings,
    focus::{FOLDERS_SCROLL, SELECTION_SCROLL},
    folder_name, parse_interval,
};
use crate::desktop::view::{
    Icon, action, icon_tinted, panel, primary, settings_button, settings_input,
};

const FOLDER_LIST_HEIGHT: f32 = 240.0;
const SELECTION_LIST_HEIGHT: f32 = 112.0;
const FOLDER_SCROLLBAR_GAP: f32 = 6.0;
const ACTION_GAP: f32 = 8.0;
const FOLDER_ICON_EDGE: f32 = 15.0;

impl Settings {
    #[cfg(test)]
    pub fn view(&self, surfaces: Surfaces) -> Column<'_, Message> {
        self.view_with_focus(surfaces, None)
    }

    pub fn view_with_focus(
        &self,
        surfaces: Surfaces,
        focus: Option<&Control>,
    ) -> Column<'_, Message> {
        let mut inner = column![].spacing(14);
        let connected = self.snapshot.account.is_some() && !self.snapshot.reconnect;
        if connected {
            inner = inner
                .push(text("Connected to Dropbox").size(13))
                .push(
                    row![
                        keyboard_button(
                            settings_button("Browse folders", surfaces)
                                .on_press_maybe(self.control_message(&Control::Browse)),
                            Control::Browse,
                            focus
                        ),
                        keyboard_button(
                            settings_button("Sync now", surfaces)
                                .on_press_maybe(self.control_message(&Control::Sync)),
                            Control::Sync,
                            focus
                        ),
                        keyboard_button(
                            quiet_button("Disconnect")
                                .on_press_maybe(self.control_message(&Control::Disconnect)),
                            Control::Disconnect,
                            focus
                        ),
                    ]
                    .spacing(ACTION_GAP)
                    .align_y(Alignment::Center)
                    .wrap(),
                )
                .push(self.sync_interval(surfaces, focus));
            if !self.snapshot.path.is_empty() {
                inner = inner.push(self.folder_browser(surfaces, focus));
            }
        }
        if self.auth.is_some() || !connected {
            inner = inner.push(self.authorization(surfaces, focus));
        }
        if self.busy {
            inner = inner.push(
                text(if self.connecting {
                    "Connecting…"
                } else {
                    "Updating Dropbox…"
                })
                .size(12)
                .color(surfaces.muted),
            );
        }
        if let Some(error) = &self.error {
            inner = inner.push(text(error).size(12));
        }
        inner.width(Fill)
    }

    fn authorization(&self, surfaces: Surfaces, focus: Option<&Control>) -> Column<'_, Message> {
        if self.auth.is_some() {
            let submit = self.control_message(&Control::Connect);
            return column![
                text("Approve access in your browser, then paste the authorization code here.")
                    .size(13),
                text("Authorization code").size(12).color(surfaces.muted),
                settings_input(&self.code, surfaces)
                    .id(Control::Code.id())
                    .secure(true)
                    .on_input_maybe(
                        (!self.busy).then_some(|value| Message::Dropbox(Action::Code(value)))
                    )
                    .on_submit_maybe(submit.clone()),
                row![
                    keyboard_button(
                        accent_button("Finish connecting").on_press_maybe(submit),
                        Control::Connect,
                        focus
                    ),
                    keyboard_button(
                        quiet_button("Restart authorization")
                            .on_press_maybe(self.control_message(&Control::Restart)),
                        Control::Restart,
                        focus
                    ),
                    keyboard_button(
                        quiet_button("Cancel")
                            .on_press_maybe(self.control_message(&Control::Cancel)),
                        Control::Cancel,
                        focus
                    ),
                ]
                .spacing(ACTION_GAP)
                .wrap(),
            ]
            .spacing(ACTION_GAP);
        }
        let submit = self.control_message(&Control::Connect);
        column![
            text("Dropbox app key").size(12).color(surfaces.muted),
            settings_input(&self.app_key, surfaces)
                .id(Control::AppKey.id())
                .on_input_maybe(
                    (!self.busy).then_some(|value| Message::Dropbox(Action::Key(value)))
                )
                .on_submit_maybe(submit.clone()),
            keyboard_button(
                accent_button(if self.snapshot.reconnect {
                    "Reconnect Dropbox"
                } else {
                    "Connect Dropbox"
                })
                .on_press_maybe(submit),
                Control::Connect,
                focus
            ),
        ]
        .spacing(ACTION_GAP)
    }

    fn sync_interval(&self, surfaces: Surfaces, focus: Option<&Control>) -> Column<'_, Message> {
        let minutes = parse_interval(&self.interval);
        let save = self.control_message(&Control::SaveInterval);
        let mut controls = column![
            text("Sync interval").size(12).color(surfaces.muted),
            row![
                settings_input(&self.interval, surfaces)
                    .id(Control::Interval.id())
                    .on_input_maybe(
                        (!self.busy).then_some(|value| Message::Dropbox(Action::Interval(value)))
                    )
                    .on_submit_maybe(save.clone())
                    .width(58),
                text("minutes").size(12).color(surfaces.muted),
                keyboard_button(
                    settings_button("Save", surfaces).on_press_maybe(save),
                    Control::SaveInterval,
                    focus
                ),
            ]
            .spacing(ACTION_GAP)
            .align_y(Alignment::Center),
        ]
        .spacing(5);
        if let Err(error) = minutes {
            controls = controls.push(text(error.to_string()).size(12));
        }
        controls
    }

    fn folder_browser(&self, surfaces: Surfaces, focus: Option<&Control>) -> Element<'_, Message> {
        let path = &self.snapshot.path;
        let breadcrumb = if path == "/" {
            "Dropbox".into()
        } else {
            let folders = path.trim_start_matches('/').replace('/', " / ");
            format!("Dropbox / {folders}")
        };
        let mut heading = row![
            text(breadcrumb)
                .size(12)
                .width(Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
        ]
        .spacing(ACTION_GAP);
        if path != "/" {
            heading = heading.push(keyboard_button(
                quiet_button("Parent folder")
                    .on_press_maybe(self.control_message(&Control::Parent)),
                Control::Parent,
                focus,
            ));
        }
        let heading = heading.align_y(Alignment::Center);
        let mut browser = column![
            heading,
            self.folder_checkbox(
                path,
                &folder_name(path),
                Some("Select this folder"),
                Control::SelectCurrent,
                focus
            ),
        ]
        .spacing(ACTION_GAP);
        if self.snapshot.folders.is_empty() {
            browser = browser.push(text("No subfolders").size(12).color(surfaces.muted));
        } else {
            let rows = self
                .snapshot
                .folders
                .iter()
                .map(|(path, name)| self.folder_row(path, name, focus));
            browser = browser.push(
                container(
                    scrollable(column(rows).spacing(2))
                        .id(FOLDERS_SCROLL)
                        .spacing(FOLDER_SCROLLBAR_GAP)
                        .height(Shrink),
                )
                .max_height(FOLDER_LIST_HEIGHT)
                .width(Fill),
            );
        }
        let count = self.selected_folders.len();
        if count > 0 {
            let rows = self.selected_folders.iter().map(|(selected, name)| {
                let label = self
                    .snapshot
                    .failed_folders
                    .iter()
                    .find(|(folder, _)| folder.eq_ignore_ascii_case(selected))
                    .map_or_else(
                        || selected.clone(),
                        |(_, error)| format!("{selected}: {error}"),
                    );
                row![
                    text(label)
                        .size(12)
                        .width(Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                    keyboard_button(
                        quiet_button("Remove").on_press_maybe(self.folder_message(selected, name)),
                        Control::RemoveSelected(selected.clone()),
                        focus
                    ),
                ]
                .spacing(ACTION_GAP)
                .align_y(Alignment::Center)
                .into()
            });
            browser = browser
                .push(text("Selected folders").size(12).color(surfaces.muted))
                .push(
                    container(
                        scrollable(column(rows).spacing(2))
                            .id(SELECTION_SCROLL)
                            .spacing(FOLDER_SCROLLBAR_GAP)
                            .height(Shrink),
                    )
                    .max_height(SELECTION_LIST_HEIGHT)
                    .width(Fill),
                );
            let label = if count == 1 {
                "Add 1 folder".into()
            } else {
                format!("Add {count} folders")
            };
            browser = browser.push(keyboard_button(
                accent_button(label).on_press_maybe(self.control_message(&Control::AddSelected)),
                Control::AddSelected,
                focus,
            ));
        }
        container(browser)
            .width(Fill)
            .padding(10)
            .style(move |_| {
                container::Style::default()
                    .background(surfaces.raised)
                    .border(border::rounded(8).width(1).color(surfaces.line))
            })
            .into()
    }

    fn folder_row<'a>(
        &self,
        path: &str,
        name: &'a str,
        focus: Option<&Control>,
    ) -> Element<'a, Message> {
        let glyph = icon_tinted(Icon::Folder, |theme| theme.palette().text.scale_alpha(0.65))
            .width(FOLDER_ICON_EDGE)
            .height(FOLDER_ICON_EDGE);
        let control = Control::OpenFolder(path.into());
        let open = button(
            row![
                glyph,
                text(name)
                    .size(13)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .width(Fill),
                text("›").size(16)
            ]
            .spacing(ACTION_GAP)
            .align_y(Alignment::Center),
        )
        .width(Fill)
        .padding([5, 8])
        .style(action)
        .on_press_maybe(self.control_message(&control));
        row![
            self.folder_checkbox(path, name, None, Control::SelectFolder(path.into()), focus),
            keyboard_button(open, control, focus)
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    }

    fn folder_checkbox(
        &self,
        path: &str,
        name: &str,
        label: Option<&'static str>,
        target: Control,
        focus: Option<&Control>,
    ) -> Element<'static, Message> {
        let added = self
            .snapshot
            .added_paths
            .iter()
            .any(|added| added.eq_ignore_ascii_case(path));
        let selected = self
            .selected_folders
            .iter()
            .any(|(item, _)| item.eq_ignore_ascii_case(path));
        let command = self.folder_message(path, name);
        let mut control = checkbox(added || selected)
            .size(15)
            .text_size(12)
            .on_toggle_maybe(command.map(|message| move |_| message.clone()));
        if focus == Some(&target) {
            control = control.style(|theme, status| {
                let style = checkbox::primary(theme, status);
                checkbox::Style {
                    background: theme.palette().primary.scale_alpha(0.21).into(),
                    ..style
                }
            });
        }
        if let Some(label) = label {
            control = control.label(if added { "Already added" } else { label });
            return container(control).id(target.id()).into();
        }
        let hint = if added {
            format!("{name} is already added")
        } else if selected {
            format!("Deselect {name}")
        } else {
            format!("Select {name}")
        };
        container(
            tooltip(control, text(hint).size(11), tooltip::Position::Right)
                .gap(6)
                .style(panel),
        )
        .id(target.id())
        .into()
    }
}

fn keyboard_button<'a>(
    button: button::Button<'a, Message>,
    target: Control,
    focus: Option<&Control>,
) -> Element<'a, Message> {
    let button = if focus == Some(&target) {
        button.style(|theme, status| {
            let style = action(theme, status);
            button::Style {
                background: Some(theme.palette().primary.scale_alpha(0.19).into()),
                border: border::rounded(5),
                ..style
            }
        })
    } else {
        button
    };
    container(button).id(target.id()).into()
}

fn quiet_button(label: &str) -> button::Button<'_, Message> {
    button(text(label).size(13)).padding([5, 9]).style(action)
}

fn accent_button<'a>(label: impl Into<String>) -> button::Button<'a, Message> {
    button(text(label.into()).size(13))
        .padding([6, 10])
        .style(|theme, status| {
            if status == button::Status::Disabled {
                action(theme, status)
            } else {
                primary(theme, status)
            }
        })
}
