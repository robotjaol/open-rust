//! Open Rust desktop application entry point.

mod app;
mod cx;
mod icon;
mod module;
mod pages;
mod screenshot;

use app_config::ThemeMode;
use app_core::{AppContext, BootOptions, Page};
use std::path::PathBuf;

const HELP: &str = "\
Open Rust - a production-oriented Rust desktop application starterpack

USAGE:
    open-rust [OPTIONS]

OPTIONS:
    --home <DIR>             Keep all data below DIR (portable mode)
    --page <NAME>            Start on a page (welcome, dashboard, workspace, files, data,
                             tasks, logs, settings, appearance, about, recovery)
    --screenshot-dir <DIR>   Render every page in dark and light themes to PNG files, then exit
    --export-icon <FILE>     Write the 512px application icon as PNG, then exit
    -V, --version            Print the version
    -h, --help               Print this help

ENVIRONMENT:
    OPENRUST_HOME, OPENRUST_ENV (development|production), OPENRUST_LOG, OPENRUST_THEME";

#[derive(Debug, Default)]
struct Cli {
    home: Option<PathBuf>,
    page: Option<Page>,
    screenshot_dir: Option<PathBuf>,
    export_icon: Option<PathBuf>,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<Cli>, String> {
    let mut cli = Cli::default();
    let mut args = args.skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("open-rust {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--home" => cli.home = Some(value("--home")?.into()),
            "--screenshot-dir" => cli.screenshot_dir = Some(value("--screenshot-dir")?.into()),
            "--export-icon" => cli.export_icon = Some(value("--export-icon")?.into()),
            "--page" => {
                let v = value("--page")?;
                cli.page = Some(Page::from_slug(&v).ok_or_else(|| format!("unknown page `{v}`"))?);
            }
            other => return Err(format!("unknown option `{other}` (try --help)")),
        }
    }
    Ok(Some(cli))
}

fn main() -> eframe::Result {
    let cli = match parse_args(std::env::args()) {
        Ok(Some(c)) => c,
        Ok(None) => return Ok(()),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    if let Some(path) = &cli.export_icon {
        let rgba = icon::rgba(512);
        match image::RgbaImage::from_raw(512, 512, rgba).map(|i| i.save(path)) {
            Some(Ok(())) => println!("wrote {}", path.display()),
            _ => {
                eprintln!("error: could not write icon");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let screenshot_mode = cli.screenshot_dir.is_some();
    let (mut ctx, rx) = match AppContext::boot(&BootOptions {
        home: cli.home.clone(),
        ..BootOptions::default()
    }) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Open Rust could not start: {}", e.user_message());
            tracing::error!(error = %e, "boot failed");
            std::process::exit(1);
        }
    };
    if screenshot_mode {
        ctx.config.window.width = 1280.0;
        ctx.config.window.height = 800.0;
        ctx.config.window.x = None;
        ctx.config.window.maximized = false;
    }

    let w = &ctx.config.window;
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Open Rust")
        .with_inner_size([w.width, w.height])
        .with_min_inner_size([900.0, 600.0])
        .with_icon(egui::IconData {
            rgba: icon::rgba(128),
            width: 128,
            height: 128,
        });
    if ctx.config.general.restore_window {
        if let (Some(x), Some(y)) = (w.x, w.y) {
            viewport = viewport.with_position([x, y]);
        }
        if w.maximized && !screenshot_mode {
            viewport = viewport.with_maximized(true);
        }
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let plan = cli.screenshot_dir.map(|dir| {
        let _ = std::fs::create_dir_all(&dir);
        screenshot::Plan::new(
            dir,
            Page::ALL.to_vec(),
            vec![ThemeMode::Dark, ThemeMode::Light],
        )
    });
    let start_page = cli.page;
    eframe::run_native(
        "Open Rust",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::OpenRustApp::new(
                cc, ctx, rx, plan, start_page,
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> impl Iterator<Item = String> {
        std::iter::once("open-rust".to_string())
            .chain(v.iter().map(|s| (*s).to_string()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn parses_known_options() {
        let c = parse_args(args(&["--home", "/tmp/x", "--page", "data"]))
            .unwrap()
            .unwrap();
        assert_eq!(c.home, Some(PathBuf::from("/tmp/x")));
        assert_eq!(c.page, Some(Page::Data));
    }

    #[test]
    fn rejects_unknown_input() {
        assert!(parse_args(args(&["--nope"])).is_err());
        assert!(parse_args(args(&["--page", "nowhere"])).is_err());
        assert!(parse_args(args(&["--home"])).is_err());
    }

    #[test]
    fn help_and_version_short_circuit() {
        assert!(parse_args(args(&["--help"])).unwrap().is_none());
        assert!(parse_args(args(&["-V"])).unwrap().is_none());
    }
}
