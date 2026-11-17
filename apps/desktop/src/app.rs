//! Application shell: window chrome, navigation, shortcuts, overlays and the event loop.

use crate::cx::Cx;
use crate::module::{self, FeatureModule};
use crate::screenshot::{Plan, Step};
use app_config::{Appearance, ThemeMode};
use app_core::{
    Action, AppContext, AppEvent, AppState, EventReceiver, Level, NavGroup, Notification, Page,
};
use egui::{
    Align, Context, CornerRadius, Frame, Key, KeyboardShortcut, Layout, Margin, Modifiers,
    RichText, Sense, Stroke,
};
use std::collections::VecDeque;
use ui_components::overlays::{self, PaletteItem, PaletteOutcome, PaletteState, ToastHost};
use ui_components::tokens::{radius, space, text};
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Palette, Tone, theme};

const SIDEBAR_WIDE: f32 = 224.0;
const SIDEBAR_NARROW: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Go(Page),
    ToggleTheme,
    ToggleSidebar,
    SampleTask,
    ClearToasts,
}

pub struct OpenRustApp {
    ctx: AppContext,
    events: EventReceiver,
    state: AppState,
    modules: Vec<Box<dyn FeatureModule>>,
    toasts: ToastHost,
    palette: PaletteState,
    commands: Vec<(Command, PaletteItem)>,
    dialog_results: Vec<(app_core::DialogPurpose, Option<std::path::PathBuf>)>,
    applied: Option<Appearance>,
    frame_ms: VecDeque<f32>,
    plan: Option<Plan>,
    last_save: f64,
    config_dirty: bool,
    entered: Option<Page>,
    closing: bool,
}

