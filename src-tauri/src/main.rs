// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod overlay;
mod hook;
mod game;
mod apps;
mod hotkey;
mod widgets;
mod http_api;
mod sdk_server;
mod db;
mod tray;
mod config;

use std::sync::{Arc, Mutex};
use tauri::{Listener, Manager, State};
use tauri_plugin_sql::{Builder as SqlBuilder};

use overlay::{OverlayWindow, RegionManager};
use hook::{inject_into_league, PipeMessage, PipeServer};
use game::detect::GameDetector;
use game::session::SessionManager;
use hotkey::HotkeyManager;
use widgets::WidgetManager;

struct AppState {
    overlay: Mutex<Option<OverlayWindow>>,
    region_manager: Mutex<RegionManager>,
    pipe_server: Mutex<Option<PipeServer>>,
    hook_status: Mutex<String>,
}

#[tauri::command]
async fn show_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn set_overlay_opacity(opacity: f32, state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_opacity(opacity).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn update_interactive_regions(
    regions: Vec<overlay::Rect>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_interactive_regions(regions).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn set_screen_capture_visible(
    visible: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_screen_capture_visible(visible).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn get_interactive_regions(state: State<'_, AppState>) -> Result<Vec<overlay::Rect>, String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        Ok(window.get_interactive_regions())
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
async fn inject_hook() -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Failed to get exe directory")?
        .to_path_buf();

    let dll_path = exe_dir.join("broken_latch_hook.dll");

    if !dll_path.exists() {
        return Err(format!("DLL not found at {:?}", dll_path));
    }

    inject_into_league(dll_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_hook_status(state: State<'_, AppState>) -> Result<String, String> {
    let status = state.hook_status.lock().unwrap();
    Ok(status.clone())
}

#[tauri::command]
async fn get_game_phase(
    detector: State<'_, Arc<GameDetector>>,
) -> Result<game::GamePhase, String> {
    Ok(detector.get_current_phase())
}

#[tauri::command]
async fn get_game_session(
    session_mgr: State<'_, Arc<SessionManager>>,
) -> Result<Option<game::GameSession>, String> {
    Ok(session_mgr.get_current_session())
}

#[tauri::command]
async fn register_hotkey_cmd(
    app_id: String,
    hotkey_id: String,
    keys: String,
    manager: State<'_, Arc<HotkeyManager>>,
) -> Result<i32, String> {
    manager
        .register(&app_id, &hotkey_id, &keys)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn unregister_hotkey_cmd(
    win_hotkey_id: i32,
    manager: State<'_, Arc<HotkeyManager>>,
) -> Result<(), String> {
    manager
        .unregister(win_hotkey_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_hotkey_registered(
    keys: String,
    manager: State<'_, Arc<HotkeyManager>>,
) -> Result<bool, String> {
    Ok(manager.is_hotkey_taken(&keys))
}

#[tauri::command]
async fn get_app_hotkeys(
    app_id: String,
    manager: State<'_, Arc<HotkeyManager>>,
) -> Result<Vec<hotkey::HotkeyRegistration>, String> {
    Ok(manager.get_hotkeys_for_app(&app_id))
}

#[tauri::command]
async fn create_widget_cmd(
    config: widgets::WidgetConfig,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<String, String> {
    manager.create_widget(config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_widget_cmd(
    widget_id: String,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.show_widget(&widget_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn hide_widget_cmd(
    widget_id: String,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.hide_widget(&widget_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_widget_opacity_cmd(
    widget_id: String,
    opacity: f32,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager
        .set_widget_opacity(&widget_id, opacity)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_widget_position_cmd(
    widget_id: String,
    position: widgets::Position,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager
        .set_widget_position(&widget_id, position)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_widget_click_through_cmd(
    widget_id: String,
    enabled: bool,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager
        .set_click_through(&widget_id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_widget_state_cmd(
    widget_id: String,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<widgets::WidgetState, String> {
    manager.get_state(&widget_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn destroy_widget_cmd(
    widget_id: String,
    manager: State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager
        .destroy_widget(&widget_id)
        .map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Load platform configuration
    let config = config::load_config().expect("Failed to load config");
    println!("broken-latch Platform v{}", config.version);
    println!("Configuration loaded successfully");

    let default_opacity = config.overlay.default_opacity;
    let screen_capture_visible = config.overlay.screen_capture_visible;

    // Create shared game detector, session manager, and hotkey manager
    let detector = Arc::new(GameDetector::new());
    let session_mgr = Arc::new(SessionManager::new());
    let detector_for_loop = detector.clone();

    let hotkey_manager = Arc::new(
        HotkeyManager::new().expect("Failed to create hotkey manager"),
    );
    let hotkey_manager_for_loop = hotkey_manager.clone();

    tauri::Builder::default()
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:platform.db", db::get_migrations())
                .build(),
        )
        .manage(detector)
        .manage(session_mgr)
        .manage(hotkey_manager)
        .setup(move |app| {
            println!("Platform initializing...");

            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(win) = app.get_webview_window("main") {
                    win.open_devtools();
                }
            }

            // Initialize database
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::init_db(&handle).await {
                    eprintln!("Database initialization error: {}", e);
                }
            });

            // Create overlay window
            let overlay = match OverlayWindow::create(app.handle()) {
                Ok(window) => {
                    println!("Overlay window created successfully");
                    if let Err(e) = window.set_opacity(default_opacity) {
                        eprintln!("Failed to set overlay opacity: {}", e);
                    }
                    if let Err(e) = window.set_screen_capture_visible(screen_capture_visible) {
                        eprintln!("Failed to set screen capture visibility: {}", e);
                    }
                    Some(window)
                }
                Err(e) => {
                    eprintln!("Failed to create overlay window: {}", e);
                    None
                }
            };

            // Create and register widget manager
            let widget_manager = Arc::new(WidgetManager::new(app.handle().clone()));
            app.manage(widget_manager.clone());

            // Subscribe to game phase changes for widget auto-show/hide
            let widget_mgr_for_phase = widget_manager.clone();
            app.listen("game_phase_changed", move |event| {
                if let Ok(phase_event) = serde_json::from_str::<game::PhaseChangeEvent>(event.payload()) {
                    let phase_str = format!("{:?}", phase_event.current);
                    widget_mgr_for_phase.handle_phase_change(&phase_str);
                }
            });
            println!("Widget manager initialized");

            // Store state
            app.manage(AppState {
                overlay: Mutex::new(overlay),
                region_manager: Mutex::new(RegionManager::new()),
                pipe_server: Mutex::new(None),
                hook_status: Mutex::new("Not injected".to_string()),
            });

            // Start hotkey message loop
            let app_handle_for_hotkeys = app.handle().clone();
            hotkey_manager_for_loop.start_message_loop(app_handle_for_hotkeys);
            println!("Hotkey manager started");

            // Start game detection loop
            let app_handle_for_detector = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                println!("Starting game lifecycle detection...");
                detector_for_loop
                    .start_detection_loop(app_handle_for_detector)
                    .await;
            });

            // Initialize pipe server for DLL communication
            match PipeServer::new() {
                Ok(server) => {
                    println!("Named pipe server started");
                    let server_clone = PipeServer::new().unwrap();
                    let handle = app.handle().clone();
                    std::thread::spawn(move || {
                        loop {
                            if let Some(msg) = server_clone.try_recv() {
                                let state = handle.state::<AppState>();
                                let status = match msg {
                                    PipeMessage::DX11Hooked => "DirectX 11 hooked",
                                    PipeMessage::DX12Hooked => "DirectX 12 hooked",
                                    PipeMessage::HookFailed => "Hook failed",
                                    PipeMessage::Custom(ref s) => s.as_str(),
                                };
                                *state.hook_status.lock().unwrap() = status.to_string();
                                println!("Hook status: {}", status);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    });
                    *app.state::<AppState>().pipe_server.lock().unwrap() = Some(server);
                }
                Err(e) => eprintln!("Failed to start pipe server: {}", e),
            }

            println!("Platform ready!");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            hide_overlay,
            set_overlay_opacity,
            update_interactive_regions,
            set_screen_capture_visible,
            get_interactive_regions,
            inject_hook,
            get_hook_status,
            get_game_phase,
            get_game_session,
            register_hotkey_cmd,
            unregister_hotkey_cmd,
            is_hotkey_registered,
            get_app_hotkeys,
            create_widget_cmd,
            show_widget_cmd,
            hide_widget_cmd,
            set_widget_opacity_cmd,
            set_widget_position_cmd,
            set_widget_click_through_cmd,
            get_widget_state_cmd,
            destroy_widget_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
