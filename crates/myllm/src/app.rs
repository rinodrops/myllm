use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use eframe::egui::{self, Align, Color32, Layout, RichText, ScrollArea, TextEdit, ViewportCommand};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use myllm_core::{stream_run, Appearance, Config, ResolvedRun};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::args::Args;
use crate::fonts;
use crate::os;
use crate::settings;

enum StreamMsg {
    Token(String),
    Done,
    Error(String),
}

enum Job {
    Idle,
    Running { rx: Receiver<StreamMsg> },
}

struct TrayBits {
    _tray: TrayIcon,
    _reload: MenuItem,
    _open_config: MenuItem,
    _settings: MenuItem,
    _quit: MenuItem,
    tasks: Vec<(MenuItem, String)>,
}

pub struct MyApp {
    args: Args,
    config: Config,
    config_path: PathBuf,
    config_created: bool,
    appearance: Appearance,
    opacity: f32,
    input: String,
    output: String,
    error: Option<String>,
    task_id: Option<String>,
    auto_copy: bool,
    job: Job,
    follow_output: bool,
    source_pid: Option<u32>,
    visible: bool,
    single_shot: bool,
    hotkeys: Option<GlobalHotKeyManager>,
    hotkey_map: HashMap<u32, String>,
    tray: Option<TrayBits>,
    status: Option<String>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        let single_shot = args.is_single_shot();
        let (config, config_path, config_created) =
            Config::load_or_bootstrap().unwrap_or_else(|err| {
                eprintln!("{err}");
                (Config::default(), myllm_core::config_file_path(), false)
            });
        let appearance = config.appearance();
        let opacity = config.opacity();
        fonts::setup_fonts(&cc.egui_ctx);
        apply_appearance(&cc.egui_ctx, appearance);
        apply_opacity(&cc.egui_ctx, appearance, opacity);

