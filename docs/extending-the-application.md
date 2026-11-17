# Extending the application

A feature is one `FeatureModule` plus one `Page` variant. Example: a "Contacts" page.

1. Add `Contacts` to `Page` in `crates/app-core/src/state.rs` (update `ALL`, `title`, `slug`, `group`).
2. Create `apps/desktop/src/pages/contacts.rs`:

```rust
use crate::cx::Cx;
use crate::module::FeatureModule;
use app_core::Page;
use ui_components::{widgets, Icon};

#[derive(Default)]
pub struct Contacts { filter: String }

impl FeatureModule for Contacts {
    fn page(&self) -> Page { Page::Contacts }
    fn icon(&self) -> Icon { Icon::Notebook }
    fn subtitle(&self) -> &'static str { "People you work with" }

    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Cx<'_>) {
        let pal = cx.pal;
        widgets::page_header(ui, &pal, "Contacts", "A minimal example page.");
        widgets::search_field(ui, &pal, &mut self.filter, "Filter", 240.0);
    }
}
```

3. Declare it in `pages/mod.rs` (`pub mod contacts;`) and add `Box::new(pages::contacts::Contacts::default())` to `registry()` in `module.rs`.
4. Storage: add a migration in `crates/app-storage/src/migrations.rs` and repository methods.
5. Long work: `cx.app.tasks.spawn("Import contacts", |ctx| { ... })`.
6. Run `cargo test --workspace`; the test `every_page_has_exactly_one_module` fails if step 3 is missed.

Navigation, the sidebar, the command palette and the breadcrumb pick the page up automatically.
