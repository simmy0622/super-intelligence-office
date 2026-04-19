mod agents;
mod commands;
mod db;
mod http;
mod llm;
mod models;
mod scheduler;
mod services;

use db::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| -> Result<(), Box<dyn std::error::Error>> {
            let show = MenuItem::with_id(app, "show", "显示超级智能办公室", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("missing default window icon")?;

            TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(true);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            let state = AppState::new(app.handle())?;
            let scheduler_state = state.clone();
            let http_state = state.clone();
            app.manage(state);
            tauri::async_runtime::spawn(async move {
                scheduler::run_scheduler_loop(scheduler_state).await;
            });
            tauri::async_runtime::spawn(async move {
                if let Err(error) = http::run_http_server(http_state).await {
                    eprintln!("[http] server stopped: {error}");
                }
            });
            let _ = app.emit("scheduler-started", ());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_posts,
            commands::get_thread,
            commands::create_human_post,
            commands::reply_as_human,
            commands::like_toggle,
            commands::repost_as_human,
            commands::list_actors,
            commands::get_actor,
            commands::get_actor_toolbox,
            commands::list_agent_runs,
            commands::set_api_key,
            commands::get_settings,
            commands::set_settings,
            commands::run_agent_step,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
