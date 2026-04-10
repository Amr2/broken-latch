//! Overlay focus monitor — soft-shows/hides IN_GAME overlays based on which
//! window the player is currently looking at.
//!
//! # Show / hide strategy
//!
//! We use **opacity-based transitions** (`soft_show` / `soft_hide`) instead of
//! Win32 `ShowWindow` / `HideWindow`.  This is critical:
//!
//! - `ShowWindow(SW_SHOW)` activates the window → steals foreground → the focus
//!   monitor sees game lost focus → hides overlays → game regains focus → shows
//!   overlays → infinite glitch loop.
//! - `ShowWindow(SW_HIDE)` on a TOPMOST window while a DX game holds the display
//!   can force the game out of fullscreen (swap chain disruption).
//!
//! With opacity transitions the Win32 WS_VISIBLE flag and Z-order never change,
//! so the DirectX swap chain is never disturbed.
//!
//! # Debounce
//!
//! Hide transitions are debounced by 250 ms.  A brief focus blip (e.g. a
//! Windows notification, a tooltip, dragging our own overlay) won't flash the
//! overlays away.  Show transitions are immediate.
//!
//! # Dev / force-override mode
//!
//! When the dev dashboard has forced a phase, `league_pid` is `None` (League is
//! not running).  We keep overlays visible as long as our own process is in the
//! foreground so the developer can interact with the overlay widgets.

use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::Threading::GetCurrentProcessId;

use crate::AppState;

const POLL_MS:    u64 = 150;   // foreground-window check interval
const HIDE_DEBOUNCE_MS: u64 = 250; // wait this long before actually hiding

/// Start the focus-monitor loop.  Spawned as a tokio task in `main.rs`.
pub async fn start_focus_monitor(app: AppHandle) {
    let our_pid = unsafe { GetCurrentProcessId() };

    // Whether overlays are currently soft-visible
    let mut overlay_visible = true;

    // When we first detected "should hide", record the time.
    // We only actually hide after HIDE_DEBOUNCE_MS of continuous "should hide".
    let mut hide_since: Option<Instant> = None;

    loop {
        tokio::time::sleep(Duration::from_millis(POLL_MS)).await;

        let state = app.state::<AppState>();

        // Outside IN_GAME: reset state and do nothing.
        // soft_show is called when we enter IN_GAME (overlays created visible).
        {
            let phase = state.phase_manager.lock().unwrap().current_phase().to_string();
            if phase != "IN_GAME" {
                overlay_visible = true;
                hide_since = None;
                continue;
            }
        }

        let should_show = foreground_is_game_or_ours(&state, our_pid);

        if should_show {
            // ── Show path: immediate, clear any pending hide debounce ────────
            hide_since = None;
            if !overlay_visible {
                overlay_visible = true;
                state.phase_manager.lock().unwrap().set_overlay_visibility(true);
                println!("[focus] overlays shown  (game focused)");
            }
        } else {
            // ── Hide path: debounced ─────────────────────────────────────────
            if overlay_visible {
                let since = hide_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= Duration::from_millis(HIDE_DEBOUNCE_MS) {
                    overlay_visible = false;
                    hide_since = None;
                    state.phase_manager.lock().unwrap().set_overlay_visibility(false);
                    println!("[focus] overlays hidden (game lost focus)");
                }
                // else: still within debounce window, do nothing yet
            } else {
                // Already hidden — clear any stale debounce timer
                hide_since = None;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the current foreground window belongs to the League game
/// process OR to our own platform process (overlay widget or dev dashboard).
fn foreground_is_game_or_ours(state: &AppState, our_pid: u32) -> bool {
    let fg_pid = get_foreground_pid();
    if fg_pid == 0 { return true; }  // indeterminate — keep showing

    // Our own process: dev dashboard or one of the overlay windows is focused
    if fg_pid == our_pid { return true; }

    // League game process (updated every 2 s by game::detect)
    match *state.league_pid.lock().unwrap() {
        Some(lpid) => fg_pid == lpid,
        // Force-override / dev mode — League not running.
        // Hide when focus is not in our own app.
        None => false,
    }
}

/// Get the Win32 process ID that owns the current foreground window.
fn get_foreground_pid() -> u32 {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() { return 0; }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}
