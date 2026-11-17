# Developer pain points

Evidence is thin: one issue thread (S2) and the author's own build experience while making this
repository. The directive asks for issues supported by multiple sources; that bar was **not
met** by this session's research, so every item is marked accordingly.

| Pain point | Support | How the starterpack responds |
|---|---|---|
| Long first compile (434 crates) | Observed locally: first debug build took several minutes on 2 vCPUs | `Cargo.lock` committed; CI cache; thin LTO only in release |
| System libraries on Linux | Observed: `rfd` defaults pulled wayland-sys and failed to build | Dialogs use `xdg-portal` without default features |
| Accessibility cost | One report (S2) | Documented; measure before shipping |
| API churn between egui releases | Observed: 0.37 renamed `Panel::show_inside`, `WidgetType`, `Id::new` | Version pinned; migration notes in `docs/troubleshooting.md` |
| Cross-platform surprises | Not validated | CI matrix on three OS; compatibility table says "Not validated" |
| Packaging and signing | S11, S12 describe tools only | Scripted Linux packages; Windows/macOS documented, unsigned |

**Unresolved:** a proper survey (forums, issue trackers) would be needed to rank these.
