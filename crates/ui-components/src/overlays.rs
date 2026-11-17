//! Floating UI: toasts, command palette and confirmation dialogs.

use crate::icons::{self, Icon};
use crate::tokens::{Palette, Tone, radius, space, text};
use crate::widgets::{self, ButtonKind};
use egui::{
    Align2, Color32, Context, CornerRadius, Frame, Id, Key, Margin, RichText, Sense, Stroke, Ui,
    vec2,
};

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub tone: Tone,
    pub title: String,
    pub message: String,
    born: f64,
    ttl: f64,
}

/// Stack of auto-dismissing toasts anchored to the bottom-right corner.
#[derive(Debug, Default)]
pub struct ToastHost {
    toasts: Vec<Toast>,
}

impl ToastHost {
    pub const MAX_VISIBLE: usize = 4;

    pub fn push(&mut self, ctx: &Context, id: u64, tone: Tone, title: &str, message: &str) {
        let ttl = if tone == Tone::Danger { 9.0 } else { 4.5 };
        self.toasts.push(Toast {
            id,
            tone,
            title: title.into(),
            message: message.into(),
            born: ctx.input(|i| i.time),
            ttl,
        });
        if self.toasts.len() > Self::MAX_VISIBLE {
            self.toasts.remove(0);
        }
    }

    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    pub fn show(&mut self, ctx: &Context, pal: &Palette) {
        let now = ctx.input(|i| i.time);
        self.toasts.retain(|t| now - t.born < t.ttl);
        if self.toasts.is_empty() {
            return;
        }
        let mut dismiss = None;
        egui::Area::new(Id::unique("toast-host"))
            .anchor(Align2::RIGHT_BOTTOM, vec2(-16.0, -16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                for t in self.toasts.iter().rev() {
                    Frame::new()
                        .fill(pal.bg_elevated)
                        .stroke(Stroke::new(1.0, pal.border_strong))
                        .corner_radius(CornerRadius::same(radius::LG))
                        .inner_margin(Margin::same(12))
                        .shadow(ctx.global_style().visuals.popup_shadow)
                        .show(ui, |ui| {
                            ui.set_width(320.0);
                            ui.horizontal_top(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                                let icon = match t.tone {
                                    Tone::Success => Icon::Check,
                                    Tone::Warning | Tone::Danger => Icon::Alert,
                                    _ => Icon::Info,
                                };
                                icons::paint_at(
                                    ui.painter(),
                                    icon,
                                    r.center(),
                                    18.0,
                                    pal.tone(t.tone),
                                );
                                ui.vertical(|ui| {
                                    ui.set_width(250.0);
                                    ui.label(RichText::new(&t.title).strong().color(pal.text));
                                    if !t.message.is_empty() {
                                        ui.label(
                                            RichText::new(&t.message)
                                                .size(text::CAPTION + 1.0)
                                                .color(pal.text_secondary),
                                        );
                                    }
                                });
                                if widgets::icon_button(
                                    ui,
                                    pal,
                                    Icon::Close,
                                    "Dismiss notification",
                                )
                                .clicked()
                                {
                                    dismiss = Some(t.id);
                                }
                            });
                        });
                }
            });
        if let Some(id) = dismiss {
            self.toasts.retain(|t| t.id != id);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub label: String,
    pub hint: String,
    pub icon: Icon,
}

#[derive(Debug, Default)]
pub struct PaletteState {
    pub query: String,
    pub selected: usize,
    focus_requested: bool,
}

impl PaletteState {
    pub fn reset(&mut self) {
        self.query.clear();
        self.selected = 0;
        self.focus_requested = false;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PaletteOutcome {
    None,
    Close,
    /// Index into the `items` slice passed to [`command_palette`].
    Run(usize),
}

/// Case-insensitive subsequence match; returns indices of matching items, best first.
pub fn filter_items(items: &[PaletteItem], query: &str) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    let mut scored: Vec<(usize, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| {
            let l = it.label.to_lowercase();
            if q.is_empty() {
                return Some((i, 0));
            }
            if let Some(pos) = l.find(&q) {
                return Some((i, pos)); // substring: earlier match ranks higher
            }
            let mut chars = l.chars();
            q.chars()
                .all(|c| chars.any(|x| x == c))
                .then_some((i, 1000))
        })
        .collect();
    scored.sort_by_key(|&(i, s)| (s, i));
    scored.into_iter().map(|(i, _)| i).collect()
}

pub fn command_palette(
    ctx: &Context,
    pal: &Palette,
    st: &mut PaletteState,
    items: &[PaletteItem],
) -> PaletteOutcome {
    let matches = filter_items(items, &st.query);
    if st.selected >= matches.len() {
        st.selected = matches.len().saturating_sub(1);
    }
    let (down, up, enter) = ctx.input_mut(|i| {
        (
            i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
            i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
            i.consume_key(egui::Modifiers::NONE, Key::Enter),
        )
    });
    if down && !matches.is_empty() {
        st.selected = (st.selected + 1) % matches.len();
    }
    if up && !matches.is_empty() {
        st.selected = (st.selected + matches.len() - 1) % matches.len();
    }
    let mut outcome = PaletteOutcome::None;
    if enter && let Some(&i) = matches.get(st.selected) {
        outcome = PaletteOutcome::Run(i);
    }
    let modal = egui::Modal::new(Id::unique("command-palette"))
        .backdrop_color(Color32::from_black_alpha(if pal.dark { 150 } else { 80 }))
        .frame(
            Frame::new()
                .fill(pal.bg_elevated)
                .stroke(Stroke::new(1.0, pal.border_strong))
                .corner_radius(CornerRadius::same(radius::LG))
                .inner_margin(Margin::same(8)),
        )
        .area(
            egui::Modal::default_area(Id::unique("command-palette"))
                .anchor(Align2::CENTER_TOP, vec2(0.0, 90.0)),
        )
        .show(ctx, |ui| {
            ui.set_width(520.0);
            let resp =
                widgets::search_field(ui, pal, &mut st.query, "Type a command or page name", 504.0);
            if !st.focus_requested {
                resp.request_focus();
                st.focus_requested = true;
            }
            if resp.changed() {
                st.selected = 0;
            }
            ui.add_space(space::XS);
            if matches.is_empty() {
                ui.add_space(space::MD);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No matching commands").color(pal.text_secondary))
                });
                ui.add_space(space::MD);
            }
            egui::ScrollArea::vertical()
                .max_height(320.0)
                .show(ui, |ui| {
                    for (row, &idx) in matches.iter().enumerate() {
                        let it = &items[idx];
                        let (rect, r) = ui
                            .allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
                        r.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::Button, true, &it.label)
                        });
                        let sel = row == st.selected;
                        if sel || r.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::same(radius::MD),
                                if sel { pal.accent_soft } else { pal.bg_hover },
                            );
                        }
                        if sel {
                            ui.scroll_to_rect(rect, None);
                        }
                        icons::paint_at(
                            ui.painter(),
                            it.icon,
                            egui::pos2(rect.left() + 18.0, rect.center().y),
                            17.0,
                            if sel {
                                pal.accent_fg
                            } else {
                                pal.text_secondary
                            },
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 38.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            &it.label,
                            egui::FontId::proportional(text::BODY),
                            pal.text,
                        );
                        ui.painter().text(
                            egui::pos2(rect.right() - 12.0, rect.center().y),
                            Align2::RIGHT_CENTER,
                            &it.hint,
                            egui::FontId::proportional(text::CAPTION),
                            pal.text_muted,
                        );
                        if r.clicked() {
                            outcome = PaletteOutcome::Run(idx);
                        }
                        if r.hovered() && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
                            st.selected = row;
                        }
                    }
                });
        });
    if modal.should_close() && outcome == PaletteOutcome::None {
        outcome = PaletteOutcome::Close;
    }
    outcome
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmOutcome {
    Pending,
    Confirmed,
    Cancelled,
}

