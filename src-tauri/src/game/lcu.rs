use serde::Deserialize;

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

    /// Find LCU credentials from lockfile and connect
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    /// Quick check if we're currently in champion select (uses HTTP, not WebSocket)
    pub async fn check_champ_select(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let (port, token) = match self.read_lockfile_credentials() {
            Ok(creds) => creds,
            Err(_) => return Ok(false),
        };

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let url = format!("https://127.0.0.1:{}/lol-champ-select/v1/session", port);

        let response = client
            .get(&url)
            .basic_auth("riot", Some(&token))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get champion select session data
    pub async fn get_champ_select_session(
        &self,
    ) -> Result<ChampSelectSession, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let url = format!(
            "https://127.0.0.1:{}/lol-champ-select/v1/session",
            self.port.ok_or("LCU not connected")?
        );

        let response = client
            .get(&url)
            .basic_auth(
                "riot",
                Some(self.auth_token.as_ref().ok_or("No auth token")?),
            )
            .send()
            .await?;

        let session: ChampSelectSession = response.json().await?;
        Ok(session)
    }

    fn get_lockfile_path(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        // Try standard Riot Games install path first
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let path = std::path::PathBuf::from(&local_app_data)
                .join("Riot Games")
                .join("League of Legends")
                .join("lockfile");
            if path.exists() {
                return Ok(path);
            }
        }

        // Try program files install location
        let paths = [
            r"C:\Riot Games\League of Legends\lockfile",
            r"D:\Riot Games\League of Legends\lockfile",
        ];
        for p in &paths {
            let path = std::path::PathBuf::from(p);
            if path.exists() {
                return Ok(path);
            }
        }

        Err("League of Legends lockfile not found".into())
    }

    /// Read lockfile and return (port, token) without persisting state
    fn read_lockfile_credentials(&self) -> Result<(u16, String), Box<dyn std::error::Error + Send + Sync>> {
        let lockfile_path = self.get_lockfile_path()?;
        let content = std::fs::read_to_string(lockfile_path)?;
        let parts: Vec<&str> = content.split(':').collect();
        if parts.len() < 5 {
            return Err("Invalid lockfile format".into());
        }
        let port: u16 = parts[2].parse()?;
        let token = parts[3].to_string();
        Ok((port, token))
    }

    /// Subscribe to LCU WebSocket events (placeholder for full implementation)
    pub async fn subscribe_events(
        &self,
        _event_handler: impl Fn(LCUEvent) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Full WebSocket subscription using tokio-tungstenite
        // For now, the detection loop uses HTTP polling via check_champ_select()
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
