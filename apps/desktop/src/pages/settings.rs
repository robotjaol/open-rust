use crate::cx::Cx;
use crate::module::FeatureModule;
use app_config::Config;
use app_core::{DialogPurpose, Notification, Page};
use app_platform::dialogs::DialogKind;
use ui_components::Icon;
use ui_components::overlays::{ConfirmOutcome, confirm_dialog};
use ui_components::tokens::space;
use ui_components::widgets::{self, ButtonKind};

#[derive(Default)]
pub struct Settings {
    confirm_reset: bool,
    confirm_restore: Option<std::path::PathBuf>,
}

const LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];

impl FeatureModule for Settings {
    fn page(&self) -> Page {
        Page::Settings
    }
    fn icon(&self) -> Icon {
        Icon::Settings
    }
    fn subtitle(&self) -> &'static str {
        "Preferences and data"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        if let Some(Some(path)) = cx.take_dialog(DialogPurpose::BackupDatabase) {
            let db = cx.app.db.clone();
            cx.app.tasks.spawn("Back up database", move |ctx| {
                ctx.progress(0.3, "Copying database");
                db.backup_to(&path)?;
                Ok(format!(
                    "Backup written to {}",
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                ))
            });
        }
        if let Some(Some(path)) = cx.take_dialog(DialogPurpose::RestoreDatabase) {
            self.confirm_restore = Some(path);
        }
        widgets::page_header(
            ui,
            &pal,
            "Settings",
            "Changes apply immediately and are saved to the user settings file.",
        );

        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "General");
            let mut restore = cx.app.config.general.restore_window;
            widgets::setting_row(
                ui,
                &pal,
                "Restore window position",
                "Reopen at the last size and position.",
                |ui| {
                    if widgets::toggle(ui, &pal, &mut restore, "").changed() {
                        cx.app.config.general.restore_window = restore;
                        cx.config_dirty = true;
                    }
                },
            );
            let mut welcome = cx.app.config.general.show_welcome;
            widgets::setting_row(
                ui,
                &pal,
                "Show welcome page",
                "Open the Welcome page on startup.",
                |ui| {
                    if widgets::toggle(ui, &pal, &mut welcome, "").changed() {
                        cx.app.config.general.show_welcome = welcome;
                        cx.config_dirty = true;
                    }
                },
            );
            widgets::setting_row(
                ui,
                &pal,
                "Language",
                "English only. The text is not yet extracted for translation.",
                |ui| {
                    widgets::badge(ui, &pal, ui_components::Tone::Neutral, "English");
                },
            );
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Files and logging");
            widgets::setting_row(
                ui,
                &pal,
                "Maximum file size to read",
                "Larger files are refused before they are loaded.",
                |ui| {
                    let mut mb = cx.app.config.files.max_file_size_mb;
                    if ui
                        .add(egui::DragValue::new(&mut mb).range(1..=512).suffix(" MB"))
                        .changed()
                    {
                        cx.app.config.files.max_file_size_mb = mb;
                        cx.config_dirty = true;
                    }
                },
            );
            widgets::setting_row(
                ui,
                &pal,
                "Log level",
                "Applies at the next start. RUST_LOG overrides it.",
                |ui| {
                    let mut cur = cx.app.config.logging.level.clone();
                    egui::ComboBox::from_id_salt("loglevel")
                        .selected_text(cur.clone())
                        .show_ui(ui, |ui| {
                            for l in LEVELS {
                                ui.selectable_value(&mut cur, l.to_string(), l);
                            }
                        });
                    if cur != cx.app.config.logging.level {
                        cx.app.config.logging.level = cur;
                        cx.config_dirty = true;
                    }
                },
            );
            let mut to_file = cx.app.config.logging.to_file;
            widgets::setting_row(
                ui,
                &pal,
                "Write log file",
                "Rotating file in the log directory.",
                |ui| {
                    if widgets::toggle(ui, &pal, &mut to_file, "").changed() {
                        cx.app.config.logging.to_file = to_file;
                        cx.config_dirty = true;
                    }
                },
            );
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Data");
            widgets::kv(
                ui,
                &pal,
                "Database",
                &cx.app.paths.database_file().display().to_string(),
            );
            widgets::kv(
                ui,
                &pal,
                "Settings file",
                &cx.app.paths.config_file().display().to_string(),
            );
            ui.add_space(space::SM);
            ui.horizontal(|ui| {
                if widgets::button(
                    ui,
                    &pal,
                    ButtonKind::Secondary,
                    Some(Icon::Download),
                    "Back up database",
                )
                .clicked()
                {
                    cx.open_dialog(
                        DialogPurpose::BackupDatabase,
                        DialogKind::SaveFile,
                        "Back up database",
                        "open-rust-backup.sqlite3",
                    );
                }
                if widgets::button(
                    ui,
                    &pal,
                    ButtonKind::Secondary,
                    Some(Icon::Upload),
                    "Restore from backup",
                )
                .clicked()
                {
                    cx.open_dialog(
                        DialogPurpose::RestoreDatabase,
                        DialogKind::OpenFile,
                        "Choose a backup",
                        "",
                    );
                }
                if widgets::button(
                    ui,
                    &pal,
                    ButtonKind::Danger,
                    Some(Icon::Refresh),
                    "Reset settings",
                )
                .clicked()
                {
                    self.confirm_reset = true;
                }
            });
        });
        if self.confirm_reset {
            match confirm_dialog(
                ui.ctx(),
                &pal,
                "reset-settings",
                "Reset all settings?",
                "Appearance, window and file settings return to their defaults. Your data is not touched.",
                "Reset settings",
                true,
            ) {
                ConfirmOutcome::Confirmed => {
                    self.confirm_reset = false;
                    cx.app.config = Config::default();
                    cx.config_dirty = true;
                    cx.appearance_dirty = true;
                    cx.notify(Notification::success(
                        "Settings reset",
                        "Defaults restored.",
                    ));
                }
                ConfirmOutcome::Cancelled => self.confirm_reset = false,
                ConfirmOutcome::Pending => {}
            }
        }
        if let Some(path) = self.confirm_restore.clone() {
            match confirm_dialog(
                ui.ctx(),
                &pal,
                "restore-db",
                "Replace current data?",
                "The database is replaced by the selected backup after it passes an integrity check.",
                "Restore backup",
                true,
            ) {
                ConfirmOutcome::Confirmed => {
                    self.confirm_restore = None;
                    match cx.app.db.restore_from(&path) {
                        Ok(()) => cx.notify(Notification::success(
                            "Backup restored",
                            "The database was replaced.",
                        )),
                        Err(e) => cx.fail("Could not restore backup", &e),
                    }
                }
                ConfirmOutcome::Cancelled => self.confirm_restore = None,
                ConfirmOutcome::Pending => {}
            }
        }
    }
}
