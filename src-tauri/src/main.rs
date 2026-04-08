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
use tauri::{Emitter, Manager, State};
use tauri_plugin_sql::{Builder as SqlBuilder};

use overlay::{OverlayWindow, RegionManager};
use hook::{inject_into_league, PipeMessage, PipeServer};

// ---------------------------------------------------------------------------
// Application State
// ---------------------------------------------------------------------------

/// Central state object shared across all Tauri commands.
///
/// Every field is wrapped in `Mutex` so Tauri can hand out `State<'_, AppState>`
/// to async command handlers safely from any thread.
struct AppState {
    /// The transparent fullscreen overlay window (created once at startup).
    overlay: Mutex<Option<OverlayWindow>>,
    /// Tracks which rectangular regions of the overlay capture mouse input.
    region_manager: Mutex<RegionManager>,
    /// Named-pipe server that receives messages from the hook DLL.
    pipe_server: Mutex<Option<PipeServer>>,
    /// Last known hook status string ("Not injected", "DirectX 11 hooked", etc.).
    hook_status: Mutex<String>,
    /// Currently simulated game phase (drives overlay content in demo mode).
    ///
    /// In production this would be set by the game lifecycle detector (Task 04).
    /// Possible values: NOT_RUNNING | LAUNCHING | IN_LOBBY | CHAMP_SELECT |
    ///                  LOADING | IN_GAME | END_GAME
    current_phase: Mutex<String>,
    /// A copy of the TOML config that was loaded at startup, exposed to the
    /// frontend via `get_platform_config` so the dev dashboard can display it.
    platform_config: config::PlatformConfig,
}

// ---------------------------------------------------------------------------
// Overlay commands
// ---------------------------------------------------------------------------

/// Show the transparent overlay window.
///
/// Call this when a game phase begins (e.g. entering CHAMP_SELECT).
/// The overlay starts hidden so it does not cover the desktop on startup.
#[tauri::command]
async fn show_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Hide the transparent overlay window.
///
/// Call this when returning to NOT_RUNNING or whenever the overlay
/// should be suppressed entirely.
#[tauri::command]
async fn hide_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Set the global opacity of the overlay window.
///
/// # Arguments
/// * `opacity` – `0.0` (fully transparent) to `1.0` (fully opaque).
///   Values are clamped by the Win32 `SetLayeredWindowAttributes` call.
///
/// # Example (JS)
/// ```js
/// await invoke('set_overlay_opacity', { opacity: 0.8 });
/// ```
#[tauri::command]
async fn set_overlay_opacity(opacity: f32, state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_opacity(opacity).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Define which rectangular areas of the overlay intercept mouse events.
///
/// Every area *outside* these rectangles remains click-through — input
/// passes straight to the game underneath.  Each installed app widget
/// registers its bounding box here so the platform can compute the
/// combined interactive region via `SetWindowRgn`.
///
/// Pass an empty `regions` list to make the entire overlay click-through.
///
/// # Example (JS)
/// ```js
/// await invoke('update_interactive_regions', {
///   regions: [{ x: 10, y: 10, width: 300, height: 200 }]
/// });
/// ```
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

/// Control whether the overlay appears in OBS / Discord screen captures.
///
/// When `visible = false` (default) the Win32 `WDA_EXCLUDEFROMCAPTURE` flag
/// hides the window from any screen-capture API — the game renders normally
/// in the stream but the overlay is invisible to streamers' audiences.
///
/// When `visible = true` the overlay is included in captures, which is useful
/// for content-creator apps that intentionally want to appear on stream.
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
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_screen_capture_visible(visible).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Return the list of currently registered interactive regions.
///
/// Useful for debugging — the dev dashboard calls this to display the
/// active widget bounding boxes.
#[tauri::command]
async fn get_interactive_regions(state: State<'_, AppState>) -> Result<Vec<overlay::Rect>, String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        Ok(window.get_interactive_regions())
    } else {
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// Hook commands (DLL injection — kept for completeness, Vanguard-unsafe)
// ---------------------------------------------------------------------------

/// Inject the DirectX hook DLL into the League of Legends process.
///
/// ⚠️  This uses classic `CreateRemoteThread` injection and is blocked by
/// Vanguard anti-cheat.  It is kept here for reference only.  The platform
/// now uses a transparent `WebviewWindow` overlay instead (Vanguard-safe).
///
/// Expects `broken_latch_hook.dll` next to the platform executable.
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

/// Return the last known hook status string.
///
/// Possible values: "Not injected" | "DirectX 11 hooked" | "DirectX 12 hooked" | "Hook failed"
#[tauri::command]
async fn get_hook_status(state: State<'_, AppState>) -> Result<String, String> {
    let status = state.hook_status.lock().unwrap();
    Ok(status.clone())
}

// ---------------------------------------------------------------------------
// Game phase simulation commands
// ---------------------------------------------------------------------------

/// Simulate a game phase transition for demo / development purposes.
///
/// In production the phase is set automatically by the game lifecycle
/// detector (Task 04: `game::detect` + `game::lcu`).  During development
/// — before the detector is wired up — this command lets you drive the
/// overlay UI through all seven phases from the dev dashboard.
///
/// # What happens internally
/// 1. `current_phase` in `AppState` is updated.
/// 2. A `phase-changed` Tauri event is emitted to **all** windows.
///    The overlay window listens for this event and swaps its React component.
/// 3. The overlay is automatically shown or hidden:
///    - `NOT_RUNNING` → hidden (no overlay on desktop)
///    - all other phases → shown (overlay visible over game)
///
/// # Arguments
/// * `phase` – one of: `NOT_RUNNING` | `LAUNCHING` | `IN_LOBBY` |
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
    // 1. Persist the new phase in state
    *state.current_phase.lock().unwrap() = phase.clone();

    // 2. Broadcast the phase to every window so the overlay React app can react
    app_handle
        .emit("phase-changed", phase.clone())
        .map_err(|e| e.to_string())?;

    // 3. Auto-manage overlay visibility based on phase
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        match phase.as_str() {
            "NOT_RUNNING" => window.hide().map_err(|e| e.to_string())?,
            _             => window.show().map_err(|e| e.to_string())?,
        }
    }

    println!("[phase] → {}", phase);
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
    Ok(state.current_phase.lock().unwrap().clone())
}

