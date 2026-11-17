use crate::cx::Cx;
use crate::module::FeatureModule;
use app_config::{Accent, Density, ThemeMode};
use app_core::Page;
use egui::{CornerRadius, RichText, Sense, Stroke};
use ui_components::tokens::{radius, space, text};
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Palette, Tone};

pub struct AppearancePage;

impl FeatureModule for AppearancePage {
    fn page(&self) -> Page {
        Page::Appearance
    }
    fn icon(&self) -> Icon {
        Icon::Palette
    }
    fn subtitle(&self) -> &'static str {
        "Theme, accent and density"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "Appearance",
            "Every colour comes from design tokens. Contrast of text and accent is checked in tests (WCAG AA, 4.5:1).",
        );
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Theme");
            let mut sel = match cx.app.config.appearance.theme {
                ThemeMode::System => 0,
                ThemeMode::Light => 1,
                ThemeMode::Dark => 2,
            };
            if widgets::segmented(ui, &pal, &mut sel, &["System", "Light", "Dark"]) {
                cx.app.config.appearance.theme =
                    [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark][sel];
                cx.config_dirty = true;
            }
            ui.add_space(space::MD);
            widgets::section_title(ui, &pal, "Accent colour");
            ui.horizontal(|ui| {
                for a in Accent::ALL {
                    let swatch = Palette::new(pal.dark, a).accent;
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), Sense::click());
                    ui.painter()
                        .rect_filled(r, CornerRadius::same(radius::MD), swatch);
                    if cx.app.config.appearance.accent == a {
                        ui.painter().rect_stroke(
                            r.expand(3.0),
                            CornerRadius::same(radius::MD + 2),
                            Stroke::new(2.0, pal.text),
                            egui::StrokeKind::Outside,
                        );
                        ui_components::icons::paint_at(
                            ui.painter(),
                            Icon::Check,
                            r.center(),
                            16.0,
                            Palette::new(pal.dark, a).on_accent,
                        );
                    }
                    let resp = resp.on_hover_text(a.label());
                    resp.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::Button,
                            true,
                            format!("Accent {}", a.label()),
                        )
                    });
                    if resp.clicked() {
                        cx.app.config.appearance.accent = a;
                        cx.config_dirty = true;
                    }
                }
            });
            ui.add_space(space::MD);
            widgets::section_title(ui, &pal, "Density");
            let mut d = match cx.app.config.appearance.density {
                Density::Compact => 0,
                Density::Comfortable => 1,
            };
            if widgets::segmented(ui, &pal, &mut d, &["Compact", "Comfortable"]) {
                cx.app.config.appearance.density = if d == 0 {
                    Density::Compact
                } else {
                    Density::Comfortable
                };
                cx.config_dirty = true;
            }
            ui.add_space(space::MD);
            widgets::section_title(ui, &pal, "Interface scale");
            let mut scale = cx.app.config.appearance.ui_scale;
            if ui
                .add(
                    egui::Slider::new(&mut scale, 0.8..=1.6)
                        .step_by(0.05)
                        .suffix("x"),
                )
                .changed()
            {
                cx.app.config.appearance.ui_scale = scale;
                cx.config_dirty = true;
            }
        });
        ui.add_space(space::MD);
        widgets::card(ui, &pal, |ui| {
            widgets::section_title(ui, &pal, "Live preview");
            ui.horizontal(|ui| {
                widgets::button(ui, &pal, ButtonKind::Primary, Some(Icon::Check), "Primary");
                widgets::button(ui, &pal, ButtonKind::Secondary, None, "Secondary");
                widgets::button(ui, &pal, ButtonKind::Danger, Some(Icon::Trash), "Danger");
                widgets::badge(ui, &pal, Tone::Success, "Success");
                widgets::badge(ui, &pal, Tone::Warning, "Warning");
                widgets::badge(ui, &pal, Tone::Info, "Info");
            });
            ui.add_space(space::SM);
            widgets::progress_bar(ui, &pal, 0.62, Tone::Accent, 320.0);
            ui.label(
                RichText::new(
                    "Secondary text uses the muted token and stays above 4.5:1 contrast.",
                )
                .size(text::CAPTION)
                .color(pal.text_secondary),
            );
        });
    }
}