impl OpenRustApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        ctx: AppContext,
        events: EventReceiver,
        plan: Option<Plan>,
        start: Option<Page>,
    ) -> Self {
        let egui_ctx = cc.egui_ctx.clone();
        ctx.events.set_waker(move || egui_ctx.request_repaint());
        let start_page = start.unwrap_or(if ctx.previous_crash.is_some() {
            Page::Recovery
        } else if ctx.config.general.show_welcome {
            Page::Welcome
        } else {
            Page::Dashboard
        });
        let start_dir = if plan.is_some() {
            demo_dir(&ctx)
        } else {
            app_platform::FileService::home_dir()
        };
        let modules = module::registry(start_dir);
        let mut commands: Vec<(Command, PaletteItem)> = modules
            .iter()
            .map(|m| {
                (
                    Command::Go(m.page()),
                    PaletteItem {
                        label: format!("Go to {}", m.page().title()),
                        hint: m.subtitle().into(),
                        icon: m.icon(),
                    },
                )
            })
            .collect();
        commands.extend([
            (
                Command::ToggleTheme,
                PaletteItem {
                    label: "Toggle light / dark theme".into(),
                    hint: "Ctrl+Shift+L".into(),
                    icon: Icon::Sun,
                },
            ),
            (
                Command::ToggleSidebar,
                PaletteItem {
                    label: "Toggle sidebar".into(),
                    hint: "Ctrl+B".into(),
                    icon: Icon::Sidebar,
                },
            ),
            (
                Command::SampleTask,
                PaletteItem {
                    label: "Run sample background task".into(),
                    hint: "Tasks".into(),
                    icon: Icon::Play,
                },
            ),
            (
                Command::ClearToasts,
                PaletteItem {
                    label: "Dismiss all notifications".into(),
                    hint: "".into(),
                    icon: Icon::Bell,
                },
            ),
        ]);
        let mut app = Self {
            ctx,
            events,
            state: AppState::new(start_page),
            modules,
            toasts: ToastHost::default(),
            palette: PaletteState::default(),
            commands,
            dialog_results: Vec::new(),
            applied: None,
            frame_ms: VecDeque::with_capacity(130),
            plan,
            last_save: 0.0,
            config_dirty: false,
            entered: None,
            closing: false,
        };
        if app.plan.is_some() {
            use crate::pages::tasks::{Sample, spawn_sample};
            spawn_sample(&app.ctx, Sample::Verify);
            spawn_sample(&app.ctx, Sample::Fail);
            spawn_sample(&app.ctx, Sample::Index);
        }
        if let Some(c) = &app.ctx.previous_crash {
            let n = Notification::warning(
                "Previous session ended unexpectedly",
                format!(
                    "Crash report from {}",
                    app_errors::time::format_utc(c.unix_time)
                ),
            );
            app.ctx.notifications.push(n);
        }
        app
    }

    fn pal(&self, ctx: &Context) -> Palette {
        theme::current_palette(ctx, self.ctx.config.appearance.accent)
    }

    fn drain_events(&mut self, ctx: &Context) {
        for ev in self.events.drain() {
            match ev {
                AppEvent::Notify(n) => {
                    let stored = self.ctx.notifications.push(n);
                    let tone = match stored.level {
                        Level::Info => Tone::Info,
                        Level::Success => Tone::Success,
                        Level::Warning => Tone::Warning,
                        Level::Error => Tone::Danger,
                    };
                    if self.plan.is_none() {
                        self.toasts
                            .push(ctx, stored.id, tone, &stored.title, &stored.message);
                    }
                }
                AppEvent::DialogResult { purpose, path } => {
                    self.dialog_results.push((purpose, path))
                }
                AppEvent::Error(report) => {
                    self.ctx.notifications.push(Notification::error(
                        report.code.clone(),
                        report.user_message,
                    ));
                }
                AppEvent::TaskChanged => {}
            }
        }
    }

    fn run_command(&mut self, c: Command, ctx: &Context) {
        match c {
            Command::Go(p) => self.state.reduce(Action::Navigate(p)),
            Command::ToggleTheme => self.toggle_theme(ctx),
            Command::ToggleSidebar => self.state.reduce(Action::ToggleSidebar),
            Command::SampleTask => {
                crate::pages::tasks::spawn_sample(&self.ctx, crate::pages::tasks::Sample::Index);
                self.state.reduce(Action::Navigate(Page::Tasks));
            }
            Command::ClearToasts => self.toasts = ToastHost::default(),
        }
    }

    fn toggle_theme(&mut self, ctx: &Context) {
        let dark_now = ctx.global_style().visuals.dark_mode;
        self.ctx.config.appearance.theme = if dark_now {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        self.config_dirty = true;
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        let k = |m: Modifiers, key: Key| KeyboardShortcut::new(m, key);
        let (palette, sidebar, back, fwd, settings, theme_toggle, esc) = ctx.input_mut(|i| {
            (
                i.consume_shortcut(&k(Modifiers::COMMAND, Key::K)),
                i.consume_shortcut(&k(Modifiers::COMMAND, Key::B)),
                i.consume_shortcut(&k(Modifiers::ALT, Key::ArrowLeft)),
                i.consume_shortcut(&k(Modifiers::ALT, Key::ArrowRight)),
                i.consume_shortcut(&k(Modifiers::COMMAND, Key::Comma)),
                i.consume_shortcut(&k(Modifiers::COMMAND | Modifiers::SHIFT, Key::L)),
                i.consume_key(Modifiers::NONE, Key::Escape),
            )
        });
        if palette {
            self.palette.reset();
            self.state.reduce(if self.state.palette_open {
                Action::ClosePalette
            } else {
                Action::OpenPalette
            });
        }
        if sidebar {
            self.state.reduce(Action::ToggleSidebar);
        }
        if back {
            self.state.reduce(Action::Back);
        }
        if fwd {
            self.state.reduce(Action::Forward);
        }
        if settings {
            self.state.reduce(Action::Navigate(Page::Settings));
        }
        if theme_toggle {
            self.toggle_theme(ctx);
        }
        if esc && self.state.notifications_open {
            self.state.reduce(Action::CloseNotifications);
        }
    }

    fn track_window(&mut self, ctx: &Context) {
        let (inner, outer, maximized) = ctx.input(|i| {
            let v = i.viewport();
            (v.inner_rect, v.outer_rect, v.maximized.unwrap_or(false))
        });
        let w = &mut self.ctx.config.window;
        let mut changed = false;
        if w.maximized != maximized {
            w.maximized = maximized;
            changed = true;
        }
        if !maximized {
            if let Some(r) = inner
                && ((w.width - r.width()).abs() > 1.0 || (w.height - r.height()).abs() > 1.0)
            {
                w.width = r.width();
                w.height = r.height();
                changed = true;
            }
            if let Some(r) = outer {
                let (x, y) = (r.min.x, r.min.y);
                if w.x.is_none_or(|v| (v - x).abs() > 1.0)
                    || w.y.is_none_or(|v| (v - y).abs() > 1.0)
                {
                    w.x = Some(x);
                    w.y = Some(y);
                    changed = true;
                }
            }
        }
        self.config_dirty |= changed;
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let collapsed = self.state.sidebar_collapsed;
        let width = if collapsed {
            SIDEBAR_NARROW
        } else {
            SIDEBAR_WIDE
        };
        egui::Panel::left("sidebar")
            .exact_size(width)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(pal.bg_surface)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::symmetric(10, 12)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::hover());
                    ui.painter()
                        .rect_filled(r, CornerRadius::same(radius::MD), pal.accent);
                    ui_components::icons::paint_at(
                        ui.painter(),
                        Icon::Terminal,
                        r.center(),
                        18.0,
                        pal.on_accent,
                    );
                    if !collapsed {
                        ui.label(
                            RichText::new("Open Rust")
                                .size(text::SUBTITLE)
                                .strong()
                                .color(pal.text),
                        );
                    }
                });
                let mut groups = [NavGroup::Overview, NavGroup::Work, NavGroup::System].into_iter();
                let mut target = None;
                for g in groups.by_ref() {
                    widgets::nav_group_label(
                        ui,
                        pal,
                        match g {
                            NavGroup::Overview => "Overview",
                            NavGroup::Work => "Work",
                            NavGroup::System => "System",
                        },
                        collapsed,
                    );
                    for m in self.modules.iter().filter(|m| m.page().group() == g) {
                        let page = m.page();
                        if page == Page::Recovery
                            && self.ctx.previous_crash.is_none()
                            && self.state.page != Page::Recovery
                        {
                            continue;
                        }
                        let badge = match page {
                            Page::Tasks => {
                                let n = self.ctx.tasks.running_count();
                                (n > 0).then(|| n.to_string())
                            }
                            _ => None,
                        };
                        if widgets::nav_item(
                            ui,
                            pal,
                            m.icon(),
                            page.title(),
                            self.state.page == page,
                            collapsed,
                            badge.as_deref(),
                        )
                        .clicked()
                        {
                            target = Some(page);
                        }
                    }
                }
                if let Some(p) = target {
                    self.state.reduce(Action::Navigate(p));
                }
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    let icon = if collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronLeft
                    };
                    if widgets::icon_button(
                        ui,
                        pal,
                        icon,
                        if collapsed {
                            "Expand sidebar (Ctrl+B)"
                        } else {
                            "Collapse sidebar (Ctrl+B)"
                        },
                    )
                    .clicked()
                    {
                        self.state.reduce(Action::ToggleSidebar);
                    }
                });
            });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        egui::Panel::top("toolbar")
            .exact_size(48.0)
            .frame(
                Frame::new()
                    .fill(pal.bg_base)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_enabled_ui(self.state.can_go_back(), |ui| {
                        if widgets::icon_button(ui, pal, Icon::ChevronLeft, "Back (Alt+Left)")
                            .clicked()
                        {
                            self.state.reduce(Action::Back);
                        }
                    });
                    ui.add_enabled_ui(self.state.can_go_forward(), |ui| {
                        if widgets::icon_button(ui, pal, Icon::ChevronRight, "Forward (Alt+Right)")
                            .clicked()
                        {
                            self.state.reduce(Action::Forward);
                        }
                    });
                    ui.add_space(space::SM);
                    ui.label(RichText::new(self.state.page.group_label()).color(pal.text_muted));
                    ui.label(RichText::new("/").color(pal.text_muted));
                    ui.label(
                        RichText::new(self.state.page.title())
                            .strong()
                            .color(pal.text),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let unread = self.ctx.notifications.unread();
                        let resp = widgets::icon_button(ui, pal, Icon::Bell, "Notifications");
                        if unread > 0 {
                            let c = resp.rect.right_top() + egui::vec2(-5.0, 7.0);
                            ui.painter().circle_filled(c, 5.0, pal.danger);
                        }
                        if resp.clicked() {
                            self.state.reduce(Action::ToggleNotifications);
                            self.ctx.notifications.mark_all_read();
                        }
                        let dark = ui.ctx().global_style().visuals.dark_mode;
                        if widgets::icon_button(
                            ui,
                            pal,
                            if dark { Icon::Sun } else { Icon::Moon },
                            "Toggle theme (Ctrl+Shift+L)",
                        )
                        .clicked()
                        {
                            self.toggle_theme(ui.ctx());
                        }
                        // Search / command palette launcher
                        let (rect, r) =
                            ui.allocate_exact_size(egui::vec2(260.0, 30.0), Sense::click());
                        r.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Button,
                                true,
                                "Open command palette",
                            )
                        });
                        let bg = if r.hovered() {
                            pal.bg_hover
                        } else {
                            pal.bg_surface
                        };
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(radius::MD),
                            bg,
                            Stroke::new(1.0, pal.border_strong),
                            egui::StrokeKind::Inside,
                        );
                        ui_components::icons::paint_at(
                            ui.painter(),
                            Icon::Search,
                            egui::pos2(rect.left() + 16.0, rect.center().y),
                            15.0,
                            pal.text_muted,
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 32.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Search or run a command",
                            egui::FontId::proportional(text::BODY),
                            pal.text_muted,
                        );
                        let mut kb = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect.shrink2(egui::vec2(6.0, 4.0)))
                                .layout(Layout::right_to_left(Align::Center)),
                        );
                        widgets::kbd(&mut kb, pal, "Ctrl K");
                        if r.clicked() {
                            self.palette.reset();
                            self.state.reduce(Action::OpenPalette);
                        }
                    });
                });
            });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        egui::Panel::bottom("status")
            .exact_size(26.0)
            .frame(
                Frame::new()
                    .fill(pal.bg_surface)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::symmetric(12, 0)),
            )
            .show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let running = self.ctx.tasks.running_count();
                    if running > 0 {
                        widgets::status(
                            ui,
                            pal,
                            Tone::Accent,
                            &format!("{running} task(s) running"),
                        );
                    } else {
                        widgets::status(ui, pal, Tone::Success, "Ready");
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "v{}  |  {:?}",
                                env!("CARGO_PKG_VERSION"),
                                self.ctx.environment
                            ))
                            .size(text::CAPTION)
                            .color(pal.text_muted),
                        );
                        ui.label(
                            RichText::new(format!(
                                "SQLite schema v{}",
                                self.ctx.db.schema_version().unwrap_or(0)
                            ))
                            .size(text::CAPTION)
                            .color(pal.text_muted),
                        );
                    });
                });
            });
    }

    fn notification_panel(&mut self, ctx: &Context, pal: &Palette) {
        if !self.state.notifications_open {
            return;
        }
        let mut close = false;
        let mut clear = false;
        let area = egui::Area::new(egui::Id::unique("notification-panel"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 56.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(pal.bg_elevated)
                    .stroke(Stroke::new(1.0, pal.border_strong))
                    .corner_radius(CornerRadius::same(radius::LG))
                    .inner_margin(Margin::same(12))
                    .shadow(ctx.global_style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_width(340.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Notifications")
                                    .strong()
                                    .size(text::SUBTITLE)
                                    .color(pal.text),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::icon_button(ui, pal, Icon::Close, "Close notifications")
                                    .clicked()
                                {
                                    close = true;
                                }
                                if widgets::icon_button(
                                    ui,
                                    pal,
                                    Icon::Trash,
                                    "Clear all notifications",
                                )
                                .clicked()
                                {
                                    clear = true;
                                }
                            });
                        });
                        ui.separator();
                        if self.ctx.notifications.is_empty() {
                            widgets::empty_state(
                                ui,
                                pal,
                                Icon::Bell,
                                "You are all caught up",
                                "Events from background work appear here.",
                                |_| {},
                            );
                        }
                        egui::ScrollArea::vertical()
                            .max_height(380.0)
                            .show(ui, |ui| {
                                for n in self.ctx.notifications.history() {
                                    let tone = match n.level {
                                        Level::Info => Tone::Info,
                                        Level::Success => Tone::Success,
                                        Level::Warning => Tone::Warning,
                                        Level::Error => Tone::Danger,
                                    };
                                    ui.horizontal_top(|ui| {
                                        let (r, _) = ui.allocate_exact_size(
                                            egui::vec2(20.0, 20.0),
                                            Sense::hover(),
                                        );
                                        let icon = match n.level {
                                            Level::Success => Icon::Check,
                                            Level::Warning | Level::Error => Icon::Alert,
                                            Level::Info => Icon::Info,
                                        };
                                        ui_components::icons::paint_at(
                                            ui.painter(),
                                            icon,
                                            r.center(),
                                            16.0,
                                            pal.tone(tone),
                                        );
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&n.title).color(pal.text));
                                            if !n.message.is_empty() {
                                                ui.label(
                                                    RichText::new(&n.message)
                                                        .size(text::CAPTION)
                                                        .color(pal.text_secondary),
                                                );
                                            }
                                            ui.label(
                                                RichText::new(
                                                    app_errors::time::format_clock(n.unix_time)
                                                        + " UTC",
                                                )
                                                .size(10.5)
                                                .color(pal.text_muted),
                                            );
                                        });
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                    });
            });
        // Click outside closes the panel.
        if ctx.input(|i| i.pointer.any_pressed())
            && let Some(p) = ctx.input(|i| i.pointer.interact_pos())
            && !area.response.rect.contains(p)
            && p.y > 52.0
        {
            close = true;
        }
        if clear {
            self.ctx.notifications.clear();
        }
        if close {
            self.state.reduce(Action::CloseNotifications);
        }
    }
}

