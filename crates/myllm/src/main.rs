mod app;
mod args;
mod assets;
mod fonts;
mod i18n;
mod os;
mod settings;

use eframe::egui;

use app::MyApp;
use args::Args;

fn main() -> eframe::Result {
    let args = Args::from_env();
    let visible = args.is_single_shot();
    let icon = eframe::icon_data::from_png_bytes(assets::APP_ICON_PNG).expect("app icon");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 460.0])
            .with_min_inner_size([420.0, 320.0])
            .with_always_on_top()
            .with_visible(visible)
            .with_title("")
            .with_app_id("jp.emotiongraphics.myllm")
            .with_icon(icon)
            .with_transparent(true)
            .with_fullsize_content_view(true)
            .with_title_shown(false)
            .with_titlebar_shown(false),
        ..Default::default()
    };
    eframe::run_native(
        "My LLM",
        native_options,
        Box::new(move |cc| Ok(Box::new(MyApp::new(cc, args)))),
    )
}
