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
mod phase_manager;

use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_sql::Builder as SqlBuilder;

use overlay::Rect;
use hook::{inject_into_league, PipeMessage, PipeServer};
use phase_manager::{PhaseManager, OverlayConfig};

// ---------------------------------------------------------------------------
// Application State
// ---------------------------------------------------------------------------

/// Central state object shared across all Tauri commands.
pub struct AppState {
    /// Manages per-phase window/overlay lifecycle (reads phases.json).
    pub phase_manager: Mutex<PhaseManager>,
    /// Named-pipe server that receives messages from the hook DLL.
    pipe_server: Mutex<Option<PipeServer>>,
    /// Last known hook status string.
    hook_status: Mutex<String>,
    /// When `true` the auto-detector will not override the current phase.
    /// Set by `simulate_phase`, cleared by `release_forced_phase`.
    pub force_override: Mutex<bool>,
    /// Copy of the TOML config loaded at startup.
    platform_config: config::PlatformConfig,
}

// ---------------------------------------------------------------------------
// Overlay control commands
// ---------------------------------------------------------------------------

/// Show the first active overlay window (if any).
#[tauri::command]
async fn show_overlay(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ov) = state.phase_manager.lock().unwrap().first_overlay() {
        ov.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Hide the first active overlay window (if any).
#[tauri::command]
async fn hide_overlay(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ov) = state.phase_manager.lock().unwrap().first_overlay() {
        ov.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Set opacity on the first active overlay window.
///
/// # Example (JS)
/// ```js
/// await invoke('set_overlay_opacity', { opacity: 0.8 });
/// ```
#[tauri::command]
async fn set_overlay_opacity(opacity: f32, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ov) = state.phase_manager.lock().unwrap().first_overlay() {
        ov.set_opacity(opacity).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Define which rectangular areas of the active overlay intercept mouse events.
///
/// # Example (JS)
/// ```js
/// await invoke('update_interactive_regions', {
///   regions: [{ x: 10, y: 10, width: 300, height: 200 }]
/// });
/// ```
#[tauri::command]
async fn update_interactive_regions(
    regions: Vec<Rect>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(ov) = state.phase_manager.lock().unwrap().first_overlay() {
        ov.set_interactive_regions(regions).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Toggle OBS/Discord screen-capture visibility for the active overlay.
///
/// # Example (JS)
/// ```js
/// await invoke('set_screen_capture_visible', { visible: true });
/// ```
#[tauri::command]
async fn set_screen_capture_visible(
    visible: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(ov) = state.phase_manager.lock().unwrap().first_overlay() {
        ov.set_screen_capture_visible(visible).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Return the active overlay's interactive regions.
#[tauri::command]
async fn get_interactive_regions(state: State<'_, AppState>) -> Result<Vec<Rect>, String> {
    Ok(state.phase_manager.lock().unwrap()
        .first_overlay()
        .map(|ov| ov.get_interactive_regions())
        .unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Hook commands
// ---------------------------------------------------------------------------

/// Inject the DirectX hook DLL into League of Legends.
///
/// ⚠️  Blocked by Vanguard anti-cheat.  Kept for reference only.
#[tauri::command]
async fn inject_hook() -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("no exe dir")?
        .to_path_buf();

    let dll_path = exe_dir.join("broken_latch_hook.dll");
    if !dll_path.exists() {
        return Err(format!("DLL not found at {:?}", dll_path));
    }
    inject_into_league(dll_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Return the last known hook status string.
#[tauri::command]
async fn get_hook_status(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.hook_status.lock().unwrap().clone())
}

// ---------------------------------------------------------------------------
// Phase commands
// ---------------------------------------------------------------------------

/// Force a specific game phase for development/testing.
///
/// While a phase is forced the automatic LoL process detector is suspended —
/// it keeps running in the background but will not override this phase.
/// Call `release_forced_phase` to hand control back to the detector.
///
/// # Arguments
/// * `phase` — one of: `NOT_RUNNING` | `LAUNCHING` | `IN_LOBBY` |
///   `CHAMP_SELECT` | `LOADING` | `IN_GAME` | `END_GAME`
///
/// # Example (JS)
/// ```js
/// await invoke('simulate_phase', { phase: 'IN_GAME' });
/// ```
#[tauri::command]
async fn simulate_phase(
    phase: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    *state.force_override.lock().unwrap() = true;
    state.phase_manager.lock().unwrap()
        .transition(&phase, &app_handle)
        .map_err(|e| e.to_string())
}

/// Release the dev-dashboard force override and let the auto-detector
/// resume controlling phase transitions.
///
/// # Example (JS)
/// ```js
/// await invoke('release_forced_phase');
/// ```
#[tauri::command]
async fn release_forced_phase(state: State<'_, AppState>) -> Result<(), String> {
    *state.force_override.lock().unwrap() = false;
    println!("[phase] auto-detection resumed");
    Ok(())
}

/// Return the currently active game phase string.
///
/// # Example (JS)
/// ```js
/// const phase = await invoke('get_current_phase'); // "IN_GAME"
/// ```
#[tauri::command]
async fn get_current_phase(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.phase_manager.lock().unwrap().current_phase().to_string())
}

/// Return `true` if the dev dashboard has forced a phase override.
///
/// # Example (JS)
/// ```js
/// const forced = await invoke('is_phase_forced');
/// ```
#[tauri::command]
async fn is_phase_forced(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.force_override.lock().unwrap())
}

/// Return the config for a specific overlay window by its id.
///
/// Called by overlay windows on mount so they can read their own `drag_edge`,
/// `click_through`, etc.  The window label is `"phase-{id}"` — strip the prefix
/// before calling this command.
///
/// # Example (JS — called from inside an overlay window)
/// ```js
/// import { getCurrentWindow } from '@tauri-apps/api/window';
/// const label = getCurrentWindow().label;          // "phase-ingame-kda"
/// const id    = label.replace(/^phase-/, '');      // "ingame-kda"
/// const cfg   = await invoke('get_overlay_config', { id });
/// ```
#[tauri::command]
async fn get_overlay_config(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<OverlayConfig>, String> {
    Ok(state.phase_manager.lock().unwrap()
        .get_overlay_config(&id)
        .cloned())
}

/// Return the full platform configuration loaded from `config.toml`.
///
/// # Example (JS)
/// ```js
/// const cfg = await invoke('get_platform_config');
/// ```
#[tauri::command]
async fn get_platform_config(state: State<'_, AppState>) -> Result<config::PlatformConfig, String> {
    Ok(state.platform_config.clone())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cfg = config::load_config().expect("Failed to load config");
    println!("broken-latch Platform v{}", cfg.version);

    tauri::Builder::default()
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:platform.db", db::get_migrations())
                .build(),
        )
        .setup(move |app| {
            println!("Platform initializing...");

            // Initialize database
            let db_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::init_db(&db_handle).await {
                    eprintln!("Database init error: {}", e);
                }
            });

            // Load phases.json from the bundled resources directory
            let phases_json = {
                let res_dir = app.path().resource_dir()
                    .expect("could not resolve resource dir");
                let path = res_dir.join("resources").join("phases.json");
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to read phases.json at {:?}: {}", path, e);
                        // Return a minimal valid JSON so the platform still starts
                        r#"{"NOT_RUNNING":{"windows":[],"overlays":[]}}"#.to_string()
                    })
            };

            let phase_manager = PhaseManager::from_json(&phases_json)
                .expect("invalid phases.json");

            // Register shared state
            app.manage(AppState {
                phase_manager:   Mutex::new(phase_manager),
                pipe_server:     Mutex::new(None),
                hook_status:     Mutex::new("Not injected".to_string()),
                force_override:  Mutex::new(false),
                platform_config: cfg.clone(),
            });

            // Start the named-pipe server for hook DLL messages
            match PipeServer::new() {
                Ok(server) => {
                    println!(r"Named pipe server started (\\.\pipe\broken_latch)");
                    let pipe_handle = app.handle().clone();
                    std::thread::spawn(move || loop {
                        if let Some(msg) = server.try_recv() {
                            let state  = pipe_handle.state::<AppState>();
                            let status = match msg {
                                PipeMessage::DX11Hooked     => "DirectX 11 hooked",
                                PipeMessage::DX12Hooked     => "DirectX 12 hooked",
                                PipeMessage::HookFailed     => "Hook failed",
                                PipeMessage::Custom(ref s)  => s.as_str(),
                            };
                            *state.hook_status.lock().unwrap() = status.to_string();
                            println!("[hook] {}", status);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    });
                }
                Err(e) => eprintln!("Pipe server error: {}", e),
            }

            // Start automatic LoL process + LCU phase detector
            let detector_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                game::start_detector(detector_handle).await;
            });

            println!("Platform ready — watching for League of Legends...");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Overlay control
            show_overlay,
            hide_overlay,
            set_overlay_opacity,
            update_interactive_regions,
            set_screen_capture_visible,
            get_interactive_regions,
            // Hook (legacy DLL approach)
            inject_hook,
            get_hook_status,
            // Phase management
            simulate_phase,
            release_forced_phase,
            get_current_phase,
            is_phase_forced,
            get_overlay_config,
            get_platform_config,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
