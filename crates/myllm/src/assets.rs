#[cfg(not(target_os = "windows"))]
pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/appicon.png");
#[cfg(target_os = "windows")]
pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/appicon-windows.png");
pub const TRAY_ICON_PNG: &[u8] = include_bytes!("../assets/trayicon.png");

#[cfg(test)]
mod tests {
    use super::{APP_ICON_PNG, TRAY_ICON_PNG};

    #[test]
    fn icons_decode() {
        let app = eframe::icon_data::from_png_bytes(APP_ICON_PNG).expect("app icon");
        assert_eq!((app.width, app.height), (1024, 1024));
        let tray = eframe::icon_data::from_png_bytes(TRAY_ICON_PNG).expect("tray icon");
        assert_eq!((tray.width, tray.height), (44, 44));
        assert_eq!(tray.width % 4, 0);
    }

    #[test]
    fn windows_app_icon_decodes() {
        let png = include_bytes!("../assets/appicon-windows.png");
        let app = eframe::icon_data::from_png_bytes(png).expect("windows app icon");
        assert_eq!((app.width, app.height), (1024, 1024));
    }
}
