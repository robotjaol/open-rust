use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::{DialogPurpose, Notification, Page};
use app_platform::dialogs::DialogKind;
use app_storage::{Item, ItemQuery, ItemSort, NewItem};
use egui::RichText;
use egui_extras::{Column, TableBuilder};
use ui_components::overlays::{ConfirmOutcome, confirm_dialog};
use ui_components::tokens::{space, text};
use ui_components::widgets::{self, ButtonKind};
use ui_components::{Icon, Tone};

const PAGE_SIZE: usize = 15;

pub struct DataTable {
    rows: Vec<Item>,
    total: u64,
    search: String,
    status: usize,
    sort: ItemSort,
    descending: bool,
    page: usize,
    stale: bool,
    adding: bool,
    form: NewItem,
    delete: Option<(i64, String)>,
}

impl Default for DataTable {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            total: 0,
            search: String::new(),
            status: 0,
            sort: ItemSort::Name,
            descending: false,
            page: 0,
            stale: true,
            adding: false,
            form: NewItem {
                name: String::new(),
                category: "General".into(),
                status: "active".into(),
                amount: 0.0,
            },
            delete: None,
        }
    }
}

const STATUSES: [&str; 4] = ["All", "active", "paused", "archived"];

impl DataTable {
    fn load(&mut self, cx: &Cx<'_>) {
        let q = ItemQuery {
            search: self.search.clone(),
            status: (self.status > 0).then(|| STATUSES[self.status].to_string()),
            sort: self.sort,
            descending: self.descending,
            limit: PAGE_SIZE,
            offset: self.page * PAGE_SIZE,
        };
        match cx.app.db.query_items(&q) {
            Ok((rows, total)) => {
                self.rows = rows;
                self.total = total;
            }
            Err(e) => cx.fail("Could not load records", &e),
        }
        self.stale = false;
    }

    fn header(
        &mut self,
        ui: &mut egui::Ui,
        pal: &ui_components::Palette,
        label: &str,
        sort: ItemSort,
    ) {
        let arrow = if self.sort == sort {
            if self.descending { " v" } else { " ^" }
        } else {
            ""
        };
        if ui
            .add(
                egui::Button::new(
                    RichText::new(format!("{label}{arrow}"))
                        .strong()
                        .color(pal.text_secondary),
                )
                .frame(false),
            )
            .clicked()
        {
            if self.sort == sort {
                self.descending = !self.descending;
            } else {
                self.sort = sort;
                self.descending = false;
            }
            self.page = 0;
            self.stale = true;
        }
    }
}

