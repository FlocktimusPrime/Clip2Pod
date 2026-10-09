mod capture;
mod commands;
mod feed;
mod firewall;
mod rip_commands;
mod rip_worker;
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
    const HOTKEY: &str = "Ctrl+Alt+G";
    let config_dir = clip2pod_core::config::default_config_dir();
    install_panic_hook(config_dir.clone());
    let config = clip2pod_core::config::load_config(&config_dir);
    let authors = clip2pod_core::authors::load_authors(&config_dir);
    let cached_voices = config.cached_voices.clone();
    let start_minimized = config.start_minimized;
    let launch_at_startup = config.launch_at_startup;
    let feed_token = feed::load_or_create_token(&config_dir);
    let (wake_tx, wake_rx) = tokio::sync::mpsc::unbounded_channel();

    // RIP mode: its own config/log subdir.
    let rip_config_dir = config_dir.join("rip");
    let rip_config = clip2pod_rip::config::load_config(&rip_config_dir);
    let (rip_wake_tx, rip_wake_rx) = tokio::sync::mpsc::unbounded_channel();

    // Intake, not generate: surface the window and let the frontend run
    // its normal paste-clipboard flow.
    let shortcut_plugin = {
        use tauri_plugin_global_shortcut::{Builder, ShortcutState};
        Builder::new()
            .with_handler(|app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    use tauri::Emitter;
                    tray::show_main(app);
                    let _ = app.emit("intake-clipboard", ());
                }
            })
            .build()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(shortcut_plugin)
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .manage(AppState {
            config_dir,
            config: Mutex::new(config),
            authors: Mutex::new(authors),
            queue: Mutex::new(Default::default()),
            voices: Mutex::new(cached_voices),
            wake_worker: wake_tx,
            cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            feed_token: Mutex::new(feed_token),
            rip_config_dir,
            rip_config: Mutex::new(rip_config),
            rip_queue: Mutex::new(Default::default()),
            rip_wake: rip_wake_tx,
            rip_running: Mutex::new(None),
        })
        .setup(move |app| {
            // Registration can fail (hotkey taken by another app, Wayland, …) —
            // the tray menu is the fallback, so never let this kill startup.
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                if let Err(e) = app.global_shortcut().register(HOTKEY) {
                    eprintln!("global hotkey '{HOTKEY}' not registered: {e}");
                }
            }
            // Reassert the OS-level autostart entry to match the saved
            // preference (covers manual removal, reinstalls, etc). Unanswered
            // (None) is left alone until the first-run prompt sets it.
            if let Some(launch_at_startup) = launch_at_startup {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                let result = if launch_at_startup {
                    autolaunch.enable()
                } else {
                    autolaunch.disable()
                };
                if let Err(e) = result {
                    eprintln!("failed to sync autostart setting: {e}");
                }
            }
            worker::spawn(app.handle().clone(), wake_rx);
            rip_worker::spawn(app.handle().clone(), rip_wake_rx);
            tray::setup(app.handle())?;
            capture::serve(app.handle().clone());
            feed::serve(app.handle().clone());
            // Window is created hidden (visible:false in tauri.conf.json);
            // only surface it when the user hasn't opted into tray-only start.
            if !start_minimized {
                use tauri::Manager;
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                }
            }
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
            commands::default_junk_phrases,
            commands::suggest_junk,
            commands::list_voices,
            commands::set_voice_enabled,
            commands::enable_all_voices,
            commands::disable_all_voices,
            commands::preview_voice,
            commands::enqueue_generate,
            commands::extract_url,
            commands::get_queue,
            commands::clear_pending,
            commands::cancel_current,
            commands::get_log,
            commands::clear_log,
            commands::get_config,
            commands::set_output_dir,
            commands::set_prefix,
            commands::set_author_gender,
            commands::lookup_author_gender,
            commands::list_authors,
            commands::upsert_author_gender,
            commands::rename_author,
            commands::delete_author,
            commands::merge_authors,
            commands::set_theme,
            commands::set_start_minimized,
            commands::set_launch_at_startup,
            commands::episode_count,
            commands::delete_all_episodes,
            commands::feed_url,
            commands::reset_feed_url,
            commands::firewall_help,
            rip_commands::rip_enqueue,
            rip_commands::rip_get_queue,
            rip_commands::rip_clear_pending,
            rip_commands::rip_stop_job,
            rip_commands::rip_get_log,
            rip_commands::rip_clear_log,
            rip_commands::rip_get_config,
            rip_commands::rip_set_output_dir,
            rip_commands::rip_set_args_template,
            rip_commands::rip_set_ytdlp_path,
            rip_commands::rip_list_episodes,
            rip_commands::rip_delete_episode,
            rip_commands::rip_delete_all_episodes,
            rip_commands::rip_feed_url,
            rip_commands::rip_doctor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
