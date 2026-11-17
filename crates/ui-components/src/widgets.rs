//! Reusable widgets. All are painted from the active [`Palette`], expose accessible labels
//! to AccessKit, draw a visible focus ring, and never use colour as the only signal.

use crate::icons::{self, Icon};
use crate::tokens::{Palette, Tone, radius, space, text};
use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Layout, Margin, Rect,
    Response, RichText, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

fn focus_ring(ui: &Ui, rect: Rect, pal: &Palette, rounding: u8) {
    ui.painter().rect_stroke(
        rect.expand(2.0),
        CornerRadius::same(rounding + 2),
        Stroke::new(2.0, pal.accent_fg),
        StrokeKind::Outside,
    );
}

/// Text button, optionally with a leading icon.
pub fn button(
    ui: &mut Ui,
    pal: &Palette,
    kind: ButtonKind,
    icon: Option<Icon>,
    label: &str,
) -> Response {
    let font = FontId::proportional(text::BODY);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font, Color32::PLACEHOLDER);
    let icon_w = if icon.is_some() { 16.0 + 6.0 } else { 0.0 };
    let h = ui.spacing().interact_size.y;
    let size = vec2(galley.size().x + icon_w + 24.0, h);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    let enabled = ui.is_enabled();
    let hovered = resp.hovered() && enabled;
    let pressed = resp.is_pointer_button_down_on() && enabled;
    let (bg, fg, border) = match kind {
        ButtonKind::Primary => (
            if pressed || hovered {
                pal.accent_hover
            } else {
                pal.accent
            },
            pal.on_accent,
            Color32::TRANSPARENT,
        ),
        ButtonKind::Secondary => (
            if pressed {
                pal.bg_active
            } else if hovered {
                pal.bg_hover
            } else {
                pal.bg_surface
            },
            pal.text,
            pal.border_strong,
        ),
        ButtonKind::Ghost => (
            if pressed {
                pal.bg_active
            } else if hovered {
                pal.bg_hover
            } else {
                Color32::TRANSPARENT
            },
            pal.text_secondary,
            Color32::TRANSPARENT,
        ),
        ButtonKind::Danger => (
            if hovered || pressed {
                pal.danger
            } else {
                pal.tone_bg(Tone::Danger)
            },
            if hovered || pressed {
                pal.bg_base
            } else {
                pal.danger
            },
            pal.danger,
        ),
    };
    let (bg, fg) = if enabled {
        (bg, fg)
    } else {
        (pal.bg_hover, pal.text_muted)
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::MD),
        bg,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    let mut x = rect.left() + 12.0;
    if let Some(i) = icon {
        icons::paint_at(
            ui.painter(),
            i,
            egui::pos2(x + 8.0, rect.center().y),
            16.0,
            fg,
        );
        x += icon_w;
    }
    ui.painter().galley(
        egui::pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
    if resp.has_focus() {
        focus_ring(ui, rect, pal, radius::MD);
    }
    if enabled {
        resp.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        resp
    }
}

pub fn primary_button(ui: &mut Ui, pal: &Palette, label: &str) -> Response {
    button(ui, pal, ButtonKind::Primary, None, label)
}

pub fn secondary_button(ui: &mut Ui, pal: &Palette, label: &str) -> Response {
    button(ui, pal, ButtonKind::Secondary, None, label)
}

/// Square icon-only button. `tooltip` doubles as the accessible name.
pub fn icon_button(ui: &mut Ui, pal: &Palette, icon: Icon, tooltip: &str) -> Response {
    let size = Vec2::splat(ui.spacing().interact_size.y.max(28.0));
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), tooltip));
    let hovered = resp.hovered();
    if hovered || resp.is_pointer_button_down_on() {
        let bg = if resp.is_pointer_button_down_on() {
            pal.bg_active
        } else {
            pal.bg_hover
        };
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::MD), bg);
    }
    let fg = if ui.is_enabled() {
        if hovered {
            pal.text
        } else {
            pal.text_secondary
        }
    } else {
        pal.text_muted
    };
    icons::paint_at(ui.painter(), icon, rect.center(), 18.0, fg);
    if resp.has_focus() {
        focus_ring(ui, rect, pal, radius::MD);
    }
    resp.on_hover_text(tooltip)
        .on_hover_cursor(CursorIcon::PointingHand)
}

