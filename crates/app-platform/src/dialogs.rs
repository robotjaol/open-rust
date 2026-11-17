//! Native file dialogs.
//!
//! Dialogs block, so the UI must never call them on the render thread. Use
//! [`spawn`], which runs the dialog on its own thread and hands back the choice
//! through a callback (the desktop app forwards it over the event bus).

#[cfg(feature = "native-dialogs")]
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    OpenFile,
    OpenFolder,
    SaveFile,
}

/// True when this build can show native dialogs.
pub const fn available() -> bool {
    cfg!(feature = "native-dialogs")
}

/// Run a dialog on a worker thread and call `done` with the selection (`None` if cancelled
/// or if dialogs are unavailable).
#[cfg(feature = "native-dialogs")]
pub fn spawn(
    kind: DialogKind,
    title: &str,
    default_name: &str,
    done: impl FnOnce(Option<PathBuf>) + Send + 'static,
) {
    let title = title.to_string();
    let default_name = default_name.to_string();
    let spawned = std::thread::Builder::new()
        .name("native-dialog".into())
        .spawn(move || {
            let dlg = rfd::FileDialog::new().set_title(title);
            let picked = match kind {
                DialogKind::OpenFile => dlg.pick_file(),
                DialogKind::OpenFolder => dlg.pick_folder(),
                DialogKind::SaveFile => dlg.set_file_name(default_name).save_file(),
            };
            done(picked);
        });
    if let Err(e) = spawned {
        tracing::error!(error = %e, "could not start dialog thread");
    }
}

#[cfg(not(feature = "native-dialogs"))]
pub fn spawn(
    _kind: DialogKind,
    _title: &str,
    _default_name: &str,
    done: impl FnOnce(Option<std::path::PathBuf>) + Send + 'static,
) {
    done(None);
}
