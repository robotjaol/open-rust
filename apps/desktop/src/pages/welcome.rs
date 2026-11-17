use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::Page;
use egui::RichText;
use ui_components::tokens::{space, text};
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Tone};

pub struct Welcome;

impl FeatureModule for Welcome {
    fn page(&self) -> Page {
        Page::Welcome
    }
    fn icon(&self) -> Icon {
        Icon::Home
    }
    fn subtitle(&self) -> &'static str {
        "Start here"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        ui.add_space(space::LG);
        ui.label(
            RichText::new("Open Rust")
                .size(text::DISPLAY)
                .strong()
                .color(pal.text),
        );
        ui.label(
            RichText::new("A production-oriented starter for native Rust desktop applications.")
                .size(text::SUBTITLE)
                .color(pal.text_secondary),
        );
        ui.add_space(space::LG);
        ui.horizontal(|ui| {
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Primary,
                Some(Icon::Dashboard),
                "Open dashboard",
            )
            .clicked()
            {
                cx.goto(Page::Dashboard);
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Palette),
                "Customize appearance",
            )
            .clicked()
            {
                cx.goto(Page::Appearance);
            }
        });
        ui.add_space(space::XL);
        widgets::section_title(ui, &pal, "What is in this starter");
        let items: [(Icon, &str, &str, Page); 6] = [
            (
                Icon::Database,
                "SQLite storage",
                "Versioned migrations, backup, restore, JSON and CSV transfer.",
                Page::Data,
            ),
            (
                Icon::Tasks,
                "Background tasks",
                "Worker pool with progress, cancellation and panic isolation.",
                Page::Tasks,
            ),
            (
                Icon::Folder,
                "Safe file access",
                "Path policy, size limits and atomic writes.",
                Page::Files,
            ),
            (
                Icon::Notebook,
                "Example feature",
                "Notes with full create, edit and delete flow.",
                Page::Workspace,
            ),
            (
                Icon::Terminal,
                "Structured logs",
                "In-app viewer with level filter and rotating log file.",
                Page::Logs,
            ),
            (
                Icon::Settings,
                "Layered settings",
                "Defaults, environment, user file, OPENRUST_* overrides.",
                Page::Settings,
            ),
        ];
        ui.columns(3, |cols| {
            for (i, (icon, title, body, page)) in items.iter().enumerate() {
                let ui = &mut cols[i % 3];
                widgets::card(ui, &pal, |ui| {
                    ui.set_min_height(112.0);
                    ui.label(RichText::new(*title).strong().color(pal.text));
                    ui.label(
                        RichText::new(*body)
                            .size(text::CAPTION)
                            .color(pal.text_secondary),
                    );
                    ui.add_space(space::SM);
                    if widgets::button(ui, &pal, ButtonKind::Ghost, Some(*icon), "Open").clicked() {
                        cx.goto(*page);
                    }
                });
                ui.add_space(space::SM);
            }
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            ui.horizontal(|ui| {
                widgets::badge(ui, &pal, Tone::Info, "Tip");
                ui.label(
                    RichText::new("Press Ctrl+K to search pages and run commands.")
                        .color(pal.text_secondary),
                );
            });
        });
        ui.add_space(space::MD);
        let mut show = cx.app.config.general.show_welcome;
        ui.horizontal(|ui| {
            if widgets::toggle(ui, &pal, &mut show, "Show this page on startup").changed() {
                cx.app.config.general.show_welcome = show;
                cx.config_dirty = true;
            }
            ui.label(RichText::new("Show this page on startup").color(pal.text_secondary));
        });
    }
}
