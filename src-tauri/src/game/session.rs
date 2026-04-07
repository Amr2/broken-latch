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

    /// Build session from LCU champion select data
    pub fn build_session(
        &self,
        lcu_data: super::lcu::ChampSelectSession,
    ) -> Result<GameSession, Box<dyn std::error::Error + Send + Sync>> {
        let ally_team = lcu_data
            .my_team
            .iter()
            .map(|player| SessionPlayer {
                summoner_name: String::new(),
                puuid: String::new(),
                champion_id: player.champion_id,
                champion_name: get_champion_name(player.champion_id),
                team_id: 100,
                position: None,
            })
            .collect();

        let enemy_team = lcu_data
            .their_team
            .iter()
            .map(|player| SessionPlayer {
                summoner_name: String::new(),
                puuid: String::new(),
                champion_id: player.champion_id,
                champion_name: get_champion_name(player.champion_id),
                team_id: 200,
                position: None,
            })
            .collect();

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
}

fn get_champion_name(champion_id: i32) -> String {
    // TODO: Load from Data Dragon static data
    format!("Champion_{}", champion_id)
}
