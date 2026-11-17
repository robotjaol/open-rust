use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::Page;
use app_logging::LogLevel;
use egui::RichText;
use ui_components::tokens::text;
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Tone};

#[derive(Default)]
pub struct Logs {
    level: usize,
    search: String,
}

const LEVELS: [&str; 5] = ["All", "Info+", "Warn+", "Error", "Debug+"];

impl FeatureModule for Logs {
    fn page(&self) -> Page {
        Page::Logs
    }
    fn icon(&self) -> Icon {
        Icon::Terminal
    }
    fn subtitle(&self) -> &'static str {
        "In-app log viewer"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "Logs",
            "Structured events captured in memory (last 2000). The same events go to a rotating file.",
        );
        let min = match self.level {
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Error,
            4 => LogLevel::Debug,
            _ => LogLevel::Trace,
        };
        let q = self.search.to_lowercase();
        let records: Vec<_> = cx
            .app
            .logs
            .snapshot()
            .into_iter()
            .filter(|r| {
                r.level >= min
                    && (q.is_empty()
                        || r.message.to_lowercase().contains(&q)
                        || r.target.to_lowercase().contains(&q))
            })
            .collect();
        ui.horizontal_wrapped(|ui| {
            widgets::search_field(ui, &pal, &mut self.search, "Filter messages", 220.0);
            let mut l = self.level;
            if widgets::segmented(ui, &pal, &mut l, &LEVELS) {
                self.level = l;
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::File),
                "Copy visible",
            )
            .clicked()
            {
                let text: String = records
                    .iter()
                    .map(|r| {
                        format!(
                            "{} {:>5} {} {}\n",
                            app_errors::time::format_clock(r.unix_time),
                            r.level.label(),
                            r.target,
                            r.message
                        )
                    })
                    .collect();
                ui.ctx().copy_text(text);
            }
            if widgets::button(ui, &pal, ButtonKind::Ghost, Some(Icon::Trash), "Clear").clicked() {
                cx.app.logs.clear();
            }
        });
        ui.label(
            RichText::new(format!(
                "{} of {} records{}",
                records.len(),
                cx.app.logs.len(),
                cx.app
                    .log_file
                    .as_ref()
                    .map(|p| format!("  |  file: {}", p.display()))
                    .unwrap_or_default()
            ))
            .size(text::CAPTION)
            .color(pal.text_muted),
        );
        widgets::card(ui, &pal, |ui| {
            if records.is_empty() {
                widgets::empty_state(
                    ui,
                    &pal,
                    Icon::Terminal,
                    "No log records",
                    "Nothing matches the current filter.",
                    |_| {},
                );
                return;
            }
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(ui.available_height().max(300.0))
                .show(ui, |ui| {
                    for r in &records {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(app_errors::time::format_clock(r.unix_time))
                                    .monospace()
                                    .size(text::CAPTION)
                                    .color(pal.text_muted),
                            );
                            let tone = match r.level {
                                LogLevel::Error => Tone::Danger,
                                LogLevel::Warn => Tone::Warning,
                                LogLevel::Info => Tone::Info,
                                _ => Tone::Neutral,
                            };
                            ui.label(
                                RichText::new(format!("{:<5}", r.level.label()))
                                    .monospace()
                                    .size(text::CAPTION)
                                    .color(pal.tone(tone)),
                            );
                            ui.label(
                                RichText::new(&r.message)
                                    .monospace()
                                    .size(text::CAPTION)
                                    .color(pal.text),
                            );
                        });
                    }
                });
        });
    }
}
