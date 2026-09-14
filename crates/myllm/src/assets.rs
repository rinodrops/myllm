pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/appicon.png");
pub const TRAY_ICON_PNG: &[u8] = include_bytes!("../assets/trayicon.png");

#[cfg(test)]
mod tests {
    use super::{APP_ICON_PNG, TRAY_ICON_PNG};

    #[test]
    fn icons_decode() {
        let app = eframe::icon_data::from_png_bytes(APP_ICON_PNG).expect("app icon");
        assert_eq!((app.width, app.height), (512, 512));
        let tray = eframe::icon_data::from_png_bytes(TRAY_ICON_PNG).expect("tray icon");
        assert_eq!((tray.width, tray.height), (44, 44));
        assert_eq!(tray.width % 4, 0);
    }
}
