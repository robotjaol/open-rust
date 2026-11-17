//! Design tokens. Every colour in the application comes from a [`Palette`].

use app_config::Accent;
use egui::Color32;

/// Semantic colour role used by badges, toasts, progress bars and status dots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    Neutral,
    Accent,
    Info,
    Success,
    Warning,
    Danger,
}

/// 4-point spacing scale.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

/// Corner radii (px).
pub mod radius {
    pub const SM: u8 = 4;
    pub const MD: u8 = 6;
    pub const LG: u8 = 10;
}

/// Type scale (px).
pub mod text {
    pub const CAPTION: f32 = 11.5;
    pub const BODY: f32 = 13.5;
    pub const SUBTITLE: f32 = 15.0;
    pub const TITLE: f32 = 20.0;
    pub const DISPLAY: f32 = 30.0;
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// A complete colour set for one theme and accent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub dark: bool,
    pub bg_base: Color32,
    pub bg_surface: Color32,
    pub bg_elevated: Color32,
    pub bg_hover: Color32,
    pub bg_active: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    /// Fill for primary buttons and selected states.
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Readable text on top of [`Palette::accent`].
    pub on_accent: Color32,
    /// Accent used for text and icons on normal surfaces.
    pub accent_fg: Color32,
    /// Translucent accent for selection backgrounds.
    pub accent_soft: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
}

fn accent_fill(accent: Accent, dark: bool) -> u32 {
    match (accent, dark) {
        (Accent::Indigo, true) => 0x4B5AE4,
        (Accent::Teal, true) => 0x0E8579,
        (Accent::Amber, true) => 0xE8A23A,
        (Accent::Rose, true) => 0xD03F6E,
        (Accent::Emerald, true) => 0x16925F,
        (Accent::Indigo, false) => 0x4553D6,
        (Accent::Teal, false) => 0x0B7A71,
        (Accent::Amber, false) => 0x9A5800,
        (Accent::Rose, false) => 0xC7305F,
        (Accent::Emerald, false) => 0x13795A,
    }
}

fn accent_text(accent: Accent, dark: bool) -> u32 {
    if !dark {
        return accent_fill(accent, false);
    }
    match accent {
        Accent::Indigo => 0x95A1FF,
        Accent::Teal => 0x3CC6B5,
        Accent::Amber => 0xF0B35C,
        Accent::Rose => 0xFF86A9,
        Accent::Emerald => 0x4CD39A,
    }
}

impl Palette {
    pub fn new(dark: bool, accent: Accent) -> Self {
        let fill = rgb(accent_fill(accent, dark));
        let white = Color32::WHITE;
        let ink = rgb(0x10131A);
        let on_accent = if contrast(white, fill) >= contrast(ink, fill) {
            white
        } else {
            ink
        };
        let a = Color32::from_rgba_unmultiplied(
            fill.r(),
            fill.g(),
            fill.b(),
            if dark { 56 } else { 34 },
        );
        if dark {
            Self {
                dark,
                bg_base: rgb(0x0F1115),
                bg_surface: rgb(0x161920),
                bg_elevated: rgb(0x1C2029),
                bg_hover: rgb(0x222733),
                bg_active: rgb(0x2A3040),
                border: rgb(0x2A2F3B),
                border_strong: rgb(0x3A4152),
                text: rgb(0xE6E8EE),
                text_secondary: rgb(0x9AA1B2),
                text_muted: rgb(0x8189A0),
                accent: fill,
                accent_hover: lighten(fill, 0.12),
                on_accent,
                accent_fg: rgb(accent_text(accent, true)),
                accent_soft: a,
                success: rgb(0x3DD68C),
                warning: rgb(0xF5B544),
                danger: rgb(0xF2706B),
                info: rgb(0x5AA9FF),
            }
        } else {
            Self {
                dark,
                bg_base: rgb(0xF4F5F8),
                bg_surface: rgb(0xFFFFFF),
                bg_elevated: rgb(0xFFFFFF),
                bg_hover: rgb(0xECEEF4),
                bg_active: rgb(0xE1E4EE),
                border: rgb(0xDCDFE8),
                border_strong: rgb(0xC3C8D6),
                text: rgb(0x1B1E27),
                text_secondary: rgb(0x4D5468),
                text_muted: rgb(0x6B7388),
                accent: fill,
                accent_hover: darken(fill, 0.1),
                on_accent,
                accent_fg: rgb(accent_text(accent, false)),
                accent_soft: a,
                success: rgb(0x16794D),
                warning: rgb(0x9A5B00),
                danger: rgb(0xC23030),
                info: rgb(0x1F66C2),
            }
        }
    }

