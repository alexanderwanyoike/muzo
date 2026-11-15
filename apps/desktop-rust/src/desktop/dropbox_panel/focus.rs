use iced::advanced::widget::Id;

use super::{Action, Message, Settings, folder_name, parse_interval};

pub(super) const FOLDERS_SCROLL: &str = "dropbox-folders";
pub(super) const SELECTION_SCROLL: &str = "dropbox-selection";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::desktop) enum Control {
    Browse,
    Sync,
    Disconnect,
    Interval,
    SaveInterval,
    AppKey,
    Code,
    Connect,
    Restart,
    Cancel,
    Parent,
    SelectCurrent,
    SelectFolder(String),
    OpenFolder(String),
    RemoveSelected(String),
    AddSelected,
}

impl Control {
    pub(in crate::desktop) fn id(&self) -> Id {
        Id::from(format!("dropbox-settings-{self:?}"))
    }

    pub(in crate::desktop) fn is_field(&self) -> bool {
        matches!(self, Self::Interval | Self::AppKey | Self::Code)
    }

    pub(in crate::desktop) fn focused_field(id: &Id) -> Option<Self> {
        [Self::Interval, Self::AppKey, Self::Code]
            .into_iter()
            .find(|control| control.id() == *id)
    }

    pub(in crate::desktop) fn nested_scroll(&self) -> Option<Id> {
        match self {
            Self::SelectFolder(_) | Self::OpenFolder(_) => Some(FOLDERS_SCROLL.into()),
            Self::RemoveSelected(_) => Some(SELECTION_SCROLL.into()),
            _ => None,
        }
    }

    fn action(&self, settings: &Settings) -> Option<Action> {
        Some(match self {
            Self::Browse => Action::Browse("/".into()),
            Self::Sync => Action::Sync,
            Self::Disconnect => Action::Disconnect,
            Self::SaveInterval => {
                let minutes = parse_interval(&settings.interval).ok()?;
                (minutes != settings.snapshot.interval).then_some(Action::SaveInterval)?
            }
            Self::Connect if settings.auth.is_some() => {
                (!settings.code.trim().is_empty()).then_some(Action::Connect)?
            }
            Self::Connect | Self::Restart => {
                (!settings.app_key.trim().is_empty()).then_some(Action::Begin)?
            }
            Self::Cancel => Action::CancelAuthorization,
            Self::Parent => {
                let (parent, _) = settings.snapshot.path.rsplit_once('/')?;
                Action::Browse(if parent.is_empty() { "/" } else { parent }.into())
            }
            Self::SelectCurrent => {
                let path = &settings.snapshot.path;
                settings.toggle_folder(path, &folder_name(path))?
            }
            Self::SelectFolder(path) => {
                let (_, name) = settings
                    .snapshot
                    .folders
                    .iter()
                    .find(|(item, _)| item == path)?;
                settings.toggle_folder(path, name)?
            }
            Self::OpenFolder(path) => Action::Browse(path.clone()),
            Self::RemoveSelected(path) => {
                let (_, name) = settings
                    .selected_folders
                    .iter()
                    .find(|(item, _)| item == path)?;
                settings.toggle_folder(path, name)?
            }
            Self::AddSelected => Action::AddSelected,
            Self::Interval | Self::AppKey | Self::Code => return None,
        })
    }
}

impl Settings {
    pub(in crate::desktop) fn focus_stops(&self) -> Vec<Control> {
        use Control::*;

        if self.busy {
            return Vec::new();
        }
        let connected = self.snapshot.account.is_some() && !self.snapshot.reconnect;
        let mut controls = Vec::new();
        if connected {
            controls.extend([Browse, Sync, Disconnect, Interval, SaveInterval]);
            if !self.snapshot.path.is_empty() {
                if self.snapshot.path != "/" {
                    controls.push(Parent);
                }
                controls.push(SelectCurrent);
                for (path, _) in &self.snapshot.folders {
                    if self.folder_is_selectable(path) {
                        controls.push(SelectFolder(path.clone()));
                    }
                    controls.push(OpenFolder(path.clone()));
                }
                controls.extend(
                    self.selected_folders
                        .iter()
                        .filter(|(path, _)| self.folder_is_selectable(path))
                        .map(|(path, _)| RemoveSelected(path.clone())),
                );
                if !self.selected_folders.is_empty() {
                    controls.push(AddSelected);
                }
            }
        }
        if self.auth.is_some() {
            controls.extend([Code, Connect, Restart, Cancel]);
        } else if !connected {
            controls.extend([AppKey, Connect]);
        }
        controls.retain(|control| {
            control.is_field()
                || matches!(control, SelectFolder(_) | OpenFolder(_) | RemoveSelected(_))
                || self.control_message(control).is_some()
        });
        controls
    }

    pub(in crate::desktop) fn control_message(&self, control: &Control) -> Option<Message> {
        if self.busy {
            return None;
        }
        Some(Message::Dropbox(control.action(self)?))
    }

    pub(super) fn folder_message(&self, path: &str, name: &str) -> Option<Message> {
        if self.busy {
            return None;
        }
        self.toggle_folder(path, name).map(Message::Dropbox)
    }

    fn folder_is_selectable(&self, path: &str) -> bool {
        !self
            .snapshot
            .added_paths
            .iter()
            .any(|added| added.eq_ignore_ascii_case(path))
    }

    fn toggle_folder(&self, path: &str, label: &str) -> Option<Action> {
        self.folder_is_selectable(path)
            .then(|| Action::SelectFolder(path.into(), label.into()))
    }
}
