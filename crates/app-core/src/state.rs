//! Navigation and shell state with explicit, testable transitions.
//!
//! All mutations go through [`AppState::reduce`]; UI code dispatches [`Action`]s and
//! never edits the fields directly. This keeps state changes predictable and unit-testable.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    Welcome,
    Dashboard,
    Workspace,
    Files,
    Data,
    Tasks,
    Logs,
    Settings,
    Appearance,
    About,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavGroup {
    Overview,
    Work,
    System,
}

impl Page {
    pub const ALL: [Page; 11] = [
        Page::Welcome,
        Page::Dashboard,
        Page::Workspace,
        Page::Files,
        Page::Data,
        Page::Tasks,
        Page::Logs,
        Page::Settings,
        Page::Appearance,
        Page::About,
        Page::Recovery,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Page::Welcome => "Welcome",
            Page::Dashboard => "Dashboard",
            Page::Workspace => "Workspace",
            Page::Files => "Files",
            Page::Data => "Data",
            Page::Tasks => "Tasks",
            Page::Logs => "Logs",
            Page::Settings => "Settings",
            Page::Appearance => "Appearance",
            Page::About => "About",
            Page::Recovery => "Recovery",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Page::Welcome => "welcome",
            Page::Dashboard => "dashboard",
            Page::Workspace => "workspace",
            Page::Files => "files",
            Page::Data => "data",
            Page::Tasks => "tasks",
            Page::Logs => "logs",
            Page::Settings => "settings",
            Page::Appearance => "appearance",
            Page::About => "about",
            Page::Recovery => "recovery",
        }
    }

    pub fn from_slug(s: &str) -> Option<Page> {
        Page::ALL.into_iter().find(|p| p.slug() == s)
    }

    pub fn group(self) -> NavGroup {
        match self {
            Page::Welcome | Page::Dashboard => NavGroup::Overview,
            Page::Workspace | Page::Files | Page::Data | Page::Tasks => NavGroup::Work,
            Page::Logs | Page::Settings | Page::Appearance | Page::About | Page::Recovery => {
                NavGroup::System
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Navigate(Page),
    Back,
    Forward,
    ToggleSidebar,
    OpenPalette,
    ClosePalette,
    ToggleNotifications,
    CloseNotifications,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub page: Page,
    back: Vec<Page>,
    forward: Vec<Page>,
    pub sidebar_collapsed: bool,
    pub palette_open: bool,
    pub notifications_open: bool,
}

impl AppState {
    pub fn new(start: Page) -> Self {
        Self {
            page: start,
            back: Vec::new(),
            forward: Vec::new(),
            sidebar_collapsed: false,
            palette_open: false,
            notifications_open: false,
        }
    }

    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    pub fn reduce(&mut self, action: Action) {
        match action {
            Action::Navigate(p) => {
                if p != self.page {
                    self.back.push(self.page);
                    if self.back.len() > 50 {
                        self.back.remove(0);
                    }
                    self.forward.clear();
                    self.page = p;
                }
                self.palette_open = false;
            }
            Action::Back => {
                if let Some(p) = self.back.pop() {
                    self.forward.push(self.page);
                    self.page = p;
                }
            }
            Action::Forward => {
                if let Some(p) = self.forward.pop() {
                    self.back.push(self.page);
                    self.page = p;
                }
            }
            Action::ToggleSidebar => self.sidebar_collapsed = !self.sidebar_collapsed,
            Action::OpenPalette => {
                self.palette_open = true;
                self.notifications_open = false;
            }
            Action::ClosePalette => self.palette_open = false,
            Action::ToggleNotifications => {
                self.notifications_open = !self.notifications_open;
                if self.notifications_open {
                    self.palette_open = false;
                }
            }
            Action::CloseNotifications => self.notifications_open = false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_history_works() {
        let mut s = AppState::new(Page::Welcome);
        s.reduce(Action::Navigate(Page::Dashboard));
        s.reduce(Action::Navigate(Page::Data));
        assert_eq!(s.page, Page::Data);
        s.reduce(Action::Back);
        assert_eq!(s.page, Page::Dashboard);
        s.reduce(Action::Forward);
        assert_eq!(s.page, Page::Data);
        s.reduce(Action::Back);
        s.reduce(Action::Navigate(Page::Logs));
        assert!(!s.can_go_forward(), "new navigation clears forward history");
    }

    #[test]
    fn navigating_to_current_page_does_not_grow_history() {
        let mut s = AppState::new(Page::Dashboard);
        s.reduce(Action::Navigate(Page::Dashboard));
        assert!(!s.can_go_back());
    }

    #[test]
    fn back_on_empty_history_is_a_noop() {
        let mut s = AppState::new(Page::Welcome);
        s.reduce(Action::Back);
        s.reduce(Action::Forward);
        assert_eq!(s.page, Page::Welcome);
    }

    #[test]
    fn overlays_are_mutually_exclusive() {
        let mut s = AppState::new(Page::Welcome);
        s.reduce(Action::ToggleNotifications);
        s.reduce(Action::OpenPalette);
        assert!(s.palette_open && !s.notifications_open);
        s.reduce(Action::ToggleNotifications);
        assert!(!s.palette_open && s.notifications_open);
    }

    #[test]
    fn navigating_closes_the_palette() {
        let mut s = AppState::new(Page::Welcome);
        s.reduce(Action::OpenPalette);
        s.reduce(Action::Navigate(Page::About));
        assert!(!s.palette_open);
    }

    #[test]
    fn history_is_bounded() {
        let mut s = AppState::new(Page::Welcome);
        for i in 0..200 {
            s.reduce(Action::Navigate(if i % 2 == 0 {
                Page::Data
            } else {
                Page::Logs
            }));
        }
        let mut n = 0;
        while s.can_go_back() {
            s.reduce(Action::Back);
            n += 1;
        }
        assert!(n <= 50);
    }

    #[test]
    fn slugs_round_trip() {
        for p in Page::ALL {
            assert_eq!(Page::from_slug(p.slug()), Some(p));
        }
        assert_eq!(Page::from_slug("nope"), None);
    }
}
