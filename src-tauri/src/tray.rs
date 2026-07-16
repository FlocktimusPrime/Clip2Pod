// Tray keeps the app resident after window close so the extension's capture
// listener stays reachable. Menu: Show/Hide, Quit.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

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
        .icon(app.default_window_icon().expect("bundled window icon").clone())
        .tooltip("Clip2Pod")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
