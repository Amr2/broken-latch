# Task 04: Game Lifecycle Detector

**Platform: broken-latch**  
**Dependencies:** Task 01 (Project Setup)  
**Estimated Complexity:** High  
**Priority:** P0 (Critical Path)

---

## Objective

Implement the game lifecycle detection system that monitors League of Legends process state, detects game phases (NotRunning, InLobby, ChampSelect, Loading, InGame, EndGame), and broadcasts phase change events to all registered apps. This is the heartbeat of the platform — every app depends on knowing what phase the game is in.

---

## Context

The game lifecycle detector is responsible for:

- Polling for `LeagueOfLegends.exe` and `LeagueClient.exe` process existence
- Querying Riot's local Live Client Data API (`https://127.0.0.1:2999/liveclientdata/gamestats`)
- Connecting to the League Client Update (LCU) WebSocket for champion select events
- Maintaining current phase state
- Emitting `PhaseChangeEvent` to all apps via Tauri events and HTTP webhooks

Apps use this to show/hide widgets automatically (e.g., "show Hunter Mode loading panel only during Loading phase").

---

## What You Need to Build

### 1. Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
sysinfo = "0.30"                    # Process enumeration
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tokio-tungstenite = "0.21"          # WebSocket client for LCU
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.11"
```

### 2. Game Phase Data Model (`src-tauri/src/game/mod.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    NotRunning,
    Launching,
    InLobby,
    ChampSelect,
    Loading,
    InGame,
    EndGame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseChangeEvent {
    pub previous: GamePhase,
    pub current: GamePhase,
    pub timestamp: i64,  // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub ally_team: Vec<SessionPlayer>,
    pub enemy_team: Vec<SessionPlayer>,
    pub local_player: SessionPlayer,
    pub game_mode: String,
    pub map_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPlayer {
    pub summoner_name: String,
    pub puuid: String,
    pub champion_id: i32,
    pub champion_name: String,
    pub team_id: i32,
    pub position: Option<String>,  // "TOP", "JUNGLE", etc.
}

pub mod detect;
pub mod lcu;
pub mod session;
```

### 3. Process Detection (`src-tauri/src/game/detect.rs`)

```rust
use sysinfo::{System, SystemExt, ProcessExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use super::{GamePhase, PhaseChangeEvent};

pub struct GameDetector {
    current_phase: Arc<Mutex<GamePhase>>,
    system: System,
}

impl GameDetector {
    pub fn new() -> Self {
        Self {
            current_phase: Arc::new(Mutex::new(GamePhase::NotRunning)),
            system: System::new_all(),
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

            // Update process information
            let mut system = self.system.clone();
            system.refresh_processes();

            // Check for League processes
            let league_client_running = self.is_process_running(&system, "LeagueClient.exe");
            let league_game_running = self.is_process_running(&system, "League of Legends.exe");

            let current = *self.current_phase.lock().unwrap();
            let new_phase = self.determine_phase(
                league_client_running,
                league_game_running,
                current,
            ).await;

            if new_phase != current {
                self.emit_phase_change(
                    &app_handle,
                    current,
                    new_phase,
                ).await;

                *self.current_phase.lock().unwrap() = new_phase;
            }
        }
    }

    fn is_process_running(&self, system: &System, process_name: &str) -> bool {
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
                    } else {
                        return GamePhase::InGame;
                    }
                }
                Err(_) => {
                    // API not available yet — likely still loading
                    if current_phase == GamePhase::ChampSelect {
                        return GamePhase::Loading;
                    }
                    return current_phase;
                }
            }
        }

        current_phase
    }

    /// Query Riot's Live Client Data API
    async fn query_live_client_data(&self) -> Result<LiveGameStats, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)  // LCU uses self-signed cert
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
        // This requires LCU auth — implemented in lcu.rs
        // For now, simplified implementation
        false
    }

    async fn emit_phase_change(
        &self,
        app_handle: &tauri::AppHandle,
        previous: GamePhase,
        current: GamePhase,
    ) {
        let event = PhaseChangeEvent {
            previous,
            current,
            timestamp: chrono::Utc::now().timestamp(),
        };

        log::info!("Game phase changed: {:?} -> {:?}", previous, current);

        // Emit Tauri event to all frontend listeners
        app_handle.emit_all("game_phase_changed", &event).ok();

        // TODO: Task 08 will add HTTP webhook broadcasting here
    }

    pub fn get_current_phase(&self) -> GamePhase {
        *self.current_phase.lock().unwrap()
    }
}

