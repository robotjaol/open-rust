//! Applies a [`Palette`] to egui.

use crate::tokens::{Palette, radius, text};
use app_config::{Accent, Appearance, Density, ThemeMode};
use egui::style::{Selection, TextCursorStyle, WidgetVisuals};
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Theme,
    ThemePreference,
};

fn widget(bg: Color32, border: Color32, fg: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: Stroke::new(1.0, border),
        corner_radius: CornerRadius::same(radius::MD),
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

/// Build egui visuals for a palette.
pub fn visuals(p: &Palette) -> egui::Visuals {
    let mut v = if p.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.dark_mode = p.dark;
    v.override_text_color = None;
    v.panel_fill = p.bg_base;
    v.window_fill = p.bg_elevated;
    v.window_stroke = Stroke::new(1.0, p.border_strong);
    v.window_corner_radius = CornerRadius::same(radius::LG);
    v.menu_corner_radius = CornerRadius::same(radius::MD);
    v.extreme_bg_color = p.bg_base;
    v.faint_bg_color = if p.dark {
        Color32::from_white_alpha(5)
    } else {
        Color32::from_black_alpha(6)
    };
    v.code_bg_color = p.bg_hover;
    v.text_edit_bg_color = Some(p.bg_base);
    v.hyperlink_color = p.accent_fg;
    v.warn_fg_color = p.warning;
    v.error_fg_color = p.danger;
    v.selection = Selection {
        bg_fill: p.accent_soft,
        stroke: Stroke::new(1.0, p.accent_fg),
    };
    v.window_shadow = Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(if p.dark { 140 } else { 48 }),
    };
    v.popup_shadow = Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(if p.dark { 120 } else { 40 }),
    };
    v.widgets.noninteractive = widget(p.bg_surface, p.border, p.text_secondary);
    v.widgets.inactive = widget(p.bg_hover, p.border, p.text);
    v.widgets.hovered = widget(p.bg_active, p.border_strong, p.text);
    v.widgets.active = widget(p.bg_active, p.accent_fg, p.text);
    v.widgets.open = widget(p.bg_active, p.border_strong, p.text);
    v.text_cursor = TextCursorStyle {
        stroke: Stroke::new(2.0, p.accent_fg),
        ..Default::default()
    };
    v.striped = false;
    v.slider_trailing_fill = true;
    v
}

/// Typography and spacing for a density setting.
pub fn style(density: Density) -> egui::Style {
    let mut s = egui::Style::default();
    let compact = density == Density::Compact;
    s.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(text::TITLE, FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(text::BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(text::BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(text::CAPTION, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        ),
    ]
    .into();
    s.spacing.item_spacing = if compact {
        egui::vec2(6.0, 4.0)
    } else {
        egui::vec2(8.0, 8.0)
    };
    s.spacing.button_padding = if compact {
        egui::vec2(8.0, 3.0)
    } else {
        egui::vec2(12.0, 6.0)
    };
    s.spacing.interact_size.y = if compact { 24.0 } else { 30.0 };
    s.spacing.window_margin = Margin::same(16);
    s.spacing.menu_margin = Margin::same(6);
    s.spacing.indent = 16.0;
    s.spacing.scroll.bar_width = 8.0;
    s.spacing.scroll.floating = true;
    s.interaction.selectable_labels = false;
    s
}

/// Install fonts-independent styling for both themes and select the active one.
pub fn apply(ctx: &egui::Context, appearance: &Appearance) {
    let dark = Palette::new(true, appearance.accent);
    let light = Palette::new(false, appearance.accent);
    ctx.set_style_of(Theme::Dark, style(appearance.density));
    ctx.set_style_of(Theme::Light, style(appearance.density));
    ctx.set_visuals_of(Theme::Dark, visuals(&dark));
    ctx.set_visuals_of(Theme::Light, visuals(&light));
    ctx.set_theme(match appearance.theme {
        ThemeMode::Dark => ThemePreference::Dark,
        ThemeMode::Light => ThemePreference::Light,
        ThemeMode::System => ThemePreference::System,
    });
    ctx.set_zoom_factor(appearance.ui_scale.clamp(0.8, 1.6));
}

/// Palette matching what is currently displayed (resolves "system").
pub fn current_palette(ctx: &egui::Context, accent: Accent) -> Palette {
    Palette::new(ctx.global_style().visuals.dark_mode, accent)
}