/// Return the full platform configuration that was loaded from `config.toml`
/// at startup.
///
/// The dev dashboard uses this to display the live config values so developers
/// can see which TOML settings are currently active without opening the file.
///
/// # Example (JS)
/// ```js
/// const cfg = await invoke('get_platform_config');
/// console.log(cfg.overlay.default_opacity); // 1.0
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
    // Load platform configuration from %APPDATA%/broken-latch/config.toml
    // (creates a default file on first run)
    let cfg = config::load_config().expect("Failed to load config");
    println!("broken-latch Platform v{}", cfg.version);
    println!("Configuration loaded successfully");

    tauri::Builder::default()
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:platform.db", db::get_migrations())
                .build(),
        )
        .setup(move |app| {
            println!("Platform initializing...");

            // Initialize the SQLite database (runs pending migrations)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::init_db(&handle).await {
                    eprintln!("Database initialization error: {}", e);
                }
            });

            // Create the transparent fullscreen overlay window
            let overlay = match OverlayWindow::create(app.handle()) {
                Ok(window) => {
                    println!("Overlay window created successfully");

                    if let Err(e) = window.set_opacity(cfg.overlay.default_opacity) {
                        eprintln!("Failed to set overlay opacity: {}", e);
                    }
                    if let Err(e) = window.set_screen_capture_visible(cfg.overlay.screen_capture_visible) {
                        eprintln!("Failed to set screen capture visibility: {}", e);
                    }

                    Some(window)
                }
                Err(e) => {
                    eprintln!("Failed to create overlay window: {}", e);
                    None
                }
            };

            // Register all shared state
            app.manage(AppState {
                overlay:         Mutex::new(overlay),
                region_manager:  Mutex::new(RegionManager::new()),
                pipe_server:     Mutex::new(None),
                hook_status:     Mutex::new("Not injected".to_string()),
                current_phase:   Mutex::new("NOT_RUNNING".to_string()),
                platform_config: cfg.clone(),
            });

            // Start the named-pipe server that receives messages from the hook DLL
            match PipeServer::new() {
                Ok(server) => {
                    println!(r"Named pipe server started (\.\pipe\broken_latch)");

                    let server_clone = PipeServer::new().unwrap();
                    let handle = app.handle().clone();
                    std::thread::spawn(move || {
                        loop {
                            if let Some(msg) = server_clone.try_recv() {
                                let state = handle.state::<AppState>();
                                let status = match msg {
                                    PipeMessage::DX11Hooked  => "DirectX 11 hooked",
                                    PipeMessage::DX12Hooked  => "DirectX 12 hooked",
                                    PipeMessage::HookFailed  => "Hook failed",
                                    PipeMessage::Custom(ref s) => s.as_str(),
                                };
                                *state.hook_status.lock().unwrap() = status.to_string();
                                println!("[hook] {}", status);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    });

                    *app.state::<AppState>().pipe_server.lock().unwrap() = Some(server);
                }
                Err(e) => eprintln!("Failed to start pipe server: {}", e),
            }

            println!("Platform ready! Open the Dev Simulator window to begin.");
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
            // Game phase simulation
            simulate_phase,
            get_current_phase,
            get_platform_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
