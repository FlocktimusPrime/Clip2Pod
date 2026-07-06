mod capture;
mod commands;
mod feed;
mod state;
mod tray;
mod worker;

use state::AppState;
use std::sync::Mutex;

fn install_panic_hook(config_dir: std::path::PathBuf) {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::create_dir_all(&config_dir);
        let _ = std::fs::write(
            config_dir.join("crash.log"),
            format!("{} — {info}\n", chrono::Utc::now()),
        );
        default(info);
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_dir = clip2pod_core::config::default_config_dir();
    install_panic_hook(config_dir.clone());
    let config = clip2pod_core::config::load_config(&config_dir);
    let cached_voices = config.cached_voices.clone();
    let hotkey = config.global_hotkey.clone();
    let (wake_tx, wake_rx) = tokio::sync::mpsc::unbounded_channel();

    // Registration can fail (e.g. Wayland) — the tray menu is the fallback.
    let shortcut_plugin = {
        use tauri_plugin_global_shortcut::{Builder, ShortcutState};
        // Intake, not generate: surface the window and let the frontend run
        // its normal paste-clipboard flow.
        let builder = Builder::new().with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                use tauri::Emitter;
                tray::show_main(app);
                let _ = app.emit("intake-clipboard", ());
            }
        });
        match builder.with_shortcuts([hotkey.as_str()]) {
            Ok(b) => b.build(),
            Err(e) => {
                eprintln!("global hotkey '{hotkey}' not registered: {e}");
                Builder::new().build()
            }
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(shortcut_plugin)
        .manage(AppState {
            config_dir,
            config: Mutex::new(config),
            queue: Mutex::new(Default::default()),
            voices: Mutex::new(cached_voices),
            wake_worker: wake_tx,
        })
        .setup(move |app| {
            worker::spawn(app.handle().clone(), wake_rx);
            tray::setup(app.handle())?;
            capture::serve(app.handle().clone());
            feed::serve(app.handle().clone());
            Ok(())
        })
        // Close-to-tray: the app (and the capture listener) stays resident;
        // Quit lives in the tray menu.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::clean_text,
            commands::find_junk,
            commands::get_junk_phrases,
            commands::set_junk_phrases,
            commands::list_voices,
            commands::set_voice_enabled,
            commands::enable_all_voices,
            commands::disable_all_voices,
            commands::preview_voice,
            commands::enqueue_generate,
            commands::extract_url,
            commands::get_queue,
            commands::clear_pending,
            commands::get_log,
            commands::clear_log,
            commands::get_config,
            commands::set_output_dir,
            commands::set_prefix,
            commands::set_author_gender,
            commands::set_theme,
            commands::feed_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
