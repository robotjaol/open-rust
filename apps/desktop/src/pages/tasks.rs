use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::{AppContext, Page, TaskStatus};
use app_errors::AppError;
use egui::RichText;
use std::time::Duration;
use ui_components::tokens::{space, text};
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Tone};

#[derive(Debug, Clone, Copy)]
pub enum Sample {
    /// Long job with progress that honours cancellation.
    Index,
    /// Runs the SQLite integrity check.
    Verify,
    /// Returns a typed error.
    Fail,
    /// Panics on purpose to show isolation.
    Panic,
}

pub fn spawn_sample(app: &AppContext, kind: Sample) {
    match kind {
        Sample::Index => {
            app.tasks.spawn("Index workspace (sample)", |ctx| {
                for step in 1..=20 {
                    ctx.check_cancelled()?;
                    std::thread::sleep(Duration::from_millis(250));
                    ctx.progress(step as f32 / 20.0, format!("Processing batch {step} of 20"));
                }
                Ok("Indexed 20 batches".into())
            });
        }
        Sample::Verify => {
            let db = app.db.clone();
            app.tasks.spawn("Verify database", move |ctx| {
                ctx.progress(0.4, "Running integrity check");
                db.integrity_check()?;
                Ok("Integrity check passed".into())
            });
        }
        Sample::Fail => {
            app.tasks.spawn("Failing job (sample)", |ctx| {
                ctx.progress(0.5, "About to fail");
                Err(AppError::Validation(
                    "The sample job was asked to fail.".into(),
                ))
            });
        }
        Sample::Panic => {
            app.tasks.spawn("Panicking job (sample)", |_| {
                #[allow(clippy::panic)]
                {
                    panic!("deliberate panic to demonstrate isolation");
                }
            });
        }
    }
}

pub struct Tasks;

impl FeatureModule for Tasks {
    fn page(&self) -> Page {
        Page::Tasks
    }
    fn icon(&self) -> Icon {
        Icon::Tasks
    }
    fn subtitle(&self) -> &'static str {
        "Background work with cancel"
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "Tasks",
            "Jobs run on a small worker pool. The UI stays responsive; each job can report progress and be cancelled.",
        );
        ui.horizontal_wrapped(|ui| {
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Primary,
                Some(Icon::Play),
                "Start long job",
            )
            .clicked()
            {
                spawn_sample(cx.app, Sample::Index);
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Database),
                "Verify database",
            )
            .clicked()
            {
                spawn_sample(cx.app, Sample::Verify);
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Alert),
                "Run failing job",
            )
            .clicked()
            {
                spawn_sample(cx.app, Sample::Fail);
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Bolt),
                "Run panicking job",
            )
            .clicked()
            {
                spawn_sample(cx.app, Sample::Panic);
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Ghost,
                Some(Icon::Trash),
                "Clear finished",
            )
            .clicked()
            {
                cx.app.tasks.clear_finished();
            }
        });
        ui.add_space(space::MD);
        let mut list = cx.app.tasks.snapshot();
        list.reverse();
        if list.is_empty() {
            widgets::empty_state(
                ui,
                &pal,
                Icon::Tasks,
                "No tasks yet",
                "Start a sample job to watch progress and cancellation.",
                |_| {},
            );
            return;
        }
        if list.iter().any(|t| !t.status.is_finished()) {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
        for t in list {
            widgets::card(ui, &pal, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&t.name).strong().color(pal.text));
                    let tone = match t.status {
                        TaskStatus::Queued => Tone::Neutral,
                        TaskStatus::Running => Tone::Accent,
                        TaskStatus::Succeeded => Tone::Success,
                        TaskStatus::Failed => Tone::Danger,
                        TaskStatus::Cancelled => Tone::Warning,
                    };
                    widgets::badge(ui, &pal, tone, t.status.label());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !t.status.is_finished()
                            && widgets::button(
                                ui,
                                &pal,
                                ButtonKind::Secondary,
                                Some(Icon::Stop),
                                "Cancel",
                            )
                            .clicked()
                        {
                            cx.app.tasks.cancel(t.id);
                        }
                    });
                });
                if !t.status.is_finished() {
                    widgets::progress_bar(ui, &pal, t.progress, Tone::Accent, ui.available_width());
                }
                let line = t.outcome.clone().unwrap_or_else(|| t.message.clone());
                ui.label(
                    RichText::new(line)
                        .size(text::CAPTION)
                        .color(pal.text_secondary),
                );
            });
            ui.add_space(space::XS);
        }
    }
}
