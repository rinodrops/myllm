mod app;
mod args;
mod os;
mod settings;

use eframe::egui;

use app::MyApp;
use args::Args;

fn main() -> eframe::Result {
    let args = Args::from_env();
    let visible = args.is_single_shot();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 460.0])
            .with_min_inner_size([420.0, 320.0])
            .with_always_on_top()
            .with_visible(visible)
            .with_title("My LLM")
            .with_app_id("jp.emotiongraphics.myllm"),
        ..Default::default()
    };
    eframe::run_native(
        "My LLM",
        native_options,
        Box::new(move |cc| Ok(Box::new(MyApp::new(cc, args)))),
    )
}
