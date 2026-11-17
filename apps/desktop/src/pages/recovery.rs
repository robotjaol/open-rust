use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::{Notification, Page, acknowledge_crash};
use app_errors::AppError;
use egui::RichText;
use ui_components::tokens::{space, text};
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Tone};

#[derive(Default)]
pub struct Recovery {
    integrity: Option<Result<(), String>>,
    sample_error: Option<String>,
}

impl FeatureModule for Recovery {
    fn page(&self) -> Page {
        Page::Recovery
    }
    fn icon(&self) -> Icon {
        Icon::LifeBuoy
    }
    fn subtitle(&self) -> &'static str {
        "Errors and recovery"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "Recovery",
            "What happened, what is safe, and how to continue.",
        );

        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Previous session");
            match cx.app.previous_crash.clone() {
                Some(c) => {
                    widgets::badge(ui, &pal, Tone::Warning, "Unexpected exit detected");
                    widgets::kv(
                        ui,
                        &pal,
                        "When",
                        &format!("{} UTC", app_errors::time::format_utc(c.unix_time)),
                    );
                    widgets::kv(ui, &pal, "Thread", &c.thread);
                    widgets::kv(ui, &pal, "Location", &c.location);
                    widgets::kv(ui, &pal, "Message", &c.message);
                    ui.label(
                        RichText::new(format!("Full report: {}", c.file.display()))
                            .size(text::CAPTION)
                            .color(pal.text_muted),
                    );
                    ui.add_space(space::SM);
                    if widgets::button(
                        ui,
                        &pal,
                        ButtonKind::Primary,
                        Some(Icon::Check),
                        "Dismiss report",
                    )
                    .clicked()
                    {
                        match acknowledge_crash(&c) {
                            Ok(()) => {
                                cx.app.previous_crash = None;
                                cx.notify(Notification::success(
                                    "Report dismissed",
                                    "The file was archived.",
                                ));
                            }
                            Err(e) => cx.fail("Could not archive the report", &e),
                        }
                    }
                }
                None => {
                    widgets::badge(ui, &pal, Tone::Success, "Clean exit");
                    ui.label(
                        RichText::new("The last session ended normally.").color(pal.text_secondary),
                    );
                }
            }
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Settings file");
            match &cx.app.config_problem {
                Some(p) => {
                    widgets::badge(ui, &pal, Tone::Warning, "Defaults in use");
                    ui.label(RichText::new(&p.user_message).color(pal.text_secondary));
                    ui.label(RichText::new("The unreadable file was kept next to the original with a .corrupt suffix.").size(text::CAPTION).color(pal.text_muted));
                }
                None => {
                    widgets::badge(ui, &pal, Tone::Success, "Settings loaded");
                }
            }
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Self-checks");
            ui.horizontal(|ui| {
                if widgets::button(
                    ui,
                    &pal,
                    ButtonKind::Secondary,
                    Some(Icon::Database),
                    "Check database",
                )
                .clicked()
                {
                    self.integrity =
                        Some(cx.app.db.integrity_check().map_err(|e| e.user_message()));
                }
                if widgets::button(
                    ui,
                    &pal,
                    ButtonKind::Secondary,
                    Some(Icon::Alert),
                    "Trigger sample error",
                )
                .clicked()
                {
                    let e = AppError::Validation(
                        "This is a sample error for the recovery page.".into(),
                    );
                    tracing::warn!(code = e.code(), "sample error raised");
                    self.sample_error = Some(format!("{}  ({})", e.user_message(), e.code()));
                }
            });
            match &self.integrity {
                Some(Ok(())) => {
                    widgets::badge(ui, &pal, Tone::Success, "Database integrity: ok");
                }
                Some(Err(m)) => {
                    widgets::badge(ui, &pal, Tone::Danger, "Database problem");
                    ui.label(RichText::new(m).color(pal.text_secondary));
                }
                None => {}
            }
            if let Some(m) = &self.sample_error {
                ui.label(RichText::new(m).color(pal.text_secondary));
            }
        });
    }
}