fn demo_dir(ctx: &AppContext) -> std::path::PathBuf {
    let dir = ctx.paths.data_dir.join("demo-files");
    let _ = std::fs::create_dir_all(dir.join("reports"));
    let _ = std::fs::create_dir_all(dir.join("assets"));
    let files: [(&str, &str); 5] = [
        (
            "README.md",
            "# Demo workspace\n\nThis folder is created for screenshots.\nBrowse it from the Files page.\n",
        ),
        (
            "config.example.toml",
            "schema_version = 2\n\n[appearance]\ntheme = \"dark\"\naccent = \"indigo\"\n",
        ),
        (
            "reports/2026-q3.csv",
            "id,name,total\n1,Gateway,1200.50\n2,Pipeline,830.00\n",
        ),
        (
            "reports/notes.txt",
            "Quarterly review notes.\nKeep storage migrations append-only.\n",
        ),
        (
            "assets/palette.json",
            "{\n  \"accent\": \"#4B5AE4\",\n  \"surface\": \"#161920\"\n}\n",
        ),
    ];
    for (name, body) in files {
        let _ = std::fs::write(dir.join(name), body);
    }
    dir
}

trait PageExt {
    fn group_label(self) -> &'static str;
}

impl PageExt for Page {
    fn group_label(self) -> &'static str {
        match self.group() {
            NavGroup::Overview => "Overview",
            NavGroup::Work => "Work",
            NavGroup::System => "System",
        }
    }
}

