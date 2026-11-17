//! Vector icons drawn with strokes on a 24x24 grid. No font or image assets are needed,
//! so icons stay crisp at any DPI and recolour with the theme.

use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2, epaint::PathShape, pos2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Home,
    Dashboard,
    Notebook,
    Folder,
    File,
    Table,
    Tasks,
    Terminal,
    Settings,
    Palette,
    Info,
    Alert,
    Check,
    Search,
    Plus,
    Trash,
    Close,
    Sun,
    Moon,
    Bell,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    Download,
    Upload,
    Refresh,
    Command,
    Shield,
    Database,
    Sidebar,
    Play,
    Stop,
    ArrowUp,
    ArrowDown,
    Bolt,
    LifeBuoy,
}

struct Pen<'a> {
    p: &'a Painter,
    origin: Pos2,
    k: f32,
    stroke: Stroke,
}

impl Pen<'_> {
    fn pt(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.origin.x + x * self.k, self.origin.y + y * self.k)
    }

    fn line(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(PathShape::line(v, self.stroke));
    }

    fn poly(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(PathShape::closed_line(v, self.stroke));
    }

    fn circle(&self, x: f32, y: f32, r: f32) {
        self.p.circle_stroke(self.pt(x, y), r * self.k, self.stroke);
    }

    fn dot(&self, x: f32, y: f32, r: f32) {
        self.p
            .circle_filled(self.pt(x, y), r * self.k, self.stroke.color);
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let rect = Rect::from_min_size(self.pt(x, y), Vec2::new(w * self.k, h * self.k));
        self.p
            .rect_stroke(rect, r * self.k, self.stroke, egui::StrokeKind::Middle);
    }
}

