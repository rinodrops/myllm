#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod args;
mod assets;
mod fonts;
mod i18n;
mod os;
mod settings;
mod window_state;

use eframe::egui;

use app::MyApp;
use args::Args;

fn main() -> eframe::Result {
    if !os::acquire_instance() {
        return Ok(());
    }
    let result = run();
    if let Err(err) = &result {
        os::show_startup_error(&err.to_string());
    }
    result
}

fn run() -> eframe::Result {
    let args = Args::from_env();
    let visible = args.is_single_shot();
    let icon = eframe::icon_data::from_png_bytes(assets::APP_ICON_PNG).expect("app icon");
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([600.0, 460.0])
        .with_min_inner_size([420.0, 320.0])
        .with_always_on_top()
        .with_visible(visible)
        .with_title("")
        .with_app_id("jp.emotiongraphics.myllm")
        .with_icon(icon)
        .with_transparent(cfg!(not(target_os = "windows")))
        .with_fullsize_content_view(true)
        .with_title_shown(false)
        .with_titlebar_shown(false);
    if let Some(pos) = window_state::load() {
        viewport = viewport.with_position(pos);
    }
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "My LLM",
        native_options,
        Box::new(move |cc| Ok(Box::new(MyApp::new(cc, args)))),
    )
}
