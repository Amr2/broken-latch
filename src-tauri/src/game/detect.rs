use sysinfo::System;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use super::{GamePhase, PhaseChangeEvent};

pub struct GameDetector {
    current_phase: Arc<Mutex<GamePhase>>,
    system: Mutex<System>,
}

// SAFETY: System is only accessed behind a Mutex, never shared across threads unsynchronized
unsafe impl Send for GameDetector {}
unsafe impl Sync for GameDetector {}

impl GameDetector {
    pub fn new() -> Self {
        Self {
            current_phase: Arc::new(Mutex::new(GamePhase::NotRunning)),
            system: Mutex::new(System::new()),
        }
    }

    /// Start the detection loop (runs in background task)
    pub async fn start_detection_loop(
        self: Arc<Self>,
        app_handle: tauri::AppHandle,
    ) {
        let mut ticker = interval(Duration::from_secs(3));

        loop {
            ticker.tick().await;

            // Refresh process list
            {
                let mut system = self.system.lock().unwrap();
                system.refresh_processes();
            }

            let (league_client_running, league_game_running) = {
                let system = self.system.lock().unwrap();
                (
                    Self::is_process_running(&system, "LeagueClient.exe"),
                    Self::is_process_running(&system, "League of Legends.exe")
                        || Self::is_process_running(&system, "LeagueOfLegends.exe"),
                )
            };

            let current = *self.current_phase.lock().unwrap();
            let new_phase = self
                .determine_phase(league_client_running, league_game_running, current)
                .await;

            if new_phase != current {
                self.emit_phase_change(&app_handle, current, new_phase);
                *self.current_phase.lock().unwrap() = new_phase;
            }
        }
    }

    fn is_process_running(system: &System, process_name: &str) -> bool {
        system.processes().iter().any(|(_, process)| {
            process.name().eq_ignore_ascii_case(process_name)
        })
    }

    /// Determine current game phase based on process state and API queries
    async fn determine_phase(
        &self,
        client_running: bool,
        game_running: bool,
        current_phase: GamePhase,
    ) -> GamePhase {
        if !client_running && !game_running {
            return GamePhase::NotRunning;
        }

        if client_running && !game_running {
            // Client is open — check if in champion select via LCU
            if self.is_in_champ_select().await {
                return GamePhase::ChampSelect;
            }
            return GamePhase::InLobby;
        }

        if game_running {
            // Query Live Client Data API
            match self.query_live_client_data().await {
                Ok(game_stats) => {
                    if game_stats.game_time < 10.0 {
                        return GamePhase::Loading;
                    }
                    return GamePhase::InGame;
                }
                Err(_) => {
                    // API not available yet — likely still loading
                    if current_phase == GamePhase::ChampSelect
                        || current_phase == GamePhase::InLobby
                    {
                        return GamePhase::Loading;
                    }
                    return current_phase;
                }
            }
        }

        current_phase
    }

    /// Query Riot's Live Client Data API
    async fn query_live_client_data(&self) -> Result<LiveGameStats, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let response = client
            .get("https://127.0.0.1:2999/liveclientdata/gamestats")
            .timeout(Duration::from_secs(2))
            .send()
            .await?;

        let stats: LiveGameStats = response.json().await?;
        Ok(stats)
    }

    /// Check if in champion select via LCU endpoint
    async fn is_in_champ_select(&self) -> bool {
        // Full LCU WebSocket implementation is in lcu.rs
        // Here we do a quick HTTP check against the LCU REST API
        let lcu = super::lcu::LCUClient::new();
        match lcu.check_champ_select().await {
            Ok(in_select) => in_select,
            Err(_) => false,
        }
    }

    fn emit_phase_change(
        &self,
        app_handle: &tauri::AppHandle,
        previous: GamePhase,
        current: GamePhase,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let event = PhaseChangeEvent {
            previous,
            current,
            timestamp,
        };

        println!("Game phase changed: {:?} -> {:?}", previous, current);

        // Emit Tauri event to all frontend listeners
        use tauri::Emitter;
        let _ = app_handle.emit("game_phase_changed", &event);

        // TODO: Task 08 will add HTTP webhook broadcasting here
    }

    pub fn get_current_phase(&self) -> GamePhase {
        *self.current_phase.lock().unwrap()
    }
}

#[derive(Debug, serde::Deserialize)]
struct LiveGameStats {
    #[serde(rename = "gameTime")]
    game_time: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_enum_serialization() {
        let phase = GamePhase::InGame;
        let json = serde_json::to_string(&phase).unwrap();
        assert_eq!(json, r#""InGame""#);
    }

    #[test]
    fn test_phase_change_event() {
        let event = PhaseChangeEvent {
            previous: GamePhase::Loading,
            current: GamePhase::InGame,
            timestamp: 1712345678,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("InGame"));
        assert!(json.contains("Loading"));
    }

    #[test]
    fn test_detector_initialization() {
        let detector = GameDetector::new();
        assert_eq!(detector.get_current_phase(), GamePhase::NotRunning);
    }
}