/// Small rounded label with a tone-coloured tint.
pub fn badge(ui: &mut Ui, pal: &Palette, tone: Tone, label: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(text::CAPTION),
        pal.tone(tone),
    );
    let size = vec2(galley.size().x + 14.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), pal.tone_bg(tone));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, pal.tone(tone));
    resp.widget_info(|| WidgetInfo::labeled(Role::Label, true, label));
    resp
}

/// Status dot followed by a label (shape + text, so colour is never the only cue).
pub fn status(ui: &mut Ui, pal: &Palette, tone: Tone, label: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
        ui.painter().circle_filled(r.center(), 4.0, pal.tone(tone));
        ui.label(RichText::new(label).color(pal.text_secondary));
    });
}

/// Elevated container with border.
pub fn card<R>(ui: &mut Ui, pal: &Palette, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(pal.bg_surface)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(radius::LG))
        .inner_margin(Margin::same(space::LG as i8))
        .show(ui, |ui| {
            // Columns use a justified layout; cards always lay out top-down, left-aligned.
            ui.set_min_width(ui.available_width());
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), add)
                .inner
        })
        .inner
}

/// Page title with optional subtitle.
pub fn page_header(ui: &mut Ui, pal: &Palette, title: &str, subtitle: &str) {
    ui.add_space(space::XS);
    ui.label(
        RichText::new(title)
            .size(text::TITLE)
            .strong()
            .color(pal.text),
    );
    if !subtitle.is_empty() {
        ui.label(RichText::new(subtitle).color(pal.text_secondary));
    }
    ui.add_space(space::MD);
}

pub fn section_title(ui: &mut Ui, pal: &Palette, title: &str) {
    ui.add_space(space::SM);
    ui.label(
        RichText::new(title)
            .size(text::SUBTITLE)
            .strong()
            .color(pal.text),
    );
    ui.add_space(space::XS);
}

/// KPI tile: icon, label, big value and a caption.
pub fn stat_tile(
    ui: &mut Ui,
    pal: &Palette,
    icon: Icon,
    label: &str,
    value: &str,
    caption: &str,
    tone: Tone,
) {
    card(ui, pal, |ui| {
        ui.set_min_width(150.0);
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
            ui.painter()
                .rect_filled(r, CornerRadius::same(radius::MD), pal.tone_bg(tone));
            icons::paint_at(ui.painter(), icon, r.center(), 18.0, pal.tone(tone));
            ui.label(RichText::new(label).color(pal.text_secondary));
        });
        ui.add_space(space::XS);
        ui.label(
            RichText::new(value)
                .size(text::DISPLAY)
                .strong()
                .color(pal.text),
        );
        ui.label(
            RichText::new(caption)
                .size(text::CAPTION)
                .color(pal.text_muted),
        );
    });
}

/// Centered placeholder for empty lists. Add a call-to-action in `action`.
pub fn empty_state(
    ui: &mut Ui,
    pal: &Palette,
    icon: Icon,
    title: &str,
    body: &str,
    action: impl FnOnce(&mut Ui),
) {
    ui.vertical_centered(|ui| {
        ui.add_space(space::XXL);
        let (r, _) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
        ui.painter().circle_filled(r.center(), 28.0, pal.bg_hover);
        icons::paint_at(ui.painter(), icon, r.center(), 26.0, pal.text_secondary);
        ui.add_space(space::SM);
        ui.label(
            RichText::new(title)
                .size(text::SUBTITLE)
                .strong()
                .color(pal.text),
        );
        ui.label(RichText::new(body).color(pal.text_secondary));
        ui.add_space(space::MD);
        action(ui);
        ui.add_space(space::XXL);
    });
}

