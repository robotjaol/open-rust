//! Per-frame context handed to every feature module.

use app_core::{Action, AppContext, DialogPurpose, Notification, Page};
use app_errors::AppError;
use app_platform::dialogs::{self, DialogKind};
use std::collections::VecDeque;
use std::path::PathBuf;
use ui_components::Palette;

pub struct Cx<'a> {
    pub pal: Palette,
    pub app: &'a mut AppContext,
    pub nav: Vec<Action>,
    pub dialog_results: &'a mut Vec<(DialogPurpose, Option<PathBuf>)>,
    pub frame_ms: &'a VecDeque<f32>,
    /// Set when a module changed `app.config`; the shell persists it (debounced).
    pub config_dirty: bool,
    /// Set when a module wants the theme re-applied (appearance changed).
    pub appearance_dirty: bool,
}

impl Cx<'_> {
    pub fn notify(&self, n: Notification) {
        self.app.events.notify(n);
    }

    /// Report a failure: the internal text goes to the log, the user sees the safe message.
    pub fn fail(&self, what: &str, e: &AppError) {
        tracing::warn!(error = %e, code = e.code(), "{what}");
        self.notify(Notification::error(what.to_string(), e.user_message()));
    }

    pub fn goto(&mut self, page: Page) {
        self.nav.push(Action::Navigate(page));
    }

    /// Open a native dialog on a worker thread; the answer returns as [`DialogPurpose`] data.
    pub fn open_dialog(
        &self,
        purpose: DialogPurpose,
        kind: DialogKind,
        title: &str,
        default_name: &str,
    ) {
        if !dialogs::available() {
            self.notify(Notification::warning(
                "Dialogs unavailable",
                "This build has no native file dialogs.",
            ));
            return;
        }
        let bus = self.app.events.clone();
        dialogs::spawn(kind, title, default_name, move |path| {
            bus.send(app_core::AppEvent::DialogResult { purpose, path });
        });
    }

    /// Take the answer for `purpose` if the dialog finished this frame.
    pub fn take_dialog(&mut self, purpose: DialogPurpose) -> Option<Option<PathBuf>> {
        let pos = self
            .dialog_results
            .iter()
            .position(|(p, _)| *p == purpose)?;
        Some(self.dialog_results.remove(pos).1)
    }
}
