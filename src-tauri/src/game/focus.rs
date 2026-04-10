//! Overlay focus monitor — shows/hides IN_GAME overlays based on which window
//! the player is currently looking at.
//!
//! # Behaviour
//!
//! Polls the foreground window every 150 ms (fast enough to feel instant, cheap
//! enough not to matter).  Compares the foreground process against:
//!
//! - **Our own process** (PID from `GetCurrentProcessId`) — player clicked an
//!   overlay widget; keep overlays visible.
//! - **League of Legends.exe** (PID stored in `AppState.league_pid` by the
//!   game detector) — game is in focus; show overlays.
//! - **Anything else** — player Alt-Tabbed, opened a browser, etc.; hide
//!   overlays immediately.
//!
//! # Fullscreen support
//!
//! When the game runs in fullscreen and our overlays regain visibility,
//! `reassert_topmost()` is called on every overlay.  This re-issues
//! `SetWindowPos(HWND_TOPMOST, …)` so Windows pushes our windows above the
//! game's DirectX swap chain — the standard technique for fullscreen overlays
//! on Windows 10/11 (DWM keeps the compositor active even for "fullscreen" DX).
//!
//! # Dev / force-override mode
//!
//! When the dev dashboard has forced a phase via `simulate_phase`, League is
//! not actually running, so `league_pid` is `None`.  In that case we treat
//! our own process as the sole valid "show" condition, so the overlay windows
//! remain visible whenever the developer is looking at them or at the dev
//! dashboard.

use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::Threading::GetCurrentProcessId;

use crate::AppState;

/// Start the focus-monitor loop.  Spawned as a tokio task in `main.rs`.
pub async fn start_focus_monitor(app: AppHandle) {
    let our_pid = unsafe { GetCurrentProcessId() };
    let mut overlay_visible = true;   // tracks last known show/hide state

    loop {
        tokio::time::sleep(Duration::from_millis(150)).await;

        let state = app.state::<AppState>();

        // Only manage visibility during IN_GAME; skip (and reset) otherwise.
        let phase = state.phase_manager.lock().unwrap().current_phase().to_string();
        if phase != "IN_GAME" {
            overlay_visible = true;  // reset so first IN_GAME entry shows immediately
            continue;
        }

        let should_show = foreground_is_game_or_ours(&state, our_pid);

        if should_show != overlay_visible {
            overlay_visible = should_show;
            state.phase_manager.lock().unwrap()
                .set_overlay_visibility(overlay_visible);
            println!(
                "[focus] overlays {}",
                if overlay_visible { "shown  (game focused)" } else { "hidden (game lost focus)" }
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the current foreground window belongs to the League game
/// process OR to our own platform process (overlay widget clicked).
fn foreground_is_game_or_ours(state: &AppState, our_pid: u32) -> bool {
    let fg_pid = get_foreground_pid();
    if fg_pid == 0 { return true; }   // can't determine — keep showing

    // Our own process (dev dashboard or an overlay window is focused)
    if fg_pid == our_pid { return true; }

    // The League game process
    let league_pid = *state.league_pid.lock().unwrap();
    match league_pid {
        Some(lpid) => fg_pid == lpid,
        // Force-override / dev mode: League isn't running.
        // Keep overlays visible as long as the developer's focus stays in-app.
        None => false,
    }
}

/// Get the Win32 process ID that owns the current foreground window.
/// Returns 0 if `GetForegroundWindow` returns a null HWND.
fn get_foreground_pid() -> u32 {
    unsafe {
        let hwnd = GetForegroundWindow();
        // HWND(ptr) — check for null
        if hwnd.0.is_null() { return 0; }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}
