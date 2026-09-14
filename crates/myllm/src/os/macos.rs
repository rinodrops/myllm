use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::{NSObject, Sel};
use objc2::{define_class, msg_send, sel, AnyThread};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSColor, NSImage, NSRunningApplication, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask, NSWindowTitleVisibility, NSWorkspace,
};
use objc2_foundation::{
    MainThreadMarker, NSData, NSLocale, NSNotification, NSNotificationCenter, NSString,
};

use crate::assets;

use super::read_clipboard;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceCreate(state_id: u32) -> *mut c_void;
    fn CGEventCreateKeyboardEvent(
        source: *mut c_void,
        virtual_key: u16,
        key_down: bool,
    ) -> *mut c_void;
    fn CGEventSetFlags(event: *mut c_void, flags: u64);
    fn CGEventPostToPid(pid: i32, event: *mut c_void);
    fn CFRelease(cf: *mut c_void);
}

const HID_SYSTEM_STATE: u32 = 1;
const COMMAND_FLAG: u64 = 0x0008_0000;
const KEY_C: u16 = 0x08;

static APP_MENU_QUIT: AtomicBool = AtomicBool::new(false);

thread_local! {
    static QUIT_WATCH: RefCell<Option<Retained<QuitWatch>>> = const { RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "MyLlmQuitWatch"]
    #[ivars = ()]
    struct QuitWatch;

    impl QuitWatch {
        #[unsafe(method(onMenuAction:))]
        fn on_menu_action(&self, notification: &NSNotification) {
            mark_quit_if_terminate(notification);
        }
    }
);

impl QuitWatch {
    fn new() -> Retained<Self> {
        unsafe { msg_send![super(Self::alloc().set_ivars(())), init] }
    }
}

fn mark_quit_if_terminate(notification: &NSNotification) {
    let Some(info) = notification.userInfo() else {
        return;
    };
    let key = NSString::from_str("MenuItem");
    let Some(item) = info.objectForKey(&key) else {
        return;
    };
    let action: Option<Sel> = unsafe { msg_send![&*item, action] };
    if action == Some(sel!(terminate:)) {
        APP_MENU_QUIT.store(true, Ordering::SeqCst);
    }
}

pub fn install_quit_watch() {
    QUIT_WATCH.with(|slot| {
        if slot.borrow().is_some() {
            return;
        }
        let watch = QuitWatch::new();
        let center = NSNotificationCenter::defaultCenter();
        let name = NSString::from_str("NSMenuDidSendActionNotification");
        unsafe {
            center.addObserver_selector_name_object(&watch, sel!(onMenuAction:), Some(&name), None);
        }
        *slot.borrow_mut() = Some(watch);
    });
}

pub fn take_app_menu_quit() -> bool {
    APP_MENU_QUIT.swap(false, Ordering::SeqCst)
}

pub fn set_accessory(hidden: bool) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let policy = if hidden {
        NSApplicationActivationPolicy::Accessory
    } else {
        NSApplicationActivationPolicy::Regular
    };
    let _ = app.setActivationPolicy(policy);
    if !hidden {
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
    }
}

pub fn set_app_icon() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let data = NSData::with_bytes(assets::APP_ICON_PNG);
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    unsafe { app.setApplicationIconImage(Some(&image)) };
}

pub fn preferred_ui_langs() -> Vec<String> {
    NSLocale::preferredLanguages()
        .iter()
        .map(|tag| tag.to_string())
        .collect()
}

pub fn apply_float_chrome() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    for window in windows.iter() {
        configure_window(&window);
    }
}

fn is_result_window(window: &NSWindow) -> bool {
    window.class().name().to_bytes() == b"WinitWindow"
}

fn configure_window(window: &NSWindow) {
    if !is_result_window(window) {
        return;
    }
    window.setLevel(3);
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Transient
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    window.setHidesOnDeactivate(false);
    window.setTitlebarAppearsTransparent(true);
    window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
    window.setStyleMask(window.styleMask() | NSWindowStyleMask::FullSizeContentView);
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
}

pub fn frontmost_pid() -> Option<u32> {
    let workspace = NSWorkspace::sharedWorkspace();
    let app: Option<objc2::rc::Retained<NSRunningApplication>> = workspace.frontmostApplication();
    app.map(|app| app.processIdentifier() as u32)
}

pub fn capture_selection(source_pid: Option<u32>) -> String {
    let trusted = unsafe { AXIsProcessTrusted() };
    if trusted {
        if let Some(pid) = source_pid.filter(|pid| *pid != std::process::id()) {
            post_copy(pid as i32);
            std::thread::sleep(Duration::from_millis(250));
        }
    }
    read_clipboard()
}

fn post_copy(pid: i32) {
    unsafe {
        let source = CGEventSourceCreate(HID_SYSTEM_STATE);
        if source.is_null() {
            return;
        }
        let down = CGEventCreateKeyboardEvent(source, KEY_C, true);
        let up = CGEventCreateKeyboardEvent(source, KEY_C, false);
        if !down.is_null() {
            CGEventSetFlags(down, COMMAND_FLAG);
            CGEventPostToPid(pid, down);
            CFRelease(down);
        }
        if !up.is_null() {
            CGEventSetFlags(up, COMMAND_FLAG);
            CGEventPostToPid(pid, up);
            CFRelease(up);
        }
        CFRelease(source);
    }
}