        let mut app = Self {
            args,
            config,
            config_path,
            config_created,
            appearance,
            opacity,
            input: String::new(),
            output: String::new(),
            error: None,
            task_id: None,
            auto_copy: true,
            job: Job::Idle,
            follow_output: true,
            source_pid: None,
            visible: single_shot,
            single_shot,
            hotkeys: None,
            hotkey_map: HashMap::new(),
            tray: None,
            status: None,
        };
        os::set_accessory(!single_shot);
        os::set_app_icon();
        app.install_hotkeys();
        app.install_tray();
        if single_shot {
            let task = app.args.task.clone().unwrap_or_else(|| "polish".into());
            app.launch_task(&task);
        }
        app
    }

    fn install_hotkeys(&mut self) {
        self.hotkey_map.clear();
        self.hotkeys = None;
        if !os::supports_in_process_hotkeys() {
            return;
        }
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(err) => {
                self.status = Some(format!("hotkeys unavailable: {err}"));
                return;
            }
        };
        let mut specs: Vec<(String, String)> = self
            .config
            .tasks
            .iter()
            .filter_map(|(id, task)| task.hotkey.clone().map(|hk| (id.clone(), hk)))
            .collect();
        if let Some(hk) = self.config.translation().hotkey {
            if self.config.translation().enabled {
                specs.push(("translate".into(), hk));
            }
        }
        for (id, spec) in specs {
            if let Some(hotkey) = parse_hotkey(&spec) {
                if manager.register(hotkey).is_ok() {
                    self.hotkey_map.insert(hotkey.id(), id);
                }
            }
        }
        self.hotkeys = Some(manager);
    }

    fn install_tray(&mut self) {
        self.tray = None;
        let menu = Menu::new();
        let mut tasks = Vec::new();
        for (id, task) in &self.config.tasks {
            let label = task.name.clone().unwrap_or_else(|| id.clone());
            let item = MenuItem::new(&label, true, None);
            let _ = menu.append(&item);
            tasks.push((item, id.clone()));
        }
        if self.config.translation().enabled {
            let item = MenuItem::new("Translate", true, None);
            let _ = menu.append(&item);
            tasks.push((item, "translate".into()));
        }
        let _ = menu.append(&PredefinedMenuItem::separator());
        let reload = MenuItem::new("Reload Config", true, None);
        let open_config = MenuItem::new("Open Config Folder", true, None);
        let settings_item = MenuItem::new(
            "Settings…",
            settings::find_settings_binary().is_some(),
            None,
        );
        let quit = MenuItem::new("Quit My LLM", true, None);
        let _ = menu.append(&reload);
        let _ = menu.append(&open_config);
        let _ = menu.append(&settings_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit);
        let mut tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("My LLM")
            .with_icon(tray_icon_image());
        #[cfg(target_os = "macos")]
        {
            tray = tray.with_icon_as_template(true);
        }
        match tray.build() {
            Ok(tray) => {
                self.tray = Some(TrayBits {
                    _tray: tray,
                    _reload: reload,
                    _open_config: open_config,
                    _settings: settings_item,
                    _quit: quit,
                    tasks,
                });
            }
            Err(err) => {
                self.status = Some(format!("tray unavailable: {err}"));
            }
        }
    }

    fn reload_config(&mut self, ctx: &egui::Context) {
        match Config::load_path(&self.config_path) {
            Ok(config) => {
                self.config = config;
                self.appearance = self.config.appearance();
                self.opacity = self.config.opacity();
                apply_appearance(ctx, self.appearance);
                apply_opacity(ctx, self.appearance, self.opacity);
                self.install_hotkeys();
                self.install_tray();
                self.status = Some("Config reloaded".into());
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn launch_task(&mut self, task_id: &str) {
        let input = self
            .args
            .input
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| os::capture_selection(self.source_pid));
        let input = if input.trim().is_empty() {
            os::read_clipboard()
        } else {
            input
        };
        let input = if input.trim().is_empty() {
            "No text selected and clipboard is empty.".to_string()
        } else {
            input
        };
        self.start_run(task_id, input);
    }

    fn process_input(&mut self) {
        let Some(task_id) = self.task_id.clone() else {
            return;
        };
        let input = self.input.clone();
        self.start_run(&task_id, input);
    }

    fn copy_output(&mut self) {
        if let Err(err) = os::write_clipboard(&self.output) {
            self.status = Some(err);
        }
    }

    fn start_run(&mut self, task_id: &str, input: String) {
        self.task_id = Some(task_id.to_string());
        self.input = input.clone();
        self.output.clear();
        self.error = None;
        self.follow_output = true;
        match self.config.resolve_run(
            task_id,
            &input,
            self.args.from.as_deref(),
            self.args.to.as_deref(),
        ) {
            Ok(run) => {
                self.auto_copy = if self.args.no_copy {
                    false
                } else {
                    run.auto_copy
                };
                self.visible = true;
                os::set_accessory(false);
                self.job = Job::Running {
                    rx: spawn_stream(run),
                };
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.visible = true;
                os::set_accessory(false);
                self.job = Job::Idle;
            }
        }
    }

    fn poll_hotkeys(&mut self) {
        let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() else {
            return;
        };
        if event.state != HotKeyState::Pressed {
            return;
        }
        if let Some(pid) = os::frontmost_pid() {
            if pid != os::current_pid() {
                self.source_pid = Some(pid);
            }
        }
        if let Some(task) = self.hotkey_map.get(&event.id).cloned() {
            self.launch_task(&task);
        }
    }

    fn poll_tray(&mut self, ctx: &egui::Context) {
        let Some(tray) = &self.tray else {
            return;
        };
        let Ok(event) = MenuEvent::receiver().try_recv() else {
            return;
        };
        if event.id == tray._quit.id() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        if event.id == tray._reload.id() {
            self.reload_config(ctx);
            return;
        }
        if event.id == tray._open_config.id() {
            if let Err(err) = os::open_path(&self.config_path) {
                self.status = Some(err);
            }
            return;
        }
        if event.id == tray._settings.id() {
            match settings::spawn_settings(&self.config_path) {
                Ok(()) => self.status = Some("Opened Settings".into()),
                Err(err) => self.status = Some(err),
            }
            return;
        }
        if let Some((_, id)) = tray.tasks.iter().find(|(item, _)| event.id == item.id()) {
            if let Some(pid) = os::frontmost_pid() {
                if pid != os::current_pid() {
                    self.source_pid = Some(pid);
                }
            }
            let id = id.clone();
            self.launch_task(&id);
        }
    }

    fn poll_stream(&mut self, ctx: &egui::Context) {
        let Job::Running { rx } = &self.job else {
            return;
        };
        let mut tokens = Vec::new();
        let mut done = false;
        let mut error = None;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                StreamMsg::Token(t) => tokens.push(t),
                StreamMsg::Done => done = true,
                StreamMsg::Error(e) => {
                    error = Some(e);
                    done = true;
                }
            }
        }
        if !tokens.is_empty() {
            self.output.push_str(&tokens.concat());
            ctx.request_repaint();
        }
        if let Some(err) = error {
            self.error = Some(err);
        }
        if done {
            self.job = Job::Idle;
            if self.auto_copy && self.error.is_none() && !self.output.is_empty() {
                if let Err(err) = os::write_clipboard(&self.output) {
                    self.status = Some(err);
                }
            }
        } else {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn hide_or_quit(&mut self, ctx: &egui::Context) {
        if self.single_shot {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        self.visible = false;
        os::set_accessory(true);
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        os::apply_float_chrome(ctx);
        apply_appearance(ctx, self.appearance);
        apply_opacity(ctx, self.appearance, self.opacity);

        if !self.visible {
            if let Some(pid) = os::frontmost_pid() {
                if pid != os::current_pid() {
                    self.source_pid = Some(pid);
                }
            }
        }

        self.poll_hotkeys();
        self.poll_tray(ctx);
        self.poll_stream(ctx);

        if self.visible {
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            self.hide_or_quit(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide_or_quit(ctx);
        }

        if self.config_created {
            egui::Window::new("Configuration created")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "A starter config was created at:\n{}",
                        self.config_path.display()
                    ));
                    if ui.button("OK").clicked() {
                        self.config_created = false;
                    }
                });
        }

        egui::TopBottomPanel::bottom("actions")
            .show_separator_line(false)
            .frame(actions_frame(ctx))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let can_process =
                        self.task_id.is_some() && !matches!(self.job, Job::Running { .. });
                    ui.add_enabled_ui(can_process, |ui| {
                        if pill_button(ui, "Process").clicked() {
                            self.process_input();
                        }
                    });
                    if pill_button(ui, "Copy").clicked() {
                        self.copy_output();
                    }
                });
                if let Some(status) = &self.status {
                    ui.weak(status);
                } else if !os::supports_in_process_hotkeys() && !self.single_shot {
                    ui.weak("On Wayland, assign a compositor shortcut to `myllm --task <id>`.");
                }
            });

        egui::CentralPanel::default()
            .frame(content_frame(ctx))
            .show(ctx, |ui| {
                let width = ui.available_width();
                ui.spacing_mut().item_spacing.y = 8.0;
                let half = (ui.available_height() - ui.spacing().item_spacing.y) / 2.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(width, half),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.label(RichText::new("Input").small().color(Color32::GRAY));
                        ScrollArea::vertical()
                            .id_salt("input")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add_sized(
                                    ui.available_size(),
                                    TextEdit::multiline(&mut self.input)
                                        .desired_width(f32::INFINITY),
                                );
                            });
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(width, half),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.label(RichText::new("Output").small().color(Color32::GRAY));
                        if let Some(err) = &self.error {
                            ui.colored_label(Color32::from_rgb(220, 80, 80), err);
                        }
                        ScrollArea::vertical()
                            .id_salt("output")
                            .stick_to_bottom(self.follow_output)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let mut output = self.output.clone();
                                ui.add_sized(
                                    ui.available_size(),
                                    TextEdit::multiline(&mut output)
                                        .desired_width(f32::INFINITY)
                                        .interactive(true),
                                );
                                let _ = output;
                            });
                    },
                );
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

