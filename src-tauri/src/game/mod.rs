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
    pub timestamp: i64,
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
    pub position: Option<String>,
}

pub mod detect;
pub mod lcu;
pub mod session;
