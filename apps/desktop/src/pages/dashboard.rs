use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::Page;
use app_platform::info::process_rss_bytes;
use app_storage::ItemStats;
use egui::RichText;
use ui_components::tokens::{space, text};
use ui_components::{Icon, Tone, charts, widgets};

#[derive(Default)]
pub struct Dashboard {
    stats: Option<ItemStats>,
    schema: u32,
}

impl FeatureModule for Dashboard {
    fn page(&self) -> Page {
        Page::Dashboard
    }
    fn icon(&self) -> Icon {
        Icon::Dashboard
    }
    fn subtitle(&self) -> &'static str {
        "Overview of the workspace"
    }

    fn on_enter(&mut self, cx: &mut Cx<'_>) {
        match cx.app.db.item_stats() {
            Ok(s) => self.stats = Some(s),
            Err(e) => cx.fail("Could not load statistics", &e),
        }
        self.schema = cx.app.db.schema_version().unwrap_or(0);
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "Dashboard",
            "Live figures from the local SQLite database and the running process.",
        );
        let Some(s) = self.stats.clone() else {
            widgets::empty_state(
                ui,
                &pal,
                Icon::Database,
                "No data yet",
                "Statistics appear once the database is readable.",
                |_| {},
            );
            return;
        };
        let rss = process_rss_bytes()
            .map(app_platform::fs::format_size)
            .unwrap_or_else(|| "n/a".into());
        let tasks = cx.app.tasks.snapshot();
        ui.columns(4, |c| {
            widgets::stat_tile(
                &mut c[0],
                &pal,
                Icon::Table,
                "Records",
                &s.total.to_string(),
                "in the items table",
                Tone::Accent,
            );
            widgets::stat_tile(
                &mut c[1],
                &pal,
                Icon::Check,
                "Active",
                &s.active.to_string(),
                &format!("{} paused, {} archived", s.paused, s.archived),
                Tone::Success,
            );
            widgets::stat_tile(
                &mut c[2],
                &pal,
                Icon::Tasks,
                "Tasks",
                &tasks.len().to_string(),
                &format!("{} running", cx.app.tasks.running_count()),
                Tone::Info,
            );
            widgets::stat_tile(
                &mut c[3],
                &pal,
                Icon::Bolt,
                "Memory (RSS)",
                &rss,
                "resident set size",
                Tone::Neutral,
            );
        });
        ui.add_space(space::MD);
        ui.columns(2, |c| {
            widgets::card(&mut c[0], &pal, |ui| {
                widgets::section_title(ui, &pal, "Records by category");
                let rows: Vec<(String, f32)> = s
                    .by_category
                    .iter()
                    .map(|(k, v)| (k.clone(), *v as f32))
                    .collect();
                charts::bar_chart(ui, &pal, &rows, ui.available_width());
            });
            widgets::card(&mut c[1], &pal, |ui| {
                widgets::section_title(ui, &pal, "Records by status");
                let segs = vec![
                    ("Active".to_string(), s.active as f32, pal.success),
                    ("Paused".to_string(), s.paused as f32, pal.warning),
                    ("Archived".to_string(), s.archived as f32, pal.text_muted),
                ];
                ui.horizontal(|ui| {
                    charts::donut(ui, &pal, &segs, 132.0, &s.total.to_string());
                    ui.vertical(|ui| {
                        for (name, v, col) in &segs {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(r, 2.0, *col);
                                ui.label(
                                    RichText::new(format!("{name}: {}", *v as u64))
                                        .color(pal.text_secondary),
                                );
                            });
                        }
                    });
                });
            });
        });
        ui.add_space(space::MD);
        ui.columns(2, |c| {
            widgets::card(&mut c[0], &pal, |ui| {
                widgets::section_title(ui, &pal, "Frame time (last 120 frames)");
                let data: Vec<f32> = cx.frame_ms.iter().copied().collect();
                charts::sparkline(
                    ui,
                    &pal,
                    &data,
                    egui::vec2(ui.available_width(), 64.0),
                    pal.accent,
                );
                let avg = if data.is_empty() {
                    0.0
                } else {
                    data.iter().sum::<f32>() / data.len() as f32
                };
                ui.label(
                    RichText::new(format!(
                        "mean {avg:.1} ms over {} frames (measured live, not a benchmark)",
                        data.len()
                    ))
                    .size(text::CAPTION)
                    .color(pal.text_muted),
                );
            });
            widgets::card(&mut c[1], &pal, |ui| {
                widgets::section_title(ui, &pal, "Runtime");
                widgets::kv(
                    ui,
                    &pal,
                    "Startup (boot)",
                    &format!("{} ms", cx.app.boot_time.as_millis()),
                );
                widgets::kv(ui, &pal, "Database schema", &format!("v{}", self.schema));
                widgets::kv(ui, &pal, "Total amount", &format!("{:.2}", s.total_amount));
                widgets::kv(ui, &pal, "Display server", &cx.app.platform.display_server);
            });
        });
    }
}
