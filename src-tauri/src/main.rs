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

use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_sql::{Builder as SqlBuilder};

use overlay::{OverlayWindow, RegionManager};

struct AppState {
    overlay: Mutex<Option<OverlayWindow>>,
    region_manager: Mutex<RegionManager>,
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

#[tokio::main]
async fn main() {
    // Load platform configuration
    let config = config::load_config().expect("Failed to load config");
    println!("broken-latch Platform v{}", config.version);
    println!("Configuration loaded successfully");

    // Initialize Tauri app
    tauri::Builder::default()
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:platform.db", db::get_migrations())
                .build(),
        )
        .setup(|app| {
            println!("Platform initializing...");
            
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
                    
                    // Apply default opacity from config
                    let opacity = config.overlay.default_opacity;
                    if let Err(e) = window.set_opacity(opacity) {
                        eprintln!("Failed to set overlay opacity: {}", e);
                    }
                    
                    // Apply screen capture setting from config
                    let capture_visible = config.overlay.screen_capture_visible;
                    if let Err(e) = window.set_screen_capture_visible(capture_visible) {
                        eprintln!("Failed to set screen capture visibility: {}", e);
                    }
                    
                    Some(window)
                }
                Err(e) => {
                    eprintln!("Failed to create overlay window: {}", e);
                    None
                }
            };

            // Store state
            app.manage(AppState {
                overlay: Mutex::new(overlay),
                region_manager: Mutex::new(RegionManager::new()),
            });

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
