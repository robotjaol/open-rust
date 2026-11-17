//! Automated screenshots for documentation and visual regression checks.
//!
//! `open-rust --screenshot-dir docs/screenshots` renders every page in dark and light
//! themes through the real application code path and writes PNG files.

use app_config::ThemeMode;
use app_core::Page;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Plan {
    pub dir: PathBuf,
    queue: VecDeque<(Page, ThemeMode)>,
    current: Option<(Page, ThemeMode)>,
    frames_waited: u32,
    requested: bool,
    saved: Arc<AtomicBool>,
    pub saved_files: Vec<PathBuf>,
}

pub enum Step {
    /// Show this page/theme and keep rendering.
    Show(Page, ThemeMode),
    Done,
}

impl Plan {
    pub fn new(dir: PathBuf, pages: Vec<Page>, themes: Vec<ThemeMode>) -> Self {
        let mut queue = VecDeque::new();
        for t in themes {
            for p in &pages {
                queue.push_back((*p, t));
            }
        }
        Self {
            dir,
            queue,
            current: None,
            frames_waited: 0,
            requested: false,
            saved: Arc::new(AtomicBool::new(false)),
            saved_files: Vec::new(),
        }
    }

    fn file_name(page: Page, theme: ThemeMode) -> String {
        let t = match theme {
            ThemeMode::Light => "light",
            _ => "dark",
        };
        format!("{t}-{}.png", page.slug())
    }

    /// Advance the state machine once per frame.
    pub fn tick(&mut self, ctx: &egui::Context) -> Step {
        if self.current.is_none() {
            match self.queue.pop_front() {
                Some(next) => {
                    self.current = Some(next);
                    self.frames_waited = 0;
                    self.requested = false;
                    self.saved.store(false, Ordering::SeqCst);
                }
                None => return Step::Done,
            }
        }
        let Some((page, theme)) = self.current else {
            return Step::Done;
        };
        self.frames_waited += 1;
        ctx.request_repaint();
        // Wait for layout, animations and fonts to settle before capturing.
        if self.frames_waited >= 14 && !self.requested {
            self.requested = true;
            let path = self.dir.join(Self::file_name(page, theme));
            let saved = self.saved.clone();
            ctx.request_screenshot(move |img| {
                let [w, h] = img.size;
                let raw: Vec<u8> = img.pixels.iter().flat_map(|p| p.to_array()).collect();
                let ok = image::RgbaImage::from_raw(w as u32, h as u32, raw)
                    .map(|i| i.save(&path).is_ok())
                    .unwrap_or(false);
                if !ok {
                    tracing::error!(?path, "could not save screenshot");
                }
                saved.store(true, Ordering::SeqCst);
            });
        }
        if self.requested && self.saved.load(Ordering::SeqCst) {
            self.saved_files
                .push(self.dir.join(Self::file_name(page, theme)));
            self.current = None;
        }
        match self.current.or(self.queue.front().copied()) {
            Some((p, t)) => Step::Show(p, t),
            None => Step::Done,
        }
    }
}