impl FeatureModule for DataTable {
    fn page(&self) -> Page {
        Page::Data
    }
    fn icon(&self) -> Icon {
        Icon::Table
    }
    fn subtitle(&self) -> &'static str {
        "Sort, filter, import and export"
    }
    fn on_enter(&mut self, _cx: &mut Cx<'_>) {
        self.stale = true;
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        // Dialog answers become background tasks.
        for (purpose, kind) in [
            (DialogPurpose::ExportCsv, "csv"),
            (DialogPurpose::ExportJson, "json"),
        ] {
            if let Some(Some(path)) = cx.take_dialog(purpose) {
                let db = cx.app.db.clone();
                cx.app.tasks.spawn(format!("Export {kind}"), move |ctx| {
                    ctx.progress(0.3, "Reading records");
                    let text = if kind == "csv" {
                        db.export_items_csv()?
                    } else {
                        db.export_items_json()?
                    };
                    ctx.check_cancelled()?;
                    ctx.progress(0.8, "Writing file");
                    app_security::write_atomic(&path, text.as_bytes())?;
                    Ok(format!(
                        "Exported to {}",
                        path.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    ))
                });
            }
        }
        if let Some(Some(path)) = cx.take_dialog(DialogPurpose::ImportJson) {
            let db = cx.app.db.clone();
            let max = u64::from(cx.app.config.files.max_file_size_mb) * 1024 * 1024;
            cx.app.tasks.spawn("Import JSON", move |ctx| {
                ctx.progress(0.2, "Reading file");
                let bytes = app_security::read_limited(&path, max)?;
                let text = String::from_utf8(bytes).map_err(|_| {
                    app_errors::AppError::Validation("The file is not valid UTF-8 text.".into())
                })?;
                ctx.check_cancelled()?;
                ctx.progress(0.6, "Validating and inserting");
                let n = db.import_items_json(&text)?;
                Ok(format!("Imported {n} records"))
            });
            self.stale = true;
        }
        if self.stale {
            self.load(cx);
        }

        widgets::page_header(
            ui,
            &pal,
            "Data",
            "Records from SQLite with server-side sorting, search, filtering and pagination.",
        );
        ui.horizontal_wrapped(|ui| {
            if widgets::search_field(ui, &pal, &mut self.search, "Search name or category", 240.0)
                .changed()
            {
                self.page = 0;
                self.stale = true;
            }
            let mut s = self.status;
            if widgets::segmented(ui, &pal, &mut s, &STATUSES) {
                self.status = s;
                self.page = 0;
                self.stale = true;
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Primary,
                Some(Icon::Plus),
                "Add record",
            )
            .clicked()
            {
                self.adding = !self.adding;
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Download),
                "Export CSV",
            )
            .clicked()
            {
                cx.open_dialog(
                    DialogPurpose::ExportCsv,
                    DialogKind::SaveFile,
                    "Export CSV",
                    "records.csv",
                );
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Download),
                "Export JSON",
            )
            .clicked()
            {
                cx.open_dialog(
                    DialogPurpose::ExportJson,
                    DialogKind::SaveFile,
                    "Export JSON",
                    "records.json",
                );
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Upload),
                "Import JSON",
            )
            .clicked()
            {
                cx.open_dialog(
                    DialogPurpose::ImportJson,
                    DialogKind::OpenFile,
                    "Import JSON",
                    "",
                );
            }
        });
        if self.adding {
            widgets::card(ui, &pal, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Name");
                    ui.add(egui::TextEdit::singleline(&mut self.form.name).desired_width(160.0));
                    ui.label("Category");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.category).desired_width(110.0),
                    );
                    ui.label("Amount");
                    ui.add(
                        egui::DragValue::new(&mut self.form.amount)
                            .speed(1.0)
                            .range(0.0..=1.0e9),
                    );
                    egui::ComboBox::from_id_salt("new-status")
                        .selected_text(self.form.status.clone())
                        .show_ui(ui, |ui| {
                            for s in &STATUSES[1..] {
                                ui.selectable_value(&mut self.form.status, (*s).to_string(), *s);
                            }
                        });
                    if widgets::button(
                        ui,
                        &pal,
                        ButtonKind::Primary,
                        Some(Icon::Check),
                        "Save record",
                    )
                    .clicked()
                    {
                        match cx.app.db.insert_item(&self.form) {
                            Ok(_) => {
                                cx.notify(Notification::success(
                                    "Record added",
                                    self.form.name.clone(),
                                ));
                                self.form.name.clear();
                                self.adding = false;
                                self.stale = true;
                            }
                            Err(e) => cx.fail("Could not add record", &e),
                        }
                    }
                });
            });
        }
        ui.add_space(space::SM);
        if self.rows.is_empty() {
            widgets::empty_state(
                ui,
                &pal,
                Icon::Search,
                "No records match",
                "Change the search or filter, or add a record.",
                |ui| {
                    if widgets::button(ui, &pal, ButtonKind::Secondary, None, "Clear filters")
                        .clicked()
                    {
                        self.search.clear();
                        self.status = 0;
                        self.stale = true;
                    }
                },
            );
        } else {
            let mut want_delete = None;
            widgets::card(ui, &pal, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::exact(56.0))
                    .column(Column::remainder().at_least(160.0))
                    .column(Column::exact(130.0))
                    .column(Column::exact(100.0))
                    .column(Column::exact(100.0))
                    .column(Column::exact(120.0))
                    .column(Column::exact(44.0))
                    .header(26.0, |mut h| {
                        h.col(|ui| self.header(ui, &pal, "ID", ItemSort::Id));
                        h.col(|ui| self.header(ui, &pal, "Name", ItemSort::Name));
                        h.col(|ui| self.header(ui, &pal, "Category", ItemSort::Category));
                        h.col(|ui| self.header(ui, &pal, "Status", ItemSort::Status));
                        h.col(|ui| self.header(ui, &pal, "Amount", ItemSort::Amount));
                        h.col(|ui| self.header(ui, &pal, "Updated", ItemSort::Updated));
                        h.col(|_| {});
                    })
                    .body(|mut body| {
                        for it in &self.rows {
                            body.row(28.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(it.id.to_string()).color(pal.text_muted),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(&it.name).color(pal.text));
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(&it.category).color(pal.text_secondary));
                                });
                                row.col(|ui| {
                                    let tone = match it.status.as_str() {
                                        "active" => Tone::Success,
                                        "paused" => Tone::Warning,
                                        _ => Tone::Neutral,
                                    };
                                    widgets::badge(ui, &pal, tone, &it.status);
                                });
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(format!("{:.2}", it.amount))
                                            .monospace()
                                            .color(pal.text),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(app_errors::time::format_date(
                                            it.updated_at.max(0) as u64,
                                        ))
                                        .color(pal.text_secondary),
                                    );
                                });
                                row.col(|ui| {
                                    if widgets::icon_button(ui, &pal, Icon::Trash, "Delete record")
                                        .clicked()
                                    {
                                        want_delete = Some((it.id, it.name.clone()));
                                    }
                                });
                            });
                        }
                    });
            });
            if want_delete.is_some() {
                self.delete = want_delete;
            }
            let pages = (self.total as usize).div_ceil(PAGE_SIZE).max(1);
            ui.add_space(space::SM);
            ui.horizontal(|ui| {
                ui.add_enabled_ui(self.page > 0, |ui| {
                    if widgets::icon_button(ui, &pal, Icon::ChevronLeft, "Previous page").clicked()
                    {
                        self.page -= 1;
                        self.stale = true;
                    }
                });
                ui.label(
                    RichText::new(format!(
                        "Page {} of {}  |  {} records",
                        self.page + 1,
                        pages,
                        self.total
                    ))
                    .color(pal.text_secondary),
                );
                ui.add_enabled_ui(self.page + 1 < pages, |ui| {
                    if widgets::icon_button(ui, &pal, Icon::ChevronRight, "Next page").clicked() {
                        self.page += 1;
                        self.stale = true;
                    }
                });
            });
        }
        if let Some((id, name)) = self.delete.clone() {
            match confirm_dialog(
                ui.ctx(),
                &pal,
                "delete-record",
                &format!("Delete \"{name}\"?"),
                "The record is removed permanently.",
                "Delete record",
                true,
            ) {
                ConfirmOutcome::Confirmed => {
                    self.delete = None;
                    match cx.app.db.delete_item(id) {
                        Ok(_) => {
                            cx.notify(Notification::info("Record deleted", name));
                            self.stale = true;
                        }
                        Err(e) => cx.fail("Could not delete record", &e),
                    }
                }
                ConfirmOutcome::Cancelled => self.delete = None,
                ConfirmOutcome::Pending => {}
            }
        }
        let _ = text::BODY;
    }
}
