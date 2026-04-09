//! Game process detector — runs as a background tokio task.
//!
//! Polls the process list every two seconds using `sysinfo` and translates the
//! observed process state + LCU gameflow phase into a broken-latch phase
//! string.  When the detected phase differs from the current platform phase
//! **and** the dev dashboard has not forced an override, the PhaseManager is
//! instructed to transition.
//!
//! # Detection logic
//!
//! ```text
//! Neither LeagueClient.exe nor League of Legends.exe running
//!     → NOT_RUNNING
//!
//! LeagueClient.exe running (League of Legends.exe not yet running)
//!     → read lockfile from LeagueClient process path
//!     → query LCU  /lol-gameflow/v1/gameflow-phase
//!     → map LCU phase  →  IN_LOBBY | CHAMP_SELECT | LOADING | END_GAME
//!
//! League of Legends.exe running (the in-game process)
//!     → query https://127.0.0.1:2999/liveclientdata/gamestats
//!     → 200 OK  → IN_GAME
//!     → error   → LOADING  (game is still on loading screen)
//!
//! LeagueClient.exe just launched (lockfile not yet readable)
//!     → LAUNCHING
//! ```
//!
//! # Force-override
//! When the dev dashboard calls `simulate_phase`, `AppState.force_override`
//! is set to `true`.  The detector keeps running but will not call
//! `phase_manager.transition()` while that flag is set.  The dev dashboard
//! can call `release_forced_phase` to hand control back to the detector.

use std::path::PathBuf;
use reqwest::Client;
use sysinfo::System;
use tauri::Manager;
use tauri::AppHandle;

use super::lcu::{LcuClient, lcu_phase_to_platform};
use crate::AppState;

/// Start the background game-detection loop.
///
/// Spawned via `tauri::async_runtime::spawn` inside `main.rs setup`.
pub async fn start_detector(app: AppHandle) {
    // One reqwest client reused for every poll iteration (keeps connection pool alive)
    let live_client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let mut sys = System::new();

    loop {
        sys.refresh_processes();

        let detected = detect_phase(&sys, &live_client).await;

        let state = app.state::<AppState>();

        // Skip transition if the dev dashboard is in force-override mode
        if !*state.force_override.lock().unwrap() {
            let current = state.phase_manager.lock().unwrap().current_phase().to_string();
            if detected != current {
                let mut pm = state.phase_manager.lock().unwrap();
                if let Err(e) = pm.transition(&detected, &app) {
                    eprintln!("[detector] transition error: {}", e);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

// ---------------------------------------------------------------------------
// Phase detection logic
// ---------------------------------------------------------------------------

async fn detect_phase(sys: &System, live_client: &Client) -> String {
    let client_running = process_running(sys, "LeagueClient");
    let game_running   = process_running(sys, "League of Legends")
        || process_running(sys, "LeagueOfLegends");

    if !client_running && !game_running {
        return "NOT_RUNNING".to_string();
    }

    if game_running {
        // Determine LOADING vs IN_GAME via live-client data endpoint
        let live_active = LcuClient::is_live_game_active(live_client).await;
        return if live_active { "IN_GAME" } else { "LOADING" }.to_string();
    }

    // LeagueClient.exe is running — query LCU for lobby/champ-select/etc.
    let lockfile = find_lockfile(sys);

    let lcu = match lockfile.as_deref().and_then(|p| LcuClient::from_lockfile(p)) {
        Some(c) => c,
        None => {
            // Client is running but lockfile not readable yet → still launching
            return "LAUNCHING".to_string();
        }
    };

    match lcu.gameflow_phase().await {
        Some(lcu_phase) => {
            // Re-check live-game status for InProgress disambiguation
            let live_active = if lcu_phase == "InProgress" {
                LcuClient::is_live_game_active(lcu.http_client()).await
            } else {
                false
            };
            lcu_phase_to_platform(&lcu_phase, live_active).to_string()
        }
        None => "IN_LOBBY".to_string(),  // LCU reachable but phase query failed
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if any running process name contains `name_fragment`.
fn process_running(sys: &System, name_fragment: &str) -> bool {
    sys.processes().values().any(|p| {
        p.name().to_string().contains(name_fragment)
    })
}

/// Locate the LCU lockfile by inspecting the LeagueClient process path.
///
/// Falls back to common install directory paths if the process path cannot
/// be read.
fn find_lockfile(sys: &System) -> Option<PathBuf> {
    // Preferred: derive path from the running process
    for process in sys.processes().values() {
        let name = process.name().to_string();
        if name.contains("LeagueClient") {
            if let Some(exe) = process.exe() {
                if let Some(dir) = exe.parent() {
                    let candidate = dir.join("lockfile");
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
    }

    // Fallback: common install directories
    super::lcu::find_lockfile_default()
}