#[derive(Debug, Deserialize)]
struct LiveGameStats {
    #[serde(rename = "gameTime")]
    game_time: f64,
}
```

### 4. LCU WebSocket Client (`src-tauri/src/game/lcu.rs`)

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use super::session::SessionPlayer;

pub struct LCUClient {
    auth_token: Option<String>,
    port: Option<u16>,
}

impl LCUClient {
    pub fn new() -> Self {
        Self {
            auth_token: None,
            port: None,
        }
    }

    /// Find LCU credentials from lockfile
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Read League Client lockfile to get auth credentials
        let lockfile_path = self.get_lockfile_path()?;
        let lockfile_content = tokio::fs::read_to_string(lockfile_path).await?;

        // Parse lockfile: LeagueClient:{pid}:{port}:{password}:{protocol}
        let parts: Vec<&str> = lockfile_content.split(':').collect();
        if parts.len() < 5 {
            return Err("Invalid lockfile format".into());
        }

        self.port = Some(parts[2].parse()?);
        self.auth_token = Some(parts[3].to_string());

        Ok(())
    }

    /// Get champion select session data
    pub async fn get_champ_select_session(&self) -> Result<ChampSelectSession, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let url = format!(
            "https://127.0.0.1:{}/lol-champ-select/v1/session",
            self.port.ok_or("LCU not connected")?
        );

        let response = client
            .get(&url)
            .basic_auth("riot", Some(self.auth_token.as_ref().ok_or("No auth token")?))
            .send()
            .await?;

        let session: ChampSelectSession = response.json().await?;
        Ok(session)
    }

    fn get_lockfile_path(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        // Windows default path
        let local_app_data = std::env::var("LOCALAPPDATA")?;
        let path = std::path::PathBuf::from(local_app_data)
            .join("Riot Games")
            .join("League of Legends")
            .join("lockfile");

        Ok(path)
    }

    /// Subscribe to LCU WebSocket events
    pub async fn subscribe_events(
        &self,
        event_handler: impl Fn(LCUEvent) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_url = format!(
            "wss://127.0.0.1:{}/",
            self.port.ok_or("LCU not connected")?
        );

        // TODO: Full WebSocket implementation
        // For now, we'll use HTTP polling in Task 07

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ChampSelectSession {
    pub actions: Vec<Vec<ChampSelectAction>>,
    #[serde(rename = "myTeam")]
    pub my_team: Vec<ChampSelectPlayer>,
    #[serde(rename = "theirTeam")]
    pub their_team: Vec<ChampSelectPlayer>,
}

#[derive(Debug, Deserialize)]
pub struct ChampSelectPlayer {
    #[serde(rename = "summonerId")]
    pub summoner_id: u64,
    #[serde(rename = "championId")]
    pub champion_id: i32,
    #[serde(rename = "championPickIntent")]
    pub champion_pick_intent: i32,
}

#[derive(Debug, Deserialize)]
pub struct ChampSelectAction {
    #[serde(rename = "championId")]
    pub champion_id: i32,
    #[serde(rename = "actorCellId")]
    pub actor_cell_id: i32,
    #[serde(rename = "type")]
    pub action_type: String,
}

pub enum LCUEvent {
    ChampSelectStarted,
    ChampionPicked(i32),
    ChampSelectEnded,
}
```

### 5. Game Session Manager (`src-tauri/src/game/session.rs`)

```rust
use super::{GameSession, SessionPlayer};
use std::sync::{Arc, Mutex};

pub struct SessionManager {
    current_session: Arc<Mutex<Option<GameSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Build session from Live Client Data + LCU champion select data
    pub async fn build_session(
        &self,
        lcu_data: super::lcu::ChampSelectSession,
    ) -> Result<GameSession, Box<dyn std::error::Error>> {
        let ally_team = lcu_data.my_team.iter().map(|player| {
            SessionPlayer {
                summoner_name: String::new(),  // Filled from additional API call
                puuid: String::new(),
                champion_id: player.champion_id,
                champion_name: self.get_champion_name(player.champion_id),
                team_id: 100,
                position: None,
            }
        }).collect();

        let enemy_team = lcu_data.their_team.iter().map(|player| {
            SessionPlayer {
                summoner_name: String::new(),
                puuid: String::new(),
                champion_id: player.champion_id,
                champion_name: self.get_champion_name(player.champion_id),
                team_id: 200,
                position: None,
            }
        }).collect();

        let session = GameSession {
            ally_team,
            enemy_team,
            local_player: SessionPlayer {
                summoner_name: String::new(),
                puuid: String::new(),
                champion_id: 0,
                champion_name: String::new(),
                team_id: 100,
                position: None,
            },
            game_mode: "CLASSIC".to_string(),
            map_id: 11,
        };

        *self.current_session.lock().unwrap() = Some(session.clone());
        Ok(session)
    }

    pub fn get_current_session(&self) -> Option<GameSession> {
        self.current_session.lock().unwrap().clone()
    }

    pub fn clear_session(&self) {
        *self.current_session.lock().unwrap() = None;
    }

    fn get_champion_name(&self, champion_id: i32) -> String {
        // TODO: Load from Data Dragon static data
        // For now, return ID as string
        format!("Champion_{}", champion_id)
    }
}
```

