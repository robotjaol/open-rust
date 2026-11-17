use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::{DialogPurpose, Notification, Page};
use app_platform::FileService;
use app_platform::dialogs::DialogKind;
use app_platform::fs::{FileEntry, format_size};
use egui::RichText;
use std::path::PathBuf;
use ui_components::Icon;
use ui_components::tokens::{space, text};
use ui_components::widgets::{self, ButtonKind};

pub struct Files {
    dir: PathBuf,
    entries: Vec<FileEntry>,
    show_hidden: bool,
    selected: Option<PathBuf>,
    preview: Option<Result<String, String>>,
    error: Option<String>,
    loaded: bool,
}

impl Files {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            entries: Vec::new(),
            show_hidden: false,
            selected: None,
            preview: None,
            error: None,
            loaded: false,
        }
    }

    fn service(cx: &Cx<'_>) -> FileService {
        FileService::new(cx.app.config.files.max_file_size_mb)
    }

    fn refresh(&mut self, cx: &Cx<'_>) {
        self.loaded = true;
        match Self::service(cx).list_dir(&self.dir, self.show_hidden) {
            Ok(e) => {
                self.entries = e;
                self.error = None;
            }
            Err(e) => {
                self.entries.clear();
                self.error = Some(e.user_message());
            }
        }
    }

    fn open_dir(&mut self, dir: PathBuf, cx: &mut Cx<'_>) {
        self.dir = dir;
        self.selected = None;
        self.preview = None;
        self.refresh(cx);
    }
}

impl FeatureModule for Files {
    fn page(&self) -> Page {
        Page::Files
    }
    fn icon(&self) -> Icon {
        Icon::Folder
    }
    fn subtitle(&self) -> &'static str {
        "Browse and preview local files"
    }

    fn on_enter(&mut self, cx: &mut Cx<'_>) {
        self.refresh(cx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        if let Some(Some(p)) = cx.take_dialog(DialogPurpose::BrowseFolder) {
            self.open_dir(p, cx);
        }
        if !self.loaded {
            self.refresh(cx);
        }
        widgets::page_header(
            ui,
            &pal,
            "Files",
            "Read-only browser with size limits. Paths are checked before every read.",
        );
        ui.horizontal(|ui| {
            if widgets::icon_button(ui, &pal, Icon::ArrowUp, "Parent folder").clicked()
                && let Some(parent) = self.dir.parent().map(|p| p.to_path_buf())
            {
                self.open_dir(parent, cx);
            }
            if widgets::icon_button(ui, &pal, Icon::Refresh, "Refresh").clicked() {
                self.refresh(cx);
            }
            if widgets::button(
                ui,
                &pal,
                ButtonKind::Secondary,
                Some(Icon::Folder),
                "Choose folder",
            )
            .clicked()
            {
                cx.open_dialog(
                    DialogPurpose::BrowseFolder,
                    DialogKind::OpenFolder,
                    "Choose a folder",
                    "",
                );
            }
            let mut hidden = self.show_hidden;
            if widgets::toggle(ui, &pal, &mut hidden, "Show hidden").changed() {
                self.show_hidden = hidden;
                self.refresh(cx);
            }
        });
        ui.label(
            RichText::new(self.dir.display().to_string())
                .size(text::CAPTION)
                .color(pal.text_muted),
        );
        ui.add_space(space::SM);
        if let Some(err) = &self.error {
            widgets::card(ui, &pal, |ui| {
                widgets::badge(ui, &pal, ui_components::Tone::Danger, "Cannot open folder");
                ui.label(RichText::new(err).color(pal.text_secondary));
            });
            return;
        }
        let mut open: Option<PathBuf> = None;
        let mut pick: Option<PathBuf> = None;
        ui.columns(2, |c| {
            widgets::card(&mut c[0], &pal, |ui| {
                if self.entries.is_empty() {
                    widgets::empty_state(
                        ui,
                        &pal,
                        Icon::Folder,
                        "This folder is empty",
                        "Nothing to show here.",
                        |_| {},
                    );
                }
                egui::ScrollArea::vertical()
                    .max_height(430.0)
                    .show(ui, |ui| {
                        for e in &self.entries {
                            let sel = self.selected.as_ref() == Some(&e.path);
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(
                                    egui::vec2(18.0, 18.0),
                                    egui::Sense::hover(),
                                );
                                let icon = if e.is_dir { Icon::Folder } else { Icon::File };
                                ui_components::icons::paint_at(
                                    ui.painter(),
                                    icon,
                                    r.center(),
                                    15.0,
                                    if e.is_dir {
                                        pal.accent
                                    } else {
                                        pal.text_secondary
                                    },
                                );
                                let resp = ui
                                    .selectable_label(sel, RichText::new(&e.name).color(pal.text));
                                if resp.clicked() {
                                    if e.is_dir {
                                        open = Some(e.path.clone());
                                    } else {
                                        pick = Some(e.path.clone());
                                    }
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if !e.is_dir {
                                            ui.label(
                                                RichText::new(format_size(e.size))
                                                    .size(text::CAPTION)
                                                    .color(pal.text_muted),
                                            );
                                        }
                                    },
                                );
                            });
                        }
                    });
            });
            widgets::card(&mut c[1], &pal, |ui| {
                widgets::section_title(ui, &pal, "Preview");
                match &self.preview {
                    None => {
                        ui.label(
                            RichText::new("Select a file to preview its text.")
                                .color(pal.text_muted),
                        );
                    }
                    Some(Ok(t)) => {
                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(t.chars().take(4000).collect::<String>())
                                        .monospace()
                                        .size(text::CAPTION)
                                        .color(pal.text),
                                );
                            });
                    }
                    Some(Err(m)) => {
                        widgets::badge(ui, &pal, ui_components::Tone::Warning, "Cannot preview");
                        ui.label(RichText::new(m).color(pal.text_secondary));
                    }
                }
            });
        });
        if let Some(d) = open {
            self.open_dir(d, cx);
        }
        if let Some(p) = pick {
            self.preview = Some(
                Self::service(cx)
                    .read_text(&p)
                    .map_err(|e| e.user_message()),
            );
            if let Some(Err(m)) = &self.preview {
                cx.notify(Notification::warning("Preview unavailable", m.clone()));
            }
            self.selected = Some(p);
        }
    }
}
