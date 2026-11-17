//! Feature modules: the extension point of the desktop application.
//!
//! A module is one page of the workspace. Implement [`FeatureModule`], add the type to
//! [`registry`], and add a [`Page`] variant. Navigation, the command palette and the
//! window title pick it up automatically. See `docs/extending-the-application.md`.

use crate::cx::Cx;
use crate::pages;
use app_core::Page;
use ui_components::Icon;

pub trait FeatureModule {
    fn page(&self) -> Page;
    fn icon(&self) -> Icon;
    fn subtitle(&self) -> &'static str;
    /// Called every time the user navigates to the module (refresh cached data here).
    fn on_enter(&mut self, _cx: &mut Cx<'_>) {}
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>);
}

/// Every module of the application, in sidebar order.
pub fn registry(start_dir: std::path::PathBuf) -> Vec<Box<dyn FeatureModule>> {
    vec![
        Box::new(pages::welcome::Welcome),
        Box::new(pages::dashboard::Dashboard::default()),
        Box::new(pages::workspace::Workspace::default()),
        Box::new(pages::files::Files::new(start_dir)),
        Box::new(pages::data::DataTable::default()),
        Box::new(pages::tasks::Tasks),
        Box::new(pages::logs::Logs::default()),
        Box::new(pages::settings::Settings::default()),
        Box::new(pages::appearance::AppearancePage),
        Box::new(pages::about::About),
        Box::new(pages::recovery::Recovery::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_has_exactly_one_module() {
        let reg = registry(std::env::temp_dir());
        for p in Page::ALL {
            assert_eq!(reg.iter().filter(|m| m.page() == p).count(), 1, "{p:?}");
        }
    }
}
