// Tray keeps the app resident after window close so the extension's capture
// listener stays reachable. Menu: Show/Hide, Quit.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

// ponytail: static lavender Feed glyph (branding/feed/png/feed-tray-app-32.png:
// the 16 px small cut at 2x, so the OS downscale stays on the pixel grid), not
// theme-reactive. Upgrade path if ever reported illegible on a light taskbar:
// pick feed-tray-white-* / the black small cut from the OS theme at startup.
const TRAY_ICON: &[u8] = include_bytes!("../assets/tray-icon.png");

pub fn toggle_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    // A minimized window still reports is_visible() == true; treat it as "show".
    let minimized = window.is_minimized().unwrap_or(false);
    if !minimized && window.is_visible().unwrap_or(true) {
        let _ = window.hide();
    } else {
        show_main(app);
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        // Windows denies SetForegroundWindow from a background process; the
        // always-on-top pulse reliably brings the window to front anyway.
        let _ = window.set_always_on_top(true);
        let _ = window.set_focus();
        let _ = window.set_always_on_top(false);
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Show / Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &quit])?;

    TrayIconBuilder::new()
        .icon(tauri::image::Image::from_bytes(TRAY_ICON)?)
        .tooltip("Clip2Pod")
        .menu(&menu)
        // Left-click surfaces the window; right-click still opens the menu.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