/// Modal confirmation. The confirm button is named after the action (never "OK").
pub fn confirm_dialog(
    ctx: &Context,
    pal: &Palette,
    id: &str,
    title: &str,
    body: &str,
    confirm_label: &str,
    destructive: bool,
) -> ConfirmOutcome {
    let mut out = ConfirmOutcome::Pending;
    let modal = egui::Modal::new(Id::unique(id))
        .backdrop_color(Color32::from_black_alpha(if pal.dark { 150 } else { 80 }))
        .frame(
            Frame::new()
                .fill(pal.bg_elevated)
                .stroke(Stroke::new(1.0, pal.border_strong))
                .corner_radius(CornerRadius::same(radius::LG))
                .inner_margin(Margin::same(20)),
        )
        .show(ctx, |ui: &mut Ui| {
            ui.set_width(380.0);
            ui.label(
                RichText::new(title)
                    .size(text::SUBTITLE)
                    .strong()
                    .color(pal.text),
            );
            ui.add_space(space::SM);
            ui.label(RichText::new(body).color(pal.text_secondary));
            ui.add_space(space::LG);
            ui.horizontal(|ui| {
                let kind = if destructive {
                    ButtonKind::Danger
                } else {
                    ButtonKind::Primary
                };
                if widgets::button(ui, pal, kind, None, confirm_label).clicked() {
                    out = ConfirmOutcome::Confirmed;
                }
                if widgets::button(ui, pal, ButtonKind::Secondary, None, "Cancel").clicked() {
                    out = ConfirmOutcome::Cancelled;
                }
            });
        });
    if modal.should_close() && out == ConfirmOutcome::Pending {
        out = ConfirmOutcome::Cancelled;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<PaletteItem> {
        [
            "Go to Dashboard",
            "Go to Data",
            "Open Settings",
            "Toggle theme",
        ]
        .iter()
        .map(|l| PaletteItem {
            label: (*l).into(),
            hint: String::new(),
            icon: Icon::Home,
        })
        .collect()
    }

    #[test]
    fn empty_query_keeps_order() {
        assert_eq!(filter_items(&items(), ""), vec![0, 1, 2, 3]);
    }

    #[test]
    fn substring_beats_subsequence_and_is_case_insensitive() {
        let r = filter_items(&items(), "DATA");
        assert_eq!(r[0], 1);
    }

    #[test]
    fn subsequence_matching_works() {
        assert_eq!(filter_items(&items(), "tgl"), vec![3]); // T-o-G-g-L-e
    }

    #[test]
    fn no_match_returns_empty() {
        assert!(filter_items(&items(), "zzzz").is_empty());
    }
}