impl eframe::App for OpenRustApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(dt) = Some(ctx.input(|i| i.unstable_dt * 1000.0)).filter(|d| *d < 250.0) {
            if self.frame_ms.len() == 120 {
                self.frame_ms.pop_front();
            }
            self.frame_ms.push_back(dt);
        }
        self.drain_events(ctx);
        // Screenshot automation drives page and theme.
        let mut done = false;
        if let Some(plan) = &mut self.plan {
            match plan.tick(ctx) {
                Step::Show(page, theme) => {
                    if self.state.page != page {
                        self.state.reduce(Action::Navigate(page));
                    }
                    if self.ctx.config.appearance.theme != theme {
                        self.ctx.config.appearance.theme = theme;
                    }
                }
                Step::Done => done = true,
            }
        }
        if done {
            if let Some(plan) = self.plan.take() {
                println!(
                    "saved {} screenshots to {}",
                    plan.saved_files.len(),
                    plan.dir.display()
                );
            }
            self.closing = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.applied.as_ref() != Some(&self.ctx.config.appearance) {
            theme::apply(ctx, &self.ctx.config.appearance);
            self.applied = Some(self.ctx.config.appearance.clone());
        }
        if self.plan.is_none() {
            self.handle_shortcuts(ctx);
            self.track_window(ctx);
        }
        let now = ctx.input(|i| i.time);
        if self.config_dirty && now - self.last_save > 1.5 && self.plan.is_none() {
            if let Err(e) = self.ctx.save_config() {
                tracing::warn!(error = %e, "could not save settings");
            }
            self.config_dirty = false;
            self.last_save = now;
        }
        if self.config_dirty {
            ctx.request_repaint_after(std::time::Duration::from_millis(1600));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let pal = self.pal(&ctx);
        self.sidebar(ui, &pal);
        self.toolbar(ui, &pal);
        self.status_bar(ui, &pal);

        let page = self.state.page;
        let idx = self.modules.iter().position(|m| m.page() == page);
        let mut nav_actions = Vec::new();
        let mut cfg_dirty = false;
        let mut appearance_dirty = false;
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(pal.bg_base)
                    .inner_margin(Margin::symmetric(28, 16)),
            )
            .show(ui, |ui| {
                let Some(i) = idx else { return };
                let mut cx = Cx {
                    pal,
                    app: &mut self.ctx,
                    nav: Vec::new(),
                    dialog_results: &mut self.dialog_results,
                    frame_ms: &self.frame_ms,
                    config_dirty: false,
                    appearance_dirty: false,
                };
                if self.entered != Some(page) {
                    self.modules[i].on_enter(&mut cx);
                    self.entered = Some(page);
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(1180.0));
                        self.modules[i].ui(ui, &mut cx);
                    });
                nav_actions = std::mem::take(&mut cx.nav);
                cfg_dirty = cx.config_dirty;
                appearance_dirty = cx.appearance_dirty;
            });
        for a in nav_actions {
            self.state.reduce(a);
        }
        self.config_dirty |= cfg_dirty || appearance_dirty;

        self.notification_panel(&ctx, &pal);
        self.toasts.show(&ctx, &pal);

        if self.state.palette_open {
            let items: Vec<PaletteItem> = self.commands.iter().map(|c| c.1.clone()).collect();
            match overlays::command_palette(&ctx, &pal, &mut self.palette, &items) {
                PaletteOutcome::Run(i) => {
                    let cmd = self.commands[i].0;
                    self.state.reduce(Action::ClosePalette);
                    self.run_command(cmd, &ctx);
                }
                PaletteOutcome::Close => self.state.reduce(Action::ClosePalette),
                PaletteOutcome::None => {}
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.plan.is_none() && !self.closing {
            self.ctx.shutdown();
        } else {
            // Screenshot runs must not overwrite the user's saved settings.
            let _ = self
                .ctx
                .tasks
                .shutdown(std::time::Duration::from_millis(500));
        }
    }
}

#[allow(dead_code)]
fn _button_kind_is_used(_: ButtonKind) {}