### 6. Tauri Commands (`src-tauri/src/main.rs`)

Add these commands for frontend/apps to query game state:

```rust
use crate::game::{GamePhase, GameSession};

#[tauri::command]
async fn get_game_phase(
    detector: tauri::State<'_, Arc<GameDetector>>,
) -> Result<GamePhase, String> {
    Ok(detector.get_current_phase())
}

#[tauri::command]
async fn get_game_session(
    session_mgr: tauri::State<'_, Arc<SessionManager>>,
) -> Result<Option<GameSession>, String> {
    Ok(session_mgr.get_current_session())
}

// In main():
fn main() {
    let detector = Arc::new(GameDetector::new());
    let session_mgr = Arc::new(SessionManager::new());

    let detector_clone = detector.clone();

    tauri::Builder::default()
        .manage(detector)
        .manage(session_mgr)
        .invoke_handler(tauri::generate_handler![
            get_game_phase,
            get_game_session,
        ])
        .setup(|app| {
            let app_handle = app.handle();

            // Start game detection loop in background
            tauri::async_runtime::spawn(async move {
                detector_clone.start_detection_loop(app_handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Integration Points

### Integration with Task 02 (Overlay Window)

- When phase changes to `InGame`, Task 02's overlay window is shown
- When phase changes to `NotRunning`, overlay window is hidden

### Integration with Task 07 (App Lifecycle Manager)

- Task 07 reads `showInPhases` from app manifests
- When phase changes, Task 07 shows/hides app widgets accordingly
- Task 07 calls `get_game_session()` to pass to apps

### Integration with Task 08 (HTTP API Server)

- Exposes `GET /api/game/phase` and `GET /api/game/session` endpoints
- Broadcasts `game_phase_changed` events to registered webhooks

### Integration with Task 09 (JavaScript SDK)

- SDK wraps `get_game_phase()` and `get_game_session()` commands
- Provides `LOLOverlay.game.onPhaseChange()` event listener

---

## Testing Requirements

### Unit Tests

Create `src-tauri/src/game/detect.rs` tests:

```rust
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
    }

    #[tokio::test]
    async fn test_detector_initialization() {
        let detector = GameDetector::new();
        assert_eq!(detector.get_current_phase(), GamePhase::NotRunning);
    }
}
```

### Integration Tests

Create `src-tauri/tests/game_detection_test.rs`:

```rust
#[tokio::test]
async fn test_full_phase_lifecycle() {
    let detector = Arc::new(GameDetector::new());

    // Simulate phase transitions
    // Verify events are emitted correctly
    // Check that phase state is maintained
}
```

### Manual Testing Checklist

- [ ] Platform detects when League of Legends.exe launches
- [ ] Platform detects when LeagueClient.exe launches
- [ ] Phase changes to `InGame` when entering a practice tool match
- [ ] Phase changes to `EndGame` when exiting the game
- [ ] `get_game_phase()` command returns correct phase
- [ ] Tauri event `game_phase_changed` is emitted on phase transition
- [ ] Phase detection latency is under 5 seconds
- [ ] No CPU spikes during detection loop
- [ ] LCU lockfile is read correctly
- [ ] Live Client Data API query succeeds during game

---

## Acceptance Criteria

✅ **Complete when:**

1. All process detection logic compiles and runs
2. Game phase detection accurately identifies all 7 phases
3. Phase change events are emitted via Tauri event system
4. LCU connection successfully reads lockfile
5. Live Client Data API queries work during active game
6. `get_game_phase()` and `get_game_session()` commands work from frontend
7. Detection loop runs continuously without crashing
8. All unit tests pass
9. Integration tests pass
10. Manual testing checklist is 100% complete
11. CPU usage stays under 1% during detection loop
12. Phase detection latency is under 5 seconds

---

## Dependencies for Next Tasks

- **Task 05** depends on: Phase events for hotkey activation context
- **Task 06** depends on: Phase events for widget show/hide
- **Task 07** depends on: Phase events and session data
- **Task 08** depends on: Phase events for webhook broadcasting
- **Task 09** depends on: Tauri commands for SDK wrapping

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/game/mod.rs`
- `src-tauri/src/game/detect.rs`
- `src-tauri/src/game/lcu.rs`
- `src-tauri/src/game/session.rs`
- `src-tauri/tests/game_detection_test.rs`

### Modified Files:

- `src-tauri/src/main.rs` (add commands and start detection loop)
- `src-tauri/Cargo.toml` (add dependencies)

---

## Expected Time: 8-10 hours

## Difficulty: High (Process detection + WebSocket + API integration)
