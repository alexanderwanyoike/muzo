use iced::{
    Alignment, Element, Fill, Shrink, border,
    widget::{Column, button, checkbox, column, container, row, scrollable, text, tooltip},
};

use crate::appearance::Surfaces;

use super::{Action, Error, Message, Settings, folder_name, parse_interval};
use crate::desktop::view::{
    Icon, action, icon_tinted, panel, primary, settings_button, settings_input,
};

const FOLDER_LIST_HEIGHT: f32 = 240.0;
const ACTION_GAP: f32 = 8.0;
const FOLDER_ICON_EDGE: f32 = 15.0;

impl Settings {
    pub fn view(&self, surfaces: Surfaces) -> Column<'_, Message> {
        let mut inner = column![].spacing(14);
        if self.snapshot.account.is_some() && !self.snapshot.reconnect {
            inner = inner
                .push(text("Connected to Dropbox").size(13))
                .push(
                    row![
                        settings_button("Browse folders", surfaces)
                            .on_press_maybe(self.command(Action::Browse("/".into()))),
                        settings_button("Sync now", surfaces)
                            .on_press_maybe(self.command(Action::Sync)),
                        quiet_button("Disconnect").on_press_maybe(self.command(Action::Disconnect)),
                    ]
                    .spacing(ACTION_GAP)
                    .align_y(Alignment::Center)
                    .wrap(),
                )
                .push(self.sync_interval(surfaces));
            if !self.snapshot.path.is_empty() {
                inner = inner.push(self.folder_browser(surfaces));
            }
        } else {
            inner = inner.push(self.authorization(surfaces));
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

    fn authorization(&self, surfaces: Surfaces) -> Column<'_, Message> {
        if self.auth.is_some() {
            let submit = (!self.code.trim().is_empty())
                .then(|| self.command(Action::Connect))
                .flatten();
            return column![
                text("Approve access in your browser, then paste the authorization code here.")
                    .size(13),
                text("Authorization code").size(12).color(surfaces.muted),
                settings_input(&self.code, surfaces)
                    .secure(true)
                    .on_input_maybe(
                        (!self.busy).then_some(|value| Message::Dropbox(Action::Code(value)))
                    )
                    .on_submit_maybe(submit.clone()),
                row![
                    accent_button("Finish connecting").on_press_maybe(submit),
                    quiet_button("Cancel")
                        .on_press_maybe(self.command(Action::CancelAuthorization)),
                ]
                .spacing(ACTION_GAP)
                .wrap(),
            ]
            .spacing(ACTION_GAP);
        }
        let submit = (!self.app_key.trim().is_empty())
            .then(|| self.command(Action::Begin))
            .flatten();
        column![
            text("Dropbox app key").size(12).color(surfaces.muted),
            settings_input(&self.app_key, surfaces)
                .on_input_maybe(
                    (!self.busy).then_some(|value| Message::Dropbox(Action::Key(value)))
                )
                .on_submit_maybe(submit.clone()),
            accent_button(if self.snapshot.reconnect {
                "Reconnect Dropbox"
            } else {
                "Connect Dropbox"
            })
            .on_press_maybe(submit),
        ]
        .spacing(ACTION_GAP)
    }