fn spawn_stream(run: ResolvedRun) -> Receiver<StreamMsg> {
    let (tx, rx): (Sender<StreamMsg>, Receiver<StreamMsg>) = mpsc::channel();
    thread::spawn(move || {
        let result = stream_run(&run, |token| {
            let _ = tx.send(StreamMsg::Token(token.to_string()));
        });
        match result {
            Ok(()) => {
                let _ = tx.send(StreamMsg::Done);
            }
            Err(err) => {
                let _ = tx.send(StreamMsg::Error(err.to_string()));
            }
        }
    });
    rx
}

fn apply_appearance(ctx: &egui::Context, appearance: Appearance) {
    ctx.set_theme(match appearance {
        Appearance::Dark => egui::ThemePreference::Dark,
        Appearance::Light => egui::ThemePreference::Light,
        Appearance::System => egui::ThemePreference::System,
    });
}

fn apply_opacity(ctx: &egui::Context, appearance: Appearance, opacity: f32) {
    let dark = match appearance {
        Appearance::Dark => true,
        Appearance::Light => false,
        Appearance::System => ctx.style().visuals.dark_mode,
    };
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let alpha = (opacity.clamp(0.5, 1.0) * 255.0).round() as u8;
    visuals.panel_fill = with_alpha(visuals.panel_fill, alpha);
    visuals.window_fill = with_alpha(visuals.window_fill, alpha);
    visuals.extreme_bg_color = with_alpha(visuals.extreme_bg_color, alpha);
    visuals.faint_bg_color = with_alpha(visuals.faint_bg_color, alpha);
    ctx.set_visuals(visuals);
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn content_frame(ctx: &egui::Context) -> egui::Frame {
    let top = if cfg!(target_os = "macos") { 36 } else { 12 };
    egui::Frame::new()
        .fill(ctx.style().visuals.panel_fill)
        .inner_margin(egui::Margin {
            left: 12,
            right: 12,
            top,
            bottom: 4,
        })
}

fn actions_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::new()
        .fill(ctx.style().visuals.panel_fill)
        .inner_margin(egui::Margin {
            left: 12,
            right: 12,
            top: 4,
            bottom: 12,
        })
}