impl Icon {
    /// Paint the icon centred in `rect` (the icon fills the shorter side).
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        let size = rect.width().min(rect.height());
        let k = size / 24.0;
        let origin = rect.center() - Vec2::splat(size / 2.0);
        let pen = Pen {
            p: painter,
            origin,
            k,
            stroke: Stroke::new((1.7 * k).max(1.2), color),
        };
        match self {
            Icon::Home => {
                pen.line(&[(3.5, 11.0), (12.0, 3.5), (20.5, 11.0)]);
                pen.line(&[(5.5, 9.5), (5.5, 20.5), (18.5, 20.5), (18.5, 9.5)]);
                pen.line(&[(10.0, 20.5), (10.0, 14.5), (14.0, 14.5), (14.0, 20.5)]);
            }
            Icon::Dashboard => {
                pen.rect(3.5, 3.5, 7.0, 9.0, 1.5);
                pen.rect(13.5, 3.5, 7.0, 5.0, 1.5);
                pen.rect(13.5, 11.5, 7.0, 9.0, 1.5);
                pen.rect(3.5, 15.5, 7.0, 5.0, 1.5);
            }
            Icon::Notebook => {
                pen.rect(5.0, 3.5, 14.0, 17.0, 2.0);
                pen.line(&[(9.0, 8.5), (15.0, 8.5)]);
                pen.line(&[(9.0, 12.0), (15.0, 12.0)]);
                pen.line(&[(9.0, 15.5), (12.5, 15.5)]);
            }
            Icon::Folder => {
                pen.line(&[
                    (3.5, 7.0),
                    (3.5, 18.5),
                    (20.5, 18.5),
                    (20.5, 8.5),
                    (11.5, 8.5),
                    (9.5, 5.5),
                    (3.5, 5.5),
                    (3.5, 7.0),
                ]);
            }
            Icon::File => {
                pen.line(&[
                    (6.0, 3.5),
                    (13.5, 3.5),
                    (18.5, 8.5),
                    (18.5, 20.5),
                    (6.0, 20.5),
                    (6.0, 3.5),
                ]);
                pen.line(&[(13.5, 3.5), (13.5, 8.5), (18.5, 8.5)]);
            }
            Icon::Table => {
                pen.rect(3.5, 4.5, 17.0, 15.0, 2.0);
                pen.line(&[(3.5, 9.5), (20.5, 9.5)]);
                pen.line(&[(3.5, 14.5), (20.5, 14.5)]);
                pen.line(&[(10.0, 9.5), (10.0, 19.5)]);
            }
            Icon::Tasks => {
                pen.line(&[(4.0, 7.0), (6.0, 9.0), (9.5, 5.0)]);
                pen.line(&[(13.0, 7.0), (20.0, 7.0)]);
                pen.line(&[(4.0, 16.0), (6.0, 18.0), (9.5, 14.0)]);
                pen.line(&[(13.0, 16.0), (20.0, 16.0)]);
            }
            Icon::Terminal => {
                pen.rect(3.0, 4.5, 18.0, 15.0, 2.5);
                pen.line(&[(7.0, 9.5), (10.5, 12.0), (7.0, 14.5)]);
                pen.line(&[(12.5, 15.0), (17.0, 15.0)]);
            }
            Icon::Settings => {
                pen.circle(12.0, 12.0, 3.2);
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::FRAC_PI_4;
                    let (s, c) = a.sin_cos();
                    pen.line(&[
                        (12.0 + c * 6.6, 12.0 + s * 6.6),
                        (12.0 + c * 9.0, 12.0 + s * 9.0),
                    ]);
                }
                pen.circle(12.0, 12.0, 6.6);
            }
            Icon::Palette => {
                pen.circle(12.0, 12.0, 8.5);
                pen.dot(8.5, 10.0, 1.2);
                pen.dot(12.0, 7.5, 1.2);
                pen.dot(15.8, 10.0, 1.2);
                pen.dot(8.8, 14.5, 1.2);
            }
            Icon::Info => {
                pen.circle(12.0, 12.0, 8.5);
                pen.line(&[(12.0, 11.0), (12.0, 16.5)]);
                pen.dot(12.0, 7.8, 1.1);
            }
            Icon::Alert => {
                pen.poly(&[(12.0, 3.5), (21.5, 20.0), (2.5, 20.0)]);
                pen.line(&[(12.0, 9.5), (12.0, 14.5)]);
                pen.dot(12.0, 17.3, 1.0);
            }
            Icon::Check => pen.line(&[(4.5, 12.5), (9.5, 17.5), (19.5, 6.5)]),
            Icon::Search => {
                pen.circle(10.5, 10.5, 6.5);
                pen.line(&[(15.5, 15.5), (20.5, 20.5)]);
            }
            Icon::Plus => {
                pen.line(&[(12.0, 5.0), (12.0, 19.0)]);
                pen.line(&[(5.0, 12.0), (19.0, 12.0)]);
            }
            Icon::Trash => {
                pen.line(&[(4.0, 7.0), (20.0, 7.0)]);
                pen.line(&[(9.0, 7.0), (9.0, 4.5), (15.0, 4.5), (15.0, 7.0)]);
                pen.line(&[(6.0, 7.0), (7.0, 20.0), (17.0, 20.0), (18.0, 7.0)]);
                pen.line(&[(10.0, 11.0), (10.0, 16.0)]);
                pen.line(&[(14.0, 11.0), (14.0, 16.0)]);
            }
            Icon::Close => {
                pen.line(&[(6.0, 6.0), (18.0, 18.0)]);
                pen.line(&[(18.0, 6.0), (6.0, 18.0)]);
            }
            Icon::Sun => {
                pen.circle(12.0, 12.0, 4.2);
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::FRAC_PI_4;
                    let (s, c) = a.sin_cos();
                    pen.line(&[
                        (12.0 + c * 7.0, 12.0 + s * 7.0),
                        (12.0 + c * 9.5, 12.0 + s * 9.5),
                    ]);
                }
            }
            Icon::Moon => {
                pen.line(&[
                    (19.5, 14.5),
                    (17.5, 15.0),
                    (13.0, 14.0),
                    (9.5, 10.5),
                    (9.0, 6.0),
                    (9.5, 4.0),
                ]);
                pen.line(&[
                    (9.5, 4.0),
                    (5.5, 6.5),
                    (4.0, 11.5),
                    (6.5, 17.0),
                    (12.0, 19.5),
                    (17.0, 18.5),
                    (19.5, 14.5),
                ]);
            }
            Icon::Bell => {
                pen.line(&[
                    (6.0, 17.0),
                    (6.0, 11.0),
                    (8.0, 6.5),
                    (12.0, 5.0),
                    (16.0, 6.5),
                    (18.0, 11.0),
                    (18.0, 17.0),
                    (6.0, 17.0),
                ]);
                pen.line(&[(4.5, 17.0), (19.5, 17.0)]);
                pen.line(&[(10.0, 20.0), (14.0, 20.0)]);
            }
            Icon::ChevronLeft => pen.line(&[(15.0, 5.5), (8.5, 12.0), (15.0, 18.5)]),
            Icon::ChevronRight => pen.line(&[(9.0, 5.5), (15.5, 12.0), (9.0, 18.5)]),
            Icon::ChevronDown => pen.line(&[(5.5, 9.0), (12.0, 15.5), (18.5, 9.0)]),
            Icon::ChevronUp => pen.line(&[(5.5, 15.0), (12.0, 8.5), (18.5, 15.0)]),
            Icon::Download => {
                pen.line(&[(12.0, 4.0), (12.0, 15.0)]);
                pen.line(&[(7.0, 10.5), (12.0, 15.5), (17.0, 10.5)]);
                pen.line(&[(4.5, 19.5), (19.5, 19.5)]);
            }
            Icon::Upload => {
                pen.line(&[(12.0, 16.0), (12.0, 5.0)]);
                pen.line(&[(7.0, 9.5), (12.0, 4.5), (17.0, 9.5)]);
                pen.line(&[(4.5, 19.5), (19.5, 19.5)]);
            }
            Icon::Refresh => {
                pen.line(&[
                    (19.5, 12.0),
                    (19.0, 9.0),
                    (17.0, 6.5),
                    (14.0, 5.0),
                    (10.5, 5.2),
                    (7.5, 7.0),
                    (5.5, 10.0),
                ]);
                pen.line(&[(19.5, 4.5), (19.5, 9.5), (14.5, 9.5)]);
                pen.line(&[
                    (4.5, 12.0),
                    (5.0, 15.0),
                    (7.0, 17.5),
                    (10.0, 19.0),
                    (13.5, 18.8),
                    (16.5, 17.0),
                    (18.5, 14.0),
                ]);
                pen.line(&[(4.5, 19.5), (4.5, 14.5), (9.5, 14.5)]);
            }
            Icon::Command => {
                pen.rect(8.5, 8.5, 7.0, 7.0, 0.5);
                pen.circle(6.5, 6.5, 2.2);
                pen.circle(17.5, 6.5, 2.2);
                pen.circle(6.5, 17.5, 2.2);
                pen.circle(17.5, 17.5, 2.2);
            }
            Icon::Shield => {
                pen.line(&[
                    (12.0, 3.0),
                    (19.5, 6.0),
                    (19.5, 12.0),
                    (17.0, 16.5),
                    (12.0, 21.0),
                    (7.0, 16.5),
                    (4.5, 12.0),
                    (4.5, 6.0),
                    (12.0, 3.0),
                ]);
                pen.line(&[(8.5, 12.0), (11.0, 14.5), (15.5, 9.5)]);
            }
            Icon::Database => {
                pen.circle(12.0, 6.0, 0.0);
                pen.line(&[
                    (4.5, 6.0),
                    (4.5, 18.0),
                    (7.0, 19.8),
                    (12.0, 20.8),
                    (17.0, 19.8),
                    (19.5, 18.0),
                    (19.5, 6.0),
                ]);
                pen.line(&[
                    (4.5, 6.0),
                    (7.0, 4.2),
                    (12.0, 3.2),
                    (17.0, 4.2),
                    (19.5, 6.0),
                    (17.0, 7.8),
                    (12.0, 8.8),
                    (7.0, 7.8),
                    (4.5, 6.0),
                ]);
                pen.line(&[
                    (4.5, 12.0),
                    (7.0, 13.8),
                    (12.0, 14.8),
                    (17.0, 13.8),
                    (19.5, 12.0),
                ]);
            }
            Icon::Sidebar => {
                pen.rect(3.5, 4.5, 17.0, 15.0, 2.5);
                pen.line(&[(9.5, 4.5), (9.5, 19.5)]);
            }
            Icon::Play => pen.poly(&[(7.0, 4.5), (19.0, 12.0), (7.0, 19.5)]),
            Icon::Stop => pen.rect(6.0, 6.0, 12.0, 12.0, 2.0),
            Icon::ArrowUp => {
                pen.line(&[(12.0, 19.0), (12.0, 5.5)]);
                pen.line(&[(6.5, 11.0), (12.0, 5.5), (17.5, 11.0)]);
            }
            Icon::ArrowDown => {
                pen.line(&[(12.0, 5.0), (12.0, 18.5)]);
                pen.line(&[(6.5, 13.0), (12.0, 18.5), (17.5, 13.0)]);
            }
            Icon::Bolt => pen.poly(&[
                (13.5, 3.0),
                (5.5, 13.5),
                (11.5, 13.5),
                (10.5, 21.0),
                (18.5, 10.0),
                (12.5, 10.0),
            ]),
            Icon::LifeBuoy => {
                pen.circle(12.0, 12.0, 8.5);
                pen.circle(12.0, 12.0, 3.6);
                pen.line(&[(6.0, 6.0), (9.5, 9.5)]);
                pen.line(&[(18.0, 6.0), (14.5, 9.5)]);
                pen.line(&[(6.0, 18.0), (9.5, 14.5)]);
                pen.line(&[(18.0, 18.0), (14.5, 14.5)]);
            }
        }
    }
}

/// Convenience: paint at a point with a given pixel size.
pub fn paint_at(painter: &Painter, icon: Icon, center: Pos2, size: f32, color: Color32) {
    icon.paint(
        painter,
        Rect::from_center_size(center, Vec2::splat(size)),
        color,
    );
}
