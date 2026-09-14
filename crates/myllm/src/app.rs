use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Align2, Color32, Layout, RichText, ScrollArea, TextEdit, ViewportCommand,
};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use myllm_core::{stream_run, Appearance, Config, EmptyWindowTask, ResolvedRun};
use tray_icon::menu::accelerator::Accelerator;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::args::Args;
use crate::fonts;
use crate::i18n;
use crate::os;
use crate::settings;
use crate::window_state;

const OPEN_WINDOW: &str = "open_window";
const NOTICE_TTL: Duration = Duration::from_millis(2500);

enum StreamMsg {
    Token(String),
    Done,
    Error(String),
}

enum Job {
    Idle,
    PrepareInput { task_id: String },
    Running { rx: Receiver<StreamMsg> },
}

enum PendingShow {
    Empty,
    Task(String),
}

struct Notice {
    text: String,
    seen_at: Option<Instant>,
}

struct TrayBits {
    _tray: TrayIcon,
    _open_window: MenuItem,
    _reload: MenuItem,
    _open_config: MenuItem,
    _settings: MenuItem,
    _quit: MenuItem,
    tasks: Vec<(MenuItem, String)>,
}

struct AppMenuBits {
    _menu: Menu,
    _app: Submenu,
    _quit: PredefinedMenuItem,
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
    selected_task: String,
    last_task_id: Option<String>,
    last_submitted: Option<(String, String)>,
    auto_copy: bool,
    job: Job,
    awaiting_first_token: bool,
    prepare_after_paint: bool,
    follow_output: bool,
    source_pid: Option<u32>,
    visible: bool,
    single_shot: bool,
    quitting: bool,
    os_langs: Vec<String>,
    hotkeys: Option<GlobalHotKeyManager>,
    hotkey_map: HashMap<u32, String>,
    tray: Option<TrayBits>,
    app_menu: Option<AppMenuBits>,
    notice: Option<Notice>,
    pending_show: Option<(PendingShow, Instant)>,
    last_pos: Option<egui::Pos2>,
    pos_dirty_at: Option<Instant>,
    settings_child: Option<std::process::Child>,
    settings_mtime: Option<std::time::SystemTime>,
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
        let selected_task = config.default_task_id();
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
            selected_task,
            last_task_id: None,
            last_submitted: None,
            auto_copy: true,
            job: Job::Idle,
            awaiting_first_token: false,
            prepare_after_paint: false,
            follow_output: true,
            source_pid: None,
            visible: single_shot,
            single_shot,
            quitting: false,
            os_langs: os::preferred_ui_langs(),
            hotkeys: None,
            hotkey_map: HashMap::new(),
            tray: None,
            app_menu: None,
            notice: None,
            pending_show: None,
            last_pos: None,
            pos_dirty_at: None,
            settings_child: None,
            settings_mtime: None,
        };
        os::set_accessory(!single_shot);
        os::set_app_icon();
        os::install_quit_watch();
        app.install_hotkeys();
        app.install_tray();
        app.install_app_menu();
        if single_shot {
            let task = app
                .args
                .task
                .clone()
                .unwrap_or_else(|| app.config.default_task_id());
            app.launch_task(&task);
        }
        app
    }

    fn is_busy(&self) -> bool {
        !matches!(self.job, Job::Idle)
    }

    fn can_run(&self) -> bool {
        if self.is_busy() {
            return false;
        }
        self.last_submitted.as_ref() != Some(&(self.input.clone(), self.selected_task.clone()))
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
                self.push_notice(format!("hotkeys unavailable: {err}"));
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
        if let Some(hk) = self.config.open_hotkey() {
            specs.push((OPEN_WINDOW.into(), hk.to_string()));
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

    fn strings(&self) -> &'static i18n::Strings {
        i18n::t(self.config.ui_lang(), &self.os_langs)
    }

    fn push_notice(&mut self, text: impl Into<String>) {
        let body = text.into();
        let stamp = os::local_hm();
        let text = if stamp.is_empty() {
            body
        } else {
            format!("{stamp}  {body}")
        };
        self.notice = Some(Notice {
            text,
            seen_at: self.visible.then(Instant::now),
        });
    }

    fn expire_notice(&mut self, ctx: &egui::Context) {
        if self.visible {
            if let Some(notice) = &mut self.notice {
                if notice.seen_at.is_none() {
                    notice.seen_at = Some(Instant::now());
                }
            }
        }
        let Some(notice) = &self.notice else {
            return;
        };
        let Some(seen_at) = notice.seen_at else {
            return;
        };
        let elapsed = seen_at.elapsed();
        if elapsed >= NOTICE_TTL {
            self.notice = None;
        } else {
            ctx.request_repaint_after(NOTICE_TTL.saturating_sub(elapsed));
        }
    }

    fn paint_notice(&self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }
        let Some(notice) = &self.notice else {
            return;
        };
        let color = if ctx.style().visuals.dark_mode {
            Color32::from_rgb(0x8F, 0xCB, 0xB3)
        } else {
            Color32::from_rgb(0x3D, 0x8F, 0x78)
        };
        egui::Area::new(egui::Id::new("notice"))
            .anchor(Align2::RIGHT_TOP, egui::vec2(-12.0, 8.0))
            .interactable(false)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label(RichText::new(&notice.text).small().color(color));
            });
    }

    fn take_pending_show(&mut self, ctx: &egui::Context) {
        let Some((_, at)) = &self.pending_show else {
            return;
        };
        let wait = Duration::from_millis(50);
        if at.elapsed() < wait {
            ctx.request_repaint_after(wait.saturating_sub(at.elapsed()));
            return;
        }
        let Some((pending, _)) = self.pending_show.take() else {
            return;
        };
        match pending {
            PendingShow::Empty => self.open_empty_window(),
            PendingShow::Task(id) => self.launch_task(&id),
        }
    }

    fn install_tray(&mut self) {
        self.tray = None;
        let t = self.strings();
        let menu = Menu::new();
        let open_window = MenuItem::new(
            t.open_window,
            true,
            self.config.open_hotkey().and_then(parse_accelerator),
        );
        let _ = menu.append(&open_window);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let mut tasks = Vec::new();
        for (id, task) in &self.config.tasks {
            let name = task.name.clone().unwrap_or_else(|| id.clone());
            let item = MenuItem::new(
                &name,
                true,
                task.hotkey.as_deref().and_then(parse_accelerator),
            );
            let _ = menu.append(&item);
            tasks.push((item, id.clone()));
        }
        if self.config.translation().enabled {
            let item = MenuItem::new(
                t.translate,
                true,
                self.config
                    .translation()
                    .hotkey
                    .as_deref()
                    .and_then(parse_accelerator),
            );
            let _ = menu.append(&item);
            tasks.push((item, "translate".into()));
        }
        let _ = menu.append(&PredefinedMenuItem::separator());
        let reload = MenuItem::new(t.reload_config, true, None);
        let open_config = MenuItem::new(t.open_config, true, None);
        let settings_item =
            MenuItem::new(t.settings, settings::find_settings_binary().is_some(), None);
        let quit = MenuItem::new(t.quit, true, None);
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
                    _open_window: open_window,
                    _reload: reload,
                    _open_config: open_config,
                    _settings: settings_item,
                    _quit: quit,
                    tasks,
                });
            }
            Err(err) => {
                self.push_notice(format!("tray unavailable: {err}"));
            }
        }
    }

    fn install_app_menu(&mut self) {
        self.app_menu = None;
        #[cfg(target_os = "macos")]
        {
            let t = self.strings();
            let app = Submenu::new("My LLM", true);
            let quit = PredefinedMenuItem::quit(Some(t.quit));
            let _ = app.append(&quit);
            let menu = Menu::new();
            let _ = menu.append(&app);
            menu.init_for_nsapp();
            self.app_menu = Some(AppMenuBits {
                _menu: menu,
                _app: app,
                _quit: quit,
            });
        }
    }

    fn reload_config(&mut self, ctx: &egui::Context) {
        match Config::load_path(&self.config_path) {
            Ok(config) => {
                self.config = config;
                self.appearance = self.config.appearance();
                self.opacity = self.config.opacity();
                if !self.config.has_task(&self.selected_task) {
                    self.selected_task = self.config.default_task_id();
                }
                apply_appearance(ctx, self.appearance);
                apply_opacity(ctx, self.appearance, self.opacity);
                self.install_hotkeys();
                self.install_tray();
                self.install_app_menu();
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn settings_running(&mut self) -> bool {
        match self.settings_child.as_mut().map(|child| child.try_wait()) {
            None => false,
            Some(Ok(None)) => true,
            Some(_) => {
                self.settings_child = None;
                false
            }
        }
    }

    fn open_settings(&mut self, ctx: &egui::Context) {
        if self.settings_running() {
            return;
        }
        match settings::spawn_settings(&self.config_path) {
            Ok(child) => {
                self.settings_mtime = settings::config_mtime(&self.config_path);
                self.settings_child = Some(child);
                ctx.request_repaint_after(Duration::from_millis(400));
            }
            Err(err) => self.push_notice(err),
        }
    }

    fn poll_settings(&mut self, ctx: &egui::Context) {
        let done = match self.settings_child.as_mut().map(|child| child.try_wait()) {
            None => return,
            Some(Ok(None)) => {
                ctx.request_repaint_after(Duration::from_millis(400));
                return;
            }
            Some(Ok(Some(_))) => true,
            Some(Err(_)) => {
                self.settings_child = None;
                self.settings_mtime = None;
                return;
            }
        };
        if !done {
            return;
        }
        self.settings_child = None;
        let now = settings::config_mtime(&self.config_path);
        if now != self.settings_mtime {
            self.reload_config(ctx);
        }
        self.settings_mtime = None;
    }

    fn launch_task(&mut self, task_id: &str) {
        if self.is_busy() {
            return;
        }
        self.selected_task = task_id.to_string();
        self.last_task_id = Some(task_id.to_string());
        self.output.clear();
        self.error = None;
        self.awaiting_first_token = true;
        self.follow_output = true;
        self.show_window();
        self.job = Job::PrepareInput {
            task_id: task_id.to_string(),
        };
    }

    fn open_empty_window(&mut self) {
        if self.is_busy() {
            return;
        }
        let task = self.initial_empty_task();
        self.selected_task = task.clone();
        self.input.clear();
        self.output.clear();
        self.error = None;
        self.awaiting_first_token = false;
        self.last_submitted = Some((self.input.clone(), task));
        self.show_window();
    }

    fn initial_empty_task(&self) -> String {
        match self.config.empty_window_task() {
            EmptyWindowTask::Last => self
                .last_task_id
                .as_deref()
                .filter(|id| self.config.has_task(id))
                .map(ToString::to_string)
                .unwrap_or_else(|| self.config.default_task_id()),
            EmptyWindowTask::Default => self.config.default_task_id(),
        }
    }

    fn take_trigger_input(&self) -> String {
        if let Some(input) = self.args.input.clone().filter(|s| !s.is_empty()) {
            return input;
        }
        let text = if self.config.capture_selection() {
            os::capture_selection(self.source_pid)
        } else {
            os::read_clipboard()
        };
        if text.trim().is_empty() {
            self.strings().clipboard_empty.to_string()
        } else {
            text
        }
    }

    fn poll_prepare(&mut self) {
        let Job::PrepareInput { task_id } = &self.job else {
            return;
        };
        let task_id = task_id.clone();
        let input = self.take_trigger_input();
        self.start_run(&task_id, input);
    }

    fn run_selected(&mut self) {
        if !self.can_run() {
            return;
        }
        let task_id = self.selected_task.clone();
        let input = self.input.clone();
        self.last_task_id = Some(task_id.clone());
        self.start_run(&task_id, input);
    }

    fn copy_output(&mut self) {
        match os::write_clipboard(&self.output) {
            Ok(()) => self.push_notice(self.strings().copied),
            Err(err) => self.push_notice(err),
        }
    }

    fn start_run(&mut self, task_id: &str, input: String) {
        self.selected_task = task_id.to_string();
        self.last_task_id = Some(task_id.to_string());
        self.last_submitted = Some((input.clone(), task_id.to_string()));
        self.input = input.clone();
        self.output.clear();
        self.error = None;
        self.awaiting_first_token = true;
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
                self.show_window();
                self.job = Job::Running {
                    rx: spawn_stream(run),
                };
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.awaiting_first_token = false;
                self.show_window();
                self.job = Job::Idle;
            }
        }
    }

    fn poll_hotkeys(&mut self) {
        loop {
            let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() else {
                return;
            };
            if event.state != HotKeyState::Pressed {
                continue;
            }
            if let Some(pid) = os::frontmost_pid() {
                if pid != os::current_pid() {
                    self.source_pid = Some(pid);
                }
            }
            let Some(task) = self.hotkey_map.get(&event.id).cloned() else {
                continue;
            };
            if task == OPEN_WINDOW {
                self.open_empty_window();
            } else {
                self.launch_task(&task);
            }
        }
    }

    fn poll_tray(&mut self, ctx: &egui::Context) {
        loop {
            let Ok(event) = MenuEvent::receiver().try_recv() else {
                return;
            };
            if self.is_quit_menu(&event) {
                self.request_quit(ctx);
                return;
            }
            let Some(tray) = &self.tray else {
                continue;
            };
            let open_window = event.id == tray._open_window.id();
            let reload = event.id == tray._reload.id();
            let open_config = event.id == tray._open_config.id();
            let settings = event.id == tray._settings.id();
            let task_id = tray
                .tasks
                .iter()
                .find(|(item, _)| event.id == item.id())
                .map(|(_, id)| id.clone());
            if open_window {
                self.pending_show = Some((PendingShow::Empty, Instant::now()));
                ctx.request_repaint_after(Duration::from_millis(50));
                continue;
            }
            if reload {
                self.reload_config(ctx);
                continue;
            }
            if open_config {
                if let Err(err) = os::open_path(&self.config_path) {
                    self.push_notice(err);
                }
                continue;
            }
            if settings {
                self.open_settings(ctx);
                continue;
            }
            if let Some(id) = task_id {
                if let Some(pid) = os::frontmost_pid() {
                    if pid != os::current_pid() {
                        self.source_pid = Some(pid);
                    }
                }
                self.pending_show = Some((PendingShow::Task(id), Instant::now()));
                ctx.request_repaint_after(Duration::from_millis(50));
            }
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
            self.awaiting_first_token = false;
            self.output.push_str(&tokens.concat());
            ctx.request_repaint();
        }
        if let Some(err) = error {
            self.error = Some(err);
            self.awaiting_first_token = false;
        }
        if done {
            self.job = Job::Idle;
            self.awaiting_first_token = false;
            if self.auto_copy && self.error.is_none() && !self.output.is_empty() {
                if let Err(err) = os::write_clipboard(&self.output) {
                    self.push_notice(err);
                } else {
                    self.push_notice(self.strings().copied);
                }
            }
        } else {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn is_quit_menu(&self, event: &MenuEvent) -> bool {
        if let Some(tray) = &self.tray {
            if event.id == tray._quit.id() {
                return true;
            }
        }
        if let Some(menu) = &self.app_menu {
            if event.id == menu._quit.id() {
                return true;
            }
        }
        false
    }

    fn show_window(&mut self) {
        os::set_accessory(false);
        self.visible = true;
        if let Some(notice) = &mut self.notice {
            if notice.seen_at.is_none() {
                notice.seen_at = Some(Instant::now());
            }
        }
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        if self
            .notice
            .as_ref()
            .and_then(|notice| notice.seen_at)
            .is_some()
        {
            self.notice = None;
        }
        self.save_position(ctx);
        self.visible = false;
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        os::set_accessory(true);
    }

    fn request_quit(&mut self, ctx: &egui::Context) {
        self.save_position(ctx);
        self.quitting = true;
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    fn save_position(&mut self, ctx: &egui::Context) {
        if let Some(rect) = ctx.input(|i| i.viewport().outer_rect) {
            window_state::save(rect.min);
            self.last_pos = Some(rect.min);
            self.pos_dirty_at = None;
        } else if let Some(pos) = self.last_pos {
            window_state::save(pos);
            self.pos_dirty_at = None;
        }
    }

    fn track_position(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }
        let Some(rect) = ctx.input(|i| i.viewport().outer_rect) else {
            return;
        };
        let pos = rect.min;
        let moved = self
            .last_pos
            .map(|prev| (prev - pos).length() > 1.0)
            .unwrap_or(true);
        if moved {
            self.last_pos = Some(pos);
            self.pos_dirty_at = Some(Instant::now());
        }
        let Some(at) = self.pos_dirty_at else {
            return;
        };
        let wait = Duration::from_millis(400);
        if at.elapsed() >= wait {
            window_state::save(pos);
            self.pos_dirty_at = None;
        } else {
            ctx.request_repaint_after(wait.saturating_sub(at.elapsed()));
        }
    }

    fn handle_window_keys(&mut self, ctx: &egui::Context) {
        if !self.visible || self.quitting {
            return;
        }
        let cmd = ctx.input(|i| i.modifiers.mac_cmd || i.modifiers.command);
        if !cmd {
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.run_selected();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W)) {
            self.hide_or_quit(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::C)) {
            let already_copied = ctx.output(|o| {
                o.commands.iter().any(
                    |cmd| matches!(cmd, egui::OutputCommand::CopyText(text) if !text.is_empty()),
                )
            });
            if !already_copied {
                self.copy_output();
            }
        }
    }

    fn hide_or_quit(&mut self, ctx: &egui::Context) {
        if self.quitting {
            return;
        }
        if self.single_shot {
            self.request_quit(ctx);
            return;
        }
        self.hide_window(ctx);
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
    }

    fn task_choices(&self) -> Vec<(String, String)> {
        let t = self.strings();
        let mut choices: Vec<(String, String)> = self
            .config
            .tasks
            .iter()
            .map(|(id, task)| (id.clone(), task.name.clone().unwrap_or_else(|| id.clone())))
            .collect();
        if self.config.translation().enabled {
            choices.push(("translate".into(), t.translate.to_string()));
        }
        choices
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        os::apply_float_chrome(ctx, frame);
        apply_appearance(ctx, self.appearance);
        apply_opacity(ctx, self.appearance, self.opacity);

        self.take_pending_show(ctx);
        self.expire_notice(ctx);

        if !self.visible {
            if let Some(pid) = os::frontmost_pid() {
                if pid != os::current_pid() {
                    self.source_pid = Some(pid);
                }
            }
        }

        if self.prepare_after_paint {
            self.prepare_after_paint = false;
            self.poll_prepare();
        }

        self.poll_hotkeys();
        self.poll_tray(ctx);
        self.poll_settings(ctx);
        self.poll_stream(ctx);

        if os::take_app_menu_quit() {
            self.request_quit(ctx);
        }
        if ctx
            .input(|i| i.key_pressed(egui::Key::Q) && (i.modifiers.mac_cmd || i.modifiers.command))
        {
            self.request_quit(ctx);
        }

        if matches!(self.job, Job::PrepareInput { .. }) {
            self.prepare_after_paint = true;
            ctx.request_repaint();
        }

        if self.visible {
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        } else {
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            self.hide_or_quit(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !self.quitting {
            self.hide_or_quit(ctx);
        }

        let t = self.strings();

        if self.config_created && self.visible {
            egui::Window::new(t.config_created_title)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "{}\n{}",
                        t.config_created,
                        self.config_path.display()
                    ));
                    if ui.button(t.ok).clicked() {
                        self.config_created = false;
                    }
                });
        }

        let busy = self.is_busy();
        let can_run = self.can_run();
        let choices = self.task_choices();
        let selected_label = if self.selected_task == "translate" {
            t.translate.to_string()
        } else {
            self.config.task_label(&self.selected_task)
        };
        let backend = self.config.backend_label(&self.selected_task);
        let run_tip = shortcut_tip(t.run, "cmd+r");
        let copy_tip = shortcut_tip(t.copy, "cmd+c");

        egui::TopBottomPanel::bottom("actions")
            .show_separator_line(false)
            .frame(actions_frame(ctx))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if pill_button(ui, t.copy).on_hover_text(&copy_tip).clicked() {
                            self.copy_output();
                        }
                        ui.add_enabled_ui(can_run, |ui| {
                            if pill_button(ui, t.run).on_hover_text(&run_tip).clicked() {
                                self.run_selected();
                            }
                        });
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add_enabled_ui(!busy, |ui| {
                                egui::ComboBox::from_id_salt("task_picker")
                                    .selected_text(selected_label)
                                    .show_ui(ui, |ui| {
                                        for (id, name) in &choices {
                                            ui.selectable_value(
                                                &mut self.selected_task,
                                                id.clone(),
                                                name,
                                            );
                                        }
                                    });
                            });
                            if let Some((provider, model)) = &backend {
                                ui.weak(format!("{provider} ({model})"));
                            }
                        });
                    });
                });
                if !os::supports_in_process_hotkeys() && !self.single_shot {
                    ui.weak(t.wayland_hint);
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
                        ui.label(RichText::new(t.input).small().color(Color32::GRAY));
                        ui.add_enabled_ui(!busy, |ui| {
                            ScrollArea::vertical()
                                .id_salt("input")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let mut edit = TextEdit::multiline(&mut self.input)
                                        .desired_width(f32::INFINITY);
                                    if busy {
                                        edit = edit.text_color(Color32::GRAY);
                                    }
                                    ui.add_sized(ui.available_size(), edit);
                                });
                        });
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(width, half),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.label(RichText::new(t.output).small().color(Color32::GRAY));
                        if let Some(err) = &self.error {
                            ui.colored_label(Color32::from_rgb(220, 80, 80), err);
                        }
                        if self.awaiting_first_token && self.error.is_none() {
                            ui.centered_and_justified(|ui| {
                                ui.add(egui::Spinner::new().size(24.0));
                            });
                        } else {
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
                        }
                    },
                );
            });
        self.handle_window_keys(ctx);
        self.track_position(ctx);
        self.paint_notice(ctx);
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

fn parse_accelerator(spec: &str) -> Option<Accelerator> {
    let mut normalized = String::new();
    for (i, part) in spec
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        if i > 0 {
            normalized.push('+');
        }
        match part.to_ascii_lowercase().as_str() {
            "win" | "windows" | "super" | "meta" => normalized.push_str("cmd"),
            other => normalized.push_str(other),
        }
    }
    normalized.parse().ok()
}

#[cfg(test)]
mod accel_tests {
    use super::parse_accelerator;

    #[test]
    fn parses_cmd_shift_p() {
        assert!(parse_accelerator("cmd+shift+p").is_some());
        assert!(parse_accelerator("ctrl+cmd+b").is_some());
        assert!(parse_accelerator("not-a-key").is_none());
    }
}

fn shortcut_tip(action: &str, spec: &str) -> String {
    match i18n::format_hotkey(spec) {
        Some(keys) => format!("{action}  {keys}"),
        None => action.to_string(),
    }
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