/// Determinate progress bar with a text percentage.
pub fn progress_bar(ui: &mut Ui, pal: &Palette, fraction: f32, tone: Tone, width: f32) {
    let f = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 8.0), Sense::hover());
    resp.widget_info(|| {
        let mut i = WidgetInfo::labeled(Role::ProgressIndicator, true, "progress");
        i.value = Some(f64::from(f));
        i
    });
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), pal.bg_hover);
    let mut fill = rect;
    fill.set_width((rect.width() * f).max(if f > 0.0 { 8.0 } else { 0.0 }));
    ui.painter().rect_filled(
        fill,
        CornerRadius::same(4),
        pal.tone(if tone == Tone::Neutral {
            Tone::Accent
        } else {
            tone
        }),
    );
}

/// Animated placeholder block shown while content loads.
pub fn skeleton(ui: &mut Ui, pal: &Palette, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let t = ui.input(|i| i.time) as f32;
    let pulse = (t * 2.0).sin() * 0.5 + 0.5;
    let base = pal.bg_hover;
    let alt = pal.bg_active;
    let c = Color32::from_rgb(
        lerp_u8(base.r(), alt.r(), pulse),
        lerp_u8(base.g(), alt.g(), pulse),
        lerp_u8(base.b(), alt.b(), pulse),
    );
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius::SM), c);
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(50));
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8
}

/// Keyboard shortcut hint such as `Ctrl K`.
pub fn kbd(ui: &mut Ui, pal: &Palette, keys: &str) {
    let galley = ui.painter().layout_no_wrap(
        keys.to_string(),
        FontId::monospace(11.0),
        pal.text_secondary,
    );
    let (rect, _) = ui.allocate_exact_size(galley.size() + vec2(12.0, 6.0), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::SM),
        pal.bg_hover,
        Stroke::new(1.0, pal.border),
        StrokeKind::Inside,
    );
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        pal.text_secondary,
    );
}

/// iOS-style switch with an optional label.
pub fn toggle(ui: &mut Ui, pal: &Palette, on: &mut bool, label: &str) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(38.0, 22.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let value = *on;
    resp.widget_info(|| WidgetInfo::selected(Role::CheckBox, ui.is_enabled(), value, label));
    let t = ui.ctx().animate_bool(resp.id, *on);
    let track = if *on { pal.accent } else { pal.bg_active };
    ui.painter().rect(
        rect,
        CornerRadius::same(11),
        track,
        Stroke::new(
            1.0,
            if *on {
                Color32::TRANSPARENT
            } else {
                pal.border_strong
            },
        ),
        StrokeKind::Inside,
    );
    let knob_x = rect.left() + 11.0 + t * 16.0;
    let knob = if *on {
        pal.on_accent
    } else {
        pal.text_secondary
    };
    ui.painter()
        .circle_filled(egui::pos2(knob_x, rect.center().y), 8.0, knob);
    if resp.has_focus() {
        focus_ring(ui, rect, pal, 11);
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Row used on settings pages: title and description on the left, control on the right.
pub fn setting_row(
    ui: &mut Ui,
    pal: &Palette,
    title: &str,
    description: &str,
    control: impl FnOnce(&mut Ui),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_width((ui.available_width() - 220.0).max(160.0));
            ui.label(RichText::new(title).color(pal.text));
            if !description.is_empty() {
                ui.label(
                    RichText::new(description)
                        .size(text::CAPTION)
                        .color(pal.text_muted),
                );
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
    ui.add_space(space::XS);
    ui.separator();
    ui.add_space(space::XS);
}

/// Segmented control. Returns true when the selection changed.
pub fn segmented(ui: &mut Ui, pal: &Palette, selected: &mut usize, labels: &[&str]) -> bool {
    let mut changed = false;
    Frame::new()
        .fill(pal.bg_base)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(radius::MD))
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (i, label) in labels.iter().enumerate() {
                    let galley = ui.painter().layout_no_wrap(
                        (*label).to_string(),
                        FontId::proportional(text::BODY),
                        pal.text,
                    );
                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(galley.size().x + 22.0, 26.0), Sense::click());
                    let sel = *selected == i;
                    resp.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, sel, label));
                    if sel {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(radius::SM), pal.accent);
                    } else if resp.hovered() {
                        ui.painter().rect_filled(
                            rect,
                            CornerRadius::same(radius::SM),
                            pal.bg_hover,
                        );
                    }
                    let fg = if sel {
                        pal.on_accent
                    } else {
                        pal.text_secondary
                    };
                    ui.painter()
                        .galley(rect.center() - galley.size() / 2.0, galley, fg);
                    if resp.has_focus() {
                        focus_ring(ui, rect, pal, radius::SM);
                    }
                    if resp.clicked() && !sel {
                        *selected = i;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// Search input with a leading magnifier. Returns the text edit response.
pub fn search_field(
    ui: &mut Ui,
    pal: &Palette,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    let h = ui.spacing().interact_size.y;
    let (rect, _) = ui.allocate_exact_size(vec2(width, h), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::MD),
        pal.bg_base,
        Stroke::new(1.0, pal.border_strong),
        StrokeKind::Inside,
    );
    icons::paint_at(
        ui.painter(),
        Icon::Search,
        egui::pos2(rect.left() + 16.0, rect.center().y),
        15.0,
        pal.text_muted,
    );
    let inner = Rect::from_min_max(rect.min + vec2(30.0, 2.0), rect.max - vec2(6.0, 2.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let resp = child.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .frame(egui::Frame::NONE)
            .desired_width(inner.width())
            .text_color(pal.text),
    );
    resp.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, hint));
    if resp.has_focus() {
        focus_ring(ui, rect, pal, radius::MD);
    }
    resp
}