    /// Foreground colour for a [`Tone`].
    pub fn tone(&self, tone: Tone) -> Color32 {
        match tone {
            Tone::Neutral => self.text_secondary,
            Tone::Accent => self.accent_fg,
            Tone::Info => self.info,
            Tone::Success => self.success,
            Tone::Warning => self.warning,
            Tone::Danger => self.danger,
        }
    }

    /// Tinted background for a [`Tone`] (used by badges and toasts).
    pub fn tone_bg(&self, tone: Tone) -> Color32 {
        if tone == Tone::Neutral {
            return self.bg_hover;
        }
        let c = self.tone(tone);
        Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), if self.dark { 36 } else { 30 })
    }

    /// Colour for the i-th series in a categorical chart.
    pub fn series(&self, i: usize) -> Color32 {
        const D: [u32; 6] = [0x7C8BFF, 0x3CC6B5, 0xF0B35C, 0xFF86A9, 0x4CD39A, 0x5AA9FF];
        const L: [u32; 6] = [0x4553D6, 0x0B7A71, 0x9A5800, 0xC7305F, 0x13795A, 0x1F66C2];
        rgb(if self.dark { D[i % 6] } else { L[i % 6] })
    }
}

fn lighten(c: Color32, t: f32) -> Color32 {
    let m = |v: u8| (f32::from(v) + (255.0 - f32::from(v)) * t).round() as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

fn darken(c: Color32, t: f32) -> Color32 {
    let m = |v: u8| (f32::from(v) * (1.0 - t)).round() as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

fn channel(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.x relative luminance.
pub fn luminance(c: Color32) -> f64 {
    0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
}

/// WCAG 2.x contrast ratio between two opaque colours (1.0 ..= 21.0).
pub fn contrast(a: Color32, b: Color32) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn both() -> [Palette; 2] {
        [
            Palette::new(true, Accent::Indigo),
            Palette::new(false, Accent::Indigo),
        ]
    }

    #[test]
    fn contrast_reference_values() {
        assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 1e-6);
        assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn body_text_meets_wcag_aa_on_every_surface() {
        for p in both() {
            for bg in [p.bg_base, p.bg_surface, p.bg_elevated] {
                for (name, fg) in [("text", p.text), ("secondary", p.text_secondary)] {
                    assert!(
                        contrast(fg, bg) >= 4.5,
                        "{name} dark={} {:.2}",
                        p.dark,
                        contrast(fg, bg)
                    );
                }
            }
        }
    }

    #[test]
    fn muted_text_meets_large_text_threshold_on_surfaces() {
        for p in both() {
            for bg in [p.bg_surface, p.bg_elevated] {
                assert!(contrast(p.text_muted, bg) >= 4.5, "muted dark={}", p.dark);
            }
            assert!(
                contrast(p.text_muted, p.bg_base) >= 4.3,
                "muted on base dark={}",
                p.dark
            );
        }
    }

    #[test]
    fn accent_buttons_have_readable_labels_for_every_accent() {
        for dark in [true, false] {
            for a in Accent::ALL {
                let p = Palette::new(dark, a);
                let r = contrast(p.on_accent, p.accent);
                assert!(r >= 4.5, "{a:?} dark={dark} on_accent contrast {r:.2}");
            }
        }
    }

    #[test]
    fn accent_text_is_readable_on_surface() {
        for dark in [true, false] {
            for a in Accent::ALL {
                let p = Palette::new(dark, a);
                let r = contrast(p.accent_fg, p.bg_surface);
                assert!(r >= 4.5, "{a:?} dark={dark} accent_fg contrast {r:.2}");
            }
        }
    }

    #[test]
    fn status_colours_are_readable_on_surface() {
        for p in both() {
            for c in [p.success, p.warning, p.danger, p.info] {
                assert!(contrast(c, p.bg_surface) >= 4.5, "status dark={}", p.dark);
            }
        }
    }

    #[test]
    fn accent_fill_is_distinguishable_from_surface() {
        // WCAG 1.4.11 non-text contrast: 3:1
        for dark in [true, false] {
            for a in Accent::ALL {
                let p = Palette::new(dark, a);
                assert!(contrast(p.accent, p.bg_surface) >= 3.0, "{a:?} dark={dark}");
            }
        }
    }
}
