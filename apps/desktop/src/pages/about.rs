use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::Page;
use egui::RichText;
use ui_components::Icon;
use ui_components::tokens::{space, text};
use ui_components::widgets;

pub struct About;

impl FeatureModule for About {
    fn page(&self) -> Page {
        Page::About
    }
    fn icon(&self) -> Icon {
        Icon::Info
    }
    fn subtitle(&self) -> &'static str {
        "Version, paths and licence"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "About",
            "Open Rust Desktop Starterpack by robotjaol.",
        );
        ui.columns(2, |c| {
            widgets::card(&mut c[0], &pal, |ui| {
                widgets::section_title(ui, &pal, "Build");
                widgets::kv(ui, &pal, "Version", env!("CARGO_PKG_VERSION"));
                widgets::kv(ui, &pal, "Licence", "MIT OR Apache-2.0");
                widgets::kv(ui, &pal, "GUI toolkit", "egui / eframe 0.37");
                widgets::kv(
                    ui,
                    &pal,
                    "Profile",
                    if cx.app.platform.debug_build {
                        "debug"
                    } else {
                        "release"
                    },
                );
                widgets::kv(
                    ui,
                    &pal,
                    "Environment",
                    &format!("{:?}", cx.app.environment),
                );
            });
            widgets::card(&mut c[1], &pal, |ui| {
                widgets::section_title(ui, &pal, "System");
                widgets::kv(ui, &pal, "OS", cx.app.platform.os);
                widgets::kv(ui, &pal, "Architecture", cx.app.platform.arch);
                widgets::kv(ui, &pal, "Display server", &cx.app.platform.display_server);
                widgets::kv(
                    ui,
                    &pal,
                    "Portable mode",
                    if std::env::var_os("OPENRUST_HOME").is_some() {
                        "yes"
                    } else {
                        "no"
                    },
                );
            });
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Locations");
            widgets::kv(
                ui,
                &pal,
                "Config",
                &cx.app.paths.config_dir.display().to_string(),
            );
            widgets::kv(
                ui,
                &pal,
                "Data",
                &cx.app.paths.data_dir.display().to_string(),
            );
            widgets::kv(
                ui,
                &pal,
                "Logs",
                &cx.app.paths.log_dir.display().to_string(),
            );
            widgets::kv(
                ui,
                &pal,
                "Crash reports",
                &cx.app.paths.crash_dir.display().to_string(),
            );
            ui.add_space(space::SM);
            ui.label(
                RichText::new("No telemetry. Nothing leaves this machine.")
                    .size(text::CAPTION)
                    .color(pal.text_muted),
            );
        });
    }
}
