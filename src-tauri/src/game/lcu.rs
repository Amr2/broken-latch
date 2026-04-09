//! LCU (League Client Update) API client.
//!
//! The League client exposes a local REST API over HTTPS.  Its port and
//! credentials are written to a *lockfile* on disk the moment the client
//! starts.  We read that file, build an authenticated reqwest client, and
//! poll the `/lol-gameflow/v1/gameflow-phase` endpoint to determine which
//! phase the player is in.
//!
//! # Lockfile format
//! ```text
//! LeagueClient:<pid>:<port>:<password>:<protocol>
//! ```
//! The client uses a self-signed certificate, so TLS verification is disabled.
//!
//! # Gameflow phase → broken-latch phase mapping
//! | LCU gameflow phase                              | broken-latch phase |
//! |-------------------------------------------------|--------------------|
//! | None / Lobby / Matchmaking / ReadyCheck         | IN_LOBBY           |
//! | ChampSelect                                     | CHAMP_SELECT       |
//! | GameStart / InProgress (no live-data yet)       | LOADING            |
//! | InProgress (live-data 200 OK)                   | IN_GAME            |
//! | WaitingForStats / PreEndOfGame / EndOfGame      | END_GAME           |

use std::path::{Path, PathBuf};
use reqwest::Client;

/// Authenticated HTTP client for one running League client instance.
pub struct LcuClient {
    client:   Client,
    base_url: String,
    password: String,
}

impl LcuClient {
    /// Construct from a lockfile path.  Returns `None` if the file cannot be
    /// read or does not have the expected format.
    pub fn from_lockfile(path: &Path) -> Option<Self> {
        let content  = std::fs::read_to_string(path).ok()?;
        let parts: Vec<&str> = content.trim().split(':').collect();
        if parts.len() < 5 { return None; }

        let port     = parts[2];
        let password = parts[3].to_string();
        let protocol = parts[4]; // "https"

        let base_url = format!("{}://127.0.0.1:{}", protocol, port);

        // The LCU uses a self-signed certificate — we must skip verification.
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .ok()?;

        Some(Self { client, base_url, password })
    }

    /// Query `/lol-gameflow/v1/gameflow-phase`.
    ///
    /// Returns the raw phase string (e.g. `"ChampSelect"`) or `None` if the
    /// request fails (client not running, network error, etc.).
    pub async fn gameflow_phase(&self) -> Option<String> {
        let url  = format!("{}/lol-gameflow/v1/gameflow-phase", self.base_url);
        let resp = self.client.get(&url)
            .basic_auth("riot", Some(&self.password))
            .send().await.ok()?;

        if !resp.status().is_success() { return None; }

        // The response body is a JSON string: `"ChampSelect"`
        let text = resp.text().await.ok()?;
        Some(text.trim().trim_matches('"').to_string())
    }

    /// Check whether the live-game data endpoint is reachable.
    ///
    /// `https://127.0.0.1:2999/liveclientdata/gamestats` returns 200 only
    /// while an actual game is in progress.  A 404 or connection-refused
    /// means the game is still loading.
    pub async fn is_live_game_active(client: &Client) -> bool {
        client
            .get("https://127.0.0.1:2999/liveclientdata/gamestats")
            .send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub fn http_client(&self) -> &Client {
        &self.client
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Try the two most common lockfile locations on Windows.
///
/// Callers that have the LeagueClient process path should use that instead
/// (see `game::detect`).
pub fn find_lockfile_default() -> Option<PathBuf> {
    let candidates = [
        r"C:\Riot Games\League of Legends\lockfile",
        r"C:\Program Files\Riot Games\League of Legends\lockfile",
    ];
    candidates.iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// Map a raw LCU gameflow-phase string to a broken-latch phase string.
pub fn lcu_phase_to_platform(lcu: &str, live_game_active: bool) -> &'static str {
    match lcu {
        "None" | "Lobby" | "Matchmaking" |
        "CheckedIntoTournament" | "ReadyCheck" => "IN_LOBBY",

        "ChampSelect" => "CHAMP_SELECT",

        // GameStart = loading screen; InProgress = could still be loading
        "GameStart"   => "LOADING",
        "InProgress"  => if live_game_active { "IN_GAME" } else { "LOADING" },

        "WaitingForStats" | "PreEndOfGame" | "EndOfGame" => "END_GAME",

        // Unknown / transitional
        _ => "IN_LOBBY",
    }
}
