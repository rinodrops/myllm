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
    input: String,
    output: String,
    error: Option<String>,
    title: String,
    auto_copy: bool,
    job: Job,
    output_done: bool,
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
        let (config, config_path, config_created) = Config::load_or_bootstrap().unwrap_or_else(|err| {
            eprintln!("{err}");
            (Config::default(), myllm_core::config_file_path(), false)
        });
        let appearance = config.appearance();
        apply_appearance(&cc.egui_ctx, appearance);

        let mut app = Self {
            args,
            config,
            config_path,
            config_created,
            appearance,
            input: String::new(),
            output: String::new(),
            error: None,
            title: "My LLM".into(),
            auto_copy: true,
            job: Job::Idle,
            output_done: false,
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
        app.install_hotkeys();
        app.install_tray();
        if single_shot {
            let task = app
                .args
                .task
                .clone()
                .unwrap_or_else(|| "polish".into());
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
        match TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("My LLM")
            .with_icon(tray_icon_image())
            .build()
        {
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
                apply_appearance(ctx, self.appearance);
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

    fn start_run(&mut self, task_id: &str, input: String) {
        self.input = input.clone();
        self.output.clear();
        self.error = None;
        self.output_done = false;
        self.follow_output = true;
        match self.config.resolve_run(
            task_id,
            &input,
            self.args.from.as_deref(),
            self.args.to.as_deref(),
        ) {
            Ok(run) => {
                if let Some(name) = &self.args.task_name {
                    self.title = name.clone();
                } else {
                    self.title = run.display_name.clone();
                }
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
            self.output_done = true;
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
            ctx.send_viewport_cmd(ViewportCommand::Title(self.title.clone()));
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

        egui::TopBottomPanel::top("input_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(&self.title);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.output_done && ui.button("Copy").clicked() {
                        if let Err(err) = os::write_clipboard(&self.output) {
                            self.status = Some(err);
                        }
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            if let Some(status) = &self.status {
                ui.weak(status);
            } else if !os::supports_in_process_hotkeys() && !self.single_shot {
                ui.weak("On Wayland, assign a compositor shortcut to `myllm --task <id>`.");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_height();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), avail * 0.35),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.label(RichText::new("Input").small().color(Color32::GRAY));
                    ScrollArea::vertical()
                        .id_salt("input")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.input)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(6),
                            );
                        });
                },
            );
            ui.separator();
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
                    ui.add(
                        TextEdit::multiline(&mut output)
                            .desired_width(f32::INFINITY)
                            .desired_rows(12)
                            .interactive(true),
                    );
                    let _ = output;
                });
        });
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
    match appearance {
        Appearance::Dark => ctx.set_visuals(egui::Visuals::dark()),
        Appearance::Light => ctx.set_visuals(egui::Visuals::light()),
        Appearance::System => {}
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
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            rgba[i] = 52;
            rgba[i + 1] = 120;
            rgba[i + 2] = 247;
            rgba[i + 3] = 255;
        }
    }
    Icon::from_rgba(rgba, SIZE, SIZE).expect("tray icon")
}
