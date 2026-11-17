//! Small, dependency-free charts drawn directly with the egui painter.

use crate::tokens::Palette;
use egui::{
    Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Ui, Vec2, epaint::PathShape, pos2, vec2,
};

/// Line sparkline with a soft area fill. Empty or constant data renders a flat line.
pub fn sparkline(ui: &mut Ui, pal: &Palette, data: &[f32], size: Vec2, color: Color32) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, "trend chart"));
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(6), pal.bg_base);
    let inner = rect.shrink2(vec2(8.0, 8.0));
    if data.len() < 2 {
        p.line_segment(
            [inner.left_center(), inner.right_center()],
            Stroke::new(1.5, color),
        );
        return;
    }
    let (lo, hi) = data
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
    let span = (hi - lo).max(f32::EPSILON);
    let pts: Vec<Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            pos2(
                inner.left() + inner.width() * i as f32 / (data.len() - 1) as f32,
                inner.bottom() - inner.height() * ((v - lo) / span),
            )
        })
        .collect();
    let mut fill = pts.clone();
    fill.push(inner.right_bottom());
    fill.push(inner.left_bottom());
    p.add(PathShape::convex_polygon(
        fill,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28),
        Stroke::NONE,
    ));
    p.add(PathShape::line(pts, Stroke::new(2.0, color)));
}

/// Horizontal bar chart: label on the left, bar, value on the right.
pub fn bar_chart(ui: &mut Ui, pal: &Palette, rows: &[(String, f32)], width: f32) {
    let max = rows
        .iter()
        .map(|r| r.1)
        .fold(0.0_f32, f32::max)
        .max(f32::EPSILON);
    for (i, (label, v)) in rows.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.add_sized(
                [110.0, 18.0],
                egui::Label::new(egui::RichText::new(label).color(pal.text_secondary)).truncate(),
            );
            let bar_w = (width - 110.0 - 56.0).max(40.0);
            let (rect, resp) = ui.allocate_exact_size(vec2(bar_w, 12.0), Sense::hover());
            resp.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Label, true, format!("{label}: {v}"))
            });
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), pal.bg_hover);
            let mut f = rect;
            f.set_width((rect.width() * (v / max)).max(6.0));
            ui.painter()
                .rect_filled(f, CornerRadius::same(6), pal.series(i));
            ui.label(egui::RichText::new(format!("{v:.0}")).color(pal.text));
        });
    }
}

/// Donut chart with a centre label. Segments are `(label, value, colour)`.
pub fn donut(
    ui: &mut Ui,
    pal: &Palette,
    segments: &[(String, f32, Color32)],
    diameter: f32,
    center_text: &str,
) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, "distribution chart"));
    let total: f32 = segments.iter().map(|s| s.1).sum();
    let c = rect.center();
    let r = diameter / 2.0 - 6.0;
    let thickness = 14.0;
    let p = ui.painter();
    p.circle_stroke(c, r, Stroke::new(thickness, pal.bg_hover));
    if total > 0.0 {
        let mut start = -std::f32::consts::FRAC_PI_2;
        for (_, v, color) in segments {
            let sweep = std::f32::consts::TAU * v / total;
            let steps = ((sweep / 0.05).ceil() as usize).max(2);
            let gap = if segments.len() > 1 { 0.03 } else { 0.0 };
            let pts: Vec<Pos2> = (0..=steps)
                .map(|i| {
                    let a = start + gap + (sweep - 2.0 * gap).max(0.01) * i as f32 / steps as f32;
                    pos2(c.x + r * a.cos(), c.y + r * a.sin())
                })
                .collect();
            p.add(PathShape::line(pts, Stroke::new(thickness, *color)));
            start += sweep;
        }
    }
    p.text(
        c,
        egui::Align2::CENTER_CENTER,
        center_text,
        egui::FontId::proportional(20.0),
        pal.text,
    );
    let _ = Rect::NOTHING;
}
