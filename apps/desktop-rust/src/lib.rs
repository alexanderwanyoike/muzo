pub mod appearance;
pub mod artwork_cache;
pub mod catalogue;
pub mod desktop;
pub mod distribution;
pub mod dropbox;
pub mod fonts;
pub mod local_track;
mod opus;
pub mod playback;
pub mod session;
pub mod track;

#[cfg(test)]
mod playback_tests;

#[cfg(test)]
mod session_tests;

#[cfg(test)]
mod track_tests;

#[cfg(test)]
mod desktop_tests;

#[cfg(test)]
mod catalogue_tests;

#[cfg(test)]
mod dropbox_tests;

#[cfg(test)]
mod appearance_tests;

#[cfg(test)]
mod distribution_tests;