    fn sync_interval(&self, surfaces: Surfaces) -> Column<'_, Message> {
        let minutes = parse_interval(&self.interval);
        let save = minutes
            .filter(|minutes| *minutes != self.snapshot.interval)
            .and_then(|_| self.command(Action::SaveInterval));
        let mut controls = column![
            text("Sync interval").size(12).color(surfaces.muted),
            row![
                settings_input(&self.interval, surfaces)
                    .on_input_maybe(
                        (!self.busy).then_some(|value| Message::Dropbox(Action::Interval(value)))
                    )
                    .on_submit_maybe(save.clone())
                    .width(58),
                text("minutes").size(12).color(surfaces.muted),
                settings_button("Save", surfaces).on_press_maybe(save),
            ]
            .spacing(ACTION_GAP)
            .align_y(Alignment::Center),
        ]
        .spacing(5);
        if minutes.is_none() {
            controls = controls.push(text(Error::InvalidInterval.to_string()).size(12));
        }
        controls
    }

    fn folder_browser(&self, surfaces: Surfaces) -> Element<'_, Message> {
        let path = &self.snapshot.path;
        let breadcrumb = if path == "/" {
            "Dropbox".into()
        } else {
            let folders = path.trim_start_matches('/').replace('/', " / ");
            format!("Dropbox / {folders}")
        };
        let mut heading = row![text(breadcrumb).size(12).width(Fill),].spacing(ACTION_GAP);
        if path != "/" {
            let (parent, _) = path.rsplit_once('/').unwrap_or(("/", ""));
            let parent = if parent.is_empty() { "/" } else { parent };
            heading = heading.push(
                quiet_button("Parent folder")
                    .on_press_maybe(self.command(Action::Browse(parent.into()))),
            );
        }
        let heading = heading.align_y(Alignment::Center);
        let mut browser = column![
            heading,
            self.folder_checkbox(path, &folder_name(path), Some("Select this folder")),
        ]
        .spacing(ACTION_GAP);
        if self.snapshot.folders.is_empty() {
            browser = browser.push(text("No subfolders").size(12).color(surfaces.muted));
        } else {
            let rows = self
                .snapshot
                .folders
                .iter()
                .map(|(path, name)| self.folder_row(path, name));
            browser = browser.push(
                container(scrollable(column(rows).spacing(2)).height(Shrink))
                    .max_height(FOLDER_LIST_HEIGHT)
                    .width(Fill),
            );
        }
        let selected_elsewhere = self.selected_folders.iter().filter(|(selected, _)| {
            !path.eq_ignore_ascii_case(selected)
                && !self
                    .snapshot
                    .folders
                    .iter()
                    .any(|(visible, _)| visible.eq_ignore_ascii_case(selected))
        });
        for (selected, name) in selected_elsewhere {
            browser = browser.push(
                row![
                    text(selected).size(12).width(Fill),
                    quiet_button("Remove").on_press_maybe(
                        self.command(Action::SelectFolder(selected.clone(), name.clone()))
                    ),
                ]
                .spacing(ACTION_GAP)
                .align_y(Alignment::Center),
            );
        }
        let count = self.selected_folders.len();
        if count > 0 {
            let label = if count == 1 {
                "Add 1 folder".into()
            } else {
                format!("Add {count} folders")
            };
            browser = browser
                .push(accent_button(label).on_press_maybe(self.command(Action::AddSelected)));
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

    fn folder_row<'a>(&self, path: &str, name: &'a str) -> Element<'a, Message> {
        let glyph = icon_tinted(Icon::Folder, |theme| theme.palette().text.scale_alpha(0.65))
            .width(FOLDER_ICON_EDGE)
            .height(FOLDER_ICON_EDGE);
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
        .on_press_maybe(self.command(Action::Browse(path.into())));
        row![self.folder_checkbox(path, name, None), open]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
    }

    fn folder_checkbox(
        &self,
        path: &str,
        name: &str,
        label: Option<&'static str>,
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
        let command = Action::SelectFolder(path.to_owned(), name.to_owned());
        let mut control = checkbox(added || selected)
            .size(15)
            .text_size(12)
            .on_toggle_maybe(
                (!self.busy && !added).then_some(move |_| Message::Dropbox(command.clone())),
            );
        if let Some(label) = label {
            control = control.label(if added { "Already added" } else { label });
            return control.into();
        }
        let hint = if added {
            format!("{name} is already added")
        } else if selected {
            format!("Deselect {name}")
        } else {
            format!("Select {name}")
        };
        tooltip(control, text(hint).size(11), tooltip::Position::Right)
            .gap(6)
            .style(panel)
            .into()
    }

    fn command(&self, action: Action) -> Option<Message> {
        (!self.busy).then_some(Message::Dropbox(action))
    }
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