fn pill_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let measure = ui.fonts(|fonts| {
        fonts.layout_no_wrap(
            label.to_owned(),
            egui::FontId::proportional(13.0),
            Color32::WHITE,
        )
    });
    let height = 28.0;
    let width = (measure.size().x + 32.0).max(68.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let (fill, text) = pill_colors(dark, ui.is_enabled(), &response);
    let galley = ui.fonts(|fonts| {
        fonts.layout_no_wrap(label.to_owned(), egui::FontId::proportional(13.0), text)
    });
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same((height / 2.0) as u8), fill);
    let text_pos = egui::pos2(
        rect.center().x - galley.size().x * 0.5,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, text);
    response
}

fn pill_colors(dark: bool, enabled: bool, response: &egui::Response) -> (Color32, Color32) {
    if !enabled {
        return if dark {
            (
                Color32::from_rgb(50, 50, 52),
                Color32::from_rgb(120, 120, 124),
            )
        } else {
            (
                Color32::from_rgb(236, 236, 238),
                Color32::from_rgb(170, 170, 174),
            )
        };
    }
    if response.is_pointer_button_down_on() {
        return if dark {
            (
                Color32::from_rgb(88, 88, 92),
                Color32::from_rgb(250, 250, 252),
            )
        } else {
            (
                Color32::from_rgb(200, 200, 204),
                Color32::from_rgb(29, 29, 31),
            )
        };
    }
    if response.hovered() {
        return if dark {
            (
                Color32::from_rgb(72, 72, 76),
                Color32::from_rgb(245, 245, 247),
            )
        } else {
            (
                Color32::from_rgb(214, 214, 218),
                Color32::from_rgb(50, 50, 52),
            )
        };
    }
    if dark {
        (
            Color32::from_rgb(58, 58, 62),
            Color32::from_rgb(235, 235, 240),
        )
    } else {
        (
            Color32::from_rgb(228, 228, 230),
            Color32::from_rgb(110, 110, 115),
        )
    }
}

fn parse_hotkey(spec: &str) -> Option<HotKey> {
    let mut mods = Modifiers::empty();
    let parts: Vec<&str> = spec
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let (key, prefix) = parts.split_last()?;
    for part in prefix {
        mods |= match part.to_ascii_lowercase().as_str() {
            "cmd" | "command" | "super" | "meta" | "win" | "windows" => Modifiers::META,
            "ctrl" | "control" => Modifiers::CONTROL,
            "opt" | "option" | "alt" => Modifiers::ALT,
            "shift" => Modifiers::SHIFT,
            _ => return None,
        };
    }
    Some(HotKey::new(Some(mods), parse_code(key)?))
}

fn parse_code(key: &str) -> Option<Code> {
    if key.len() == 1 {
        let c = key.chars().next()?.to_ascii_uppercase();
        return match c {
            'A' => Some(Code::KeyA),
            'B' => Some(Code::KeyB),
            'C' => Some(Code::KeyC),
            'D' => Some(Code::KeyD),
            'E' => Some(Code::KeyE),
            'F' => Some(Code::KeyF),
            'G' => Some(Code::KeyG),
            'H' => Some(Code::KeyH),
            'I' => Some(Code::KeyI),
            'J' => Some(Code::KeyJ),
            'K' => Some(Code::KeyK),
            'L' => Some(Code::KeyL),
            'M' => Some(Code::KeyM),
            'N' => Some(Code::KeyN),
            'O' => Some(Code::KeyO),
            'P' => Some(Code::KeyP),
            'Q' => Some(Code::KeyQ),
            'R' => Some(Code::KeyR),
            'S' => Some(Code::KeyS),
            'T' => Some(Code::KeyT),
            'U' => Some(Code::KeyU),
            'V' => Some(Code::KeyV),
            'W' => Some(Code::KeyW),
            'X' => Some(Code::KeyX),
            'Y' => Some(Code::KeyY),
            'Z' => Some(Code::KeyZ),
            '0' => Some(Code::Digit0),
            '1' => Some(Code::Digit1),
            '2' => Some(Code::Digit2),
            '3' => Some(Code::Digit3),
            '4' => Some(Code::Digit4),
            '5' => Some(Code::Digit5),
            '6' => Some(Code::Digit6),
            '7' => Some(Code::Digit7),
            '8' => Some(Code::Digit8),
            '9' => Some(Code::Digit9),
            _ => None,
        };
    }
    None
}

fn tray_icon_image() -> Icon {
    let icon =
        eframe::icon_data::from_png_bytes(crate::assets::TRAY_ICON_PNG).expect("tray icon png");
    Icon::from_rgba(icon.rgba, icon.width, icon.height).expect("tray icon")
}