/// Sidebar navigation row. Returns the click response.
pub fn nav_item(
    ui: &mut Ui,
    pal: &Palette,
    icon: Icon,
    label: &str,
    selected: bool,
    collapsed: bool,
    badge_text: Option<&str>,
) -> Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
    resp.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label));
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::MD), pal.accent_soft);
        let bar = Rect::from_min_size(rect.left_center() - vec2(0.0, 9.0), vec2(3.0, 18.0));
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), pal.accent_fg);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::MD), pal.bg_hover);
    }
    let fg = if selected {
        pal.accent_fg
    } else if resp.hovered() {
        pal.text
    } else {
        pal.text_secondary
    };
    let icon_center = if collapsed {
        rect.center()
    } else {
        egui::pos2(rect.left() + 20.0, rect.center().y)
    };
    icons::paint_at(ui.painter(), icon, icon_center, 18.0, fg);
    if !collapsed {
        ui.painter().text(
            egui::pos2(rect.left() + 38.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(text::BODY),
            if selected { pal.text } else { fg },
        );
        if let Some(b) = badge_text {
            let g = ui.painter().layout_no_wrap(
                b.to_string(),
                FontId::proportional(10.5),
                pal.on_accent,
            );
            let r = Rect::from_center_size(
                egui::pos2(rect.right() - 14.0 - g.size().x / 2.0, rect.center().y),
                g.size() + vec2(10.0, 4.0),
            );
            ui.painter()
                .rect_filled(r, CornerRadius::same(8), pal.accent);
            ui.painter()
                .galley(r.center() - g.size() / 2.0, g, pal.on_accent);
        }
    }
    if resp.has_focus() {
        focus_ring(ui, rect, pal, radius::MD);
    }
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    if collapsed {
        resp.on_hover_text(label)
    } else {
        resp
    }
}

pub fn nav_group_label(ui: &mut Ui, pal: &Palette, label: &str, collapsed: bool) {
    ui.add_space(space::MD);
    if collapsed {
        ui.separator();
    } else {
        ui.label(
            RichText::new(label.to_uppercase())
                .size(10.5)
                .strong()
                .color(pal.text_muted),
        );
    }
    ui.add_space(2.0);
}

/// Key-value line used on About and detail panels.
pub fn kv(ui: &mut Ui, pal: &Palette, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [130.0, 18.0],
            egui::Label::new(RichText::new(key).color(pal.text_muted)),
        );
        ui.label(RichText::new(value).color(pal.text));
    });
}
