use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::{Notification, Page};
use app_storage::Note;
use egui::RichText;
use ui_components::Icon;
use ui_components::overlays::{ConfirmOutcome, confirm_dialog};
use ui_components::tokens::{space, text};
use ui_components::widgets::{self, ButtonKind};

#[derive(Default)]
pub struct Workspace {
    notes: Vec<Note>,
    selected: Option<i64>,
    title: String,
    body: String,
    dirty: bool,
    confirm_delete: bool,
}

impl Workspace {
    fn reload(&mut self, cx: &Cx<'_>) {
        match cx.app.db.list_notes() {
            Ok(n) => self.notes = n,
            Err(e) => cx.fail("Could not load notes", &e),
        }
    }

    fn select(&mut self, id: Option<i64>) {
        self.selected = id;
        self.dirty = false;
        if let Some(n) = id.and_then(|i| self.notes.iter().find(|n| n.id == i)) {
            self.title = n.title.clone();
            self.body = n.body.clone();
        } else {
            self.title.clear();
            self.body.clear();
        }
    }
}

impl FeatureModule for Workspace {
    fn page(&self) -> Page {
        Page::Workspace
    }
    fn icon(&self) -> Icon {
        Icon::Notebook
    }
    fn subtitle(&self) -> &'static str {
        "Notes: the example feature"
    }

    fn on_enter(&mut self, cx: &mut Cx<'_>) {
        self.reload(cx);
        if self.selected.is_none() {
            let first = self.notes.first().map(|n| n.id);
            self.select(first);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(
            ui,
            &pal,
            "Workspace",
            "Notes stored in SQLite. A complete create, read, update, delete flow to copy for your own feature.",
        );
        ui.horizontal(|ui| {
            if widgets::button(ui, &pal, ButtonKind::Primary, Some(Icon::Plus), "New note")
                .clicked()
            {
                match cx.app.db.create_note("Untitled note", "") {
                    Ok(id) => {
                        self.reload(cx);
                        self.select(Some(id));
                    }
                    Err(e) => cx.fail("Could not create note", &e),
                }
            }
            ui.add_enabled_ui(self.selected.is_some(), |ui| {
                if widgets::button(
                    ui,
                    &pal,
                    ButtonKind::Danger,
                    Some(Icon::Trash),
                    "Delete note",
                )
                .clicked()
                {
                    self.confirm_delete = true;
                }
            });
        });
        ui.add_space(space::SM);
        if self.notes.is_empty() {
            widgets::empty_state(
                ui,
                &pal,
                Icon::Notebook,
                "No notes yet",
                "Create your first note to see storage in action.",
                |_| {},
            );
        } else {
            ui.columns(2, |c| {
                c[0].set_max_width(340.0);
                let mut pick = None;
                widgets::card(&mut c[0], &pal, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| {
                            for n in &self.notes {
                                let sel = self.selected == Some(n.id);
                                let r = ui.add(
                                    egui::Button::new(RichText::new(&n.title).color(pal.text))
                                        .selected(sel)
                                        .min_size(egui::vec2(ui.available_width(), 30.0)),
                                );
                                if r.clicked() {
                                    pick = Some(n.id);
                                }
                            }
                        });
                });
                if let Some(id) = pick {
                    self.select(Some(id));
                }
                widgets::card(&mut c[1], &pal, |ui| {
                    if self.selected.is_none() {
                        ui.label(RichText::new("Select a note.").color(pal.text_muted));
                        return;
                    }
                    ui.label(
                        RichText::new("Title")
                            .size(text::CAPTION)
                            .color(pal.text_secondary),
                    );
                    let t = ui.add(
                        egui::TextEdit::singleline(&mut self.title).desired_width(f32::INFINITY),
                    );
                    ui.add_space(space::SM);
                    ui.label(
                        RichText::new("Body")
                            .size(text::CAPTION)
                            .color(pal.text_secondary),
                    );
                    let b = ui.add(
                        egui::TextEdit::multiline(&mut self.body)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY),
                    );
                    self.dirty |= t.changed() || b.changed();
                    ui.add_space(space::SM);
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(self.dirty, |ui| {
                            if widgets::button(
                                ui,
                                &pal,
                                ButtonKind::Primary,
                                Some(Icon::Check),
                                "Save changes",
                            )
                            .clicked()
                                && let Some(id) = self.selected
                            {
                                match cx.app.db.update_note(id, &self.title, &self.body) {
                                    Ok(()) => {
                                        self.dirty = false;
                                        cx.notify(Notification::success(
                                            "Note saved",
                                            self.title.clone(),
                                        ));
                                        self.reload(cx);
                                    }
                                    Err(e) => cx.fail("Could not save note", &e),
                                }
                            }
                        });
                        if self.dirty {
                            widgets::badge(
                                ui,
                                &pal,
                                ui_components::Tone::Warning,
                                "Unsaved changes",
                            );
                        }
                    });
                });
            });
        }
        if self.confirm_delete {
            match confirm_dialog(
                ui.ctx(),
                &pal,
                "delete-note",
                "Delete this note?",
                "The note is removed from the database. This cannot be undone.",
                "Delete note",
                true,
            ) {
                ConfirmOutcome::Confirmed => {
                    self.confirm_delete = false;
                    if let Some(id) = self.selected {
                        match cx.app.db.delete_note(id) {
                            Ok(_) => {
                                self.reload(cx);
                                let first = self.notes.first().map(|n| n.id);
                                self.select(first);
                                cx.notify(Notification::info("Note deleted", ""));
                            }
                            Err(e) => cx.fail("Could not delete note", &e),
                        }
                    }
                }
                ConfirmOutcome::Cancelled => self.confirm_delete = false,
                ConfirmOutcome::Pending => {}
            }
        }
    }
}
