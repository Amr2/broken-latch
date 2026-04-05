# Hunter Mode — GPT Engineering Build Prompt

You are a senior full-stack engineer tasked with building **Hunter Mode**, a lightweight standalone League of Legends coaching overlay application. Below is the complete technical specification. Build exactly what is described.

---

## What You Are Building

A standalone desktop overlay application for Windows that sits transparently on top of League of Legends and shows coaching data panels powered by the official Riot Games API. The app must be extremely lightweight (< 40MB RAM idle), have no dependency on Overwolf, and must never read game memory or interfere with the game process beyond rendering a transparent overlay window.

---

## Technology Stack (Non-Negotiable)

| Layer | Technology | Why |
|---|---|---|
| UI | React 18 + TypeScript + TailwindCSS | Component-based widgets, fast iteration |
| Desktop | Tauri 2.0 (Rust) | Lightweight — no bundled Chromium, ~3MB installer |
| Overlay Window | Windows API via `windows-rs` Rust crate | Transparent always-on-top borderless window |
| DirectX Hook | C++ DLL using Microsoft Detours or goverlay | Hook Present() to render overlay over game |
| IPC | Tauri's built-in event/command system | React <-> Rust communication |
| Local DB | SQLite via `tauri-plugin-sql` | Cache API responses, config, history |
| HTTP Client | `reqwest` (Rust) | All Riot API calls made from Rust backend only |
| Build | Vite + Cargo | Fast dev builds |

---

## Project Structure

```
hunter-mode/
├── src/                        # React frontend
│   ├── components/
│   │   ├── widgets/
│   │   │   ├── LoadingPanel.tsx       # Pre-game enemy profiles
│   │   │   ├── WinConditionPanel.tsx  # Draft analysis
│   │   │   ├── HunterFocus.tsx        # Hunter focus sidebar
│   │   │   └── PostGameReview.tsx     # End-game performance
│   │   ├── shared/
│   │   │   ├── PlayerCard.tsx
│   │   │   ├── StatBar.tsx
│   │   │   ├── ThreatBadge.tsx
│   │   │   └── Timeline.tsx
│   ├── hooks/
│   │   ├── useGameState.ts            # Polls game state from Rust
│   │   ├── useRiotData.ts             # Triggers data fetch via Tauri commands
│   │   └── useHotkeys.ts              # Global hotkey bindings
│   ├── store/
│   │   └── gameStore.ts               # Zustand global state
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── overlay/
│   │   │   ├── window.rs              # Transparent window creation (WinAPI)
│   │   │   └── hotkey.rs              # Global hotkey listener
│   │   ├── riot/
│   │   │   ├── api.rs                 # Riot API client (reqwest)
│   │   │   ├── match_history.rs       # Match v5 endpoints
│   │   │   ├── summoner.rs            # Summoner v4 endpoints
│   │   │   ├── mastery.rs             # Champion Mastery v4
│   │   │   └── league.rs              # League v4 (rank data)
│   │   ├── analysis/
│   │   │   ├── player_profile.rs      # Aggregate player tendencies
│   │   │   ├── win_condition.rs       # Draft composition analysis
│   │   │   └── performance.rs         # Post-game stat comparison
│   │   ├── game_detect.rs             # Detect game phase (lobby, loading, in-game, end)
│   │   ├── db.rs                      # SQLite operations
│   │   └── commands.rs                # Tauri commands exposed to frontend
│   ├── Cargo.toml
│   └── tauri.conf.json
├── overlay-hook/               # C++ DirectX hook DLL
│   ├── dllmain.cpp
│   ├── dx_hook.cpp             # IDXGISwapChain::Present hook
│   ├── render.cpp              # Composite webview content over game
│   └── CMakeLists.txt
└── package.json
```

---

## Step 1: Tauri Window Setup (Transparent Overlay)

In `src-tauri/src/overlay/window.rs`, create the overlay window with these exact properties:

```rust
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::HWND;

pub fn create_overlay_window(app: &tauri::AppHandle) {
    let window = tauri::WindowBuilder::new(
        app,
        "overlay",
        tauri::WindowUrl::App("index.html".into())
    )
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .fullscreen(true)
    .build()
    .unwrap();

    // Make window click-through using WinAPI
    let hwnd = window.hwnd().unwrap();
    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32
        );
    }
}

// Toggle interactive mode (remove WS_EX_TRANSPARENT when user needs to interact)
pub fn set_interactive(hwnd: HWND, interactive: bool) {
    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if interactive {
            SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style & !(WS_EX_TRANSPARENT.0 as i32));
        } else {
            SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_TRANSPARENT.0 as i32);
        }
    }
}
```

---

## Step 2: Game Phase Detection

In `src-tauri/src/game_detect.rs`, poll the Riot Client API to detect game phase. Check every 5 seconds:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    Idle,          // No game running
    ChampSelect,   // In champion selection
    Loading,       // Loading screen
    InGame,        // Active game
    EndGame,       // Post-game screen
}

pub async fn detect_game_phase(api_key: &str, summoner_name: &str) -> GamePhase {
    // 1. Try to reach local client on port 2999 (live client data)
    // 2. If reachable, check /liveclientdata/gamestats for gameTime
    // 3. If gameTime == 0 and roster present = Loading
    // 4. If gameTime > 0 = InGame
    // 5. If not reachable, check recent match via Match v5 for EndGame
    // 6. Otherwise Idle
}
```

Emit game phase changes to frontend via Tauri events:
```rust
app.emit_all("game-phase-changed", &phase).unwrap();
```

---

## Step 3: Riot API Client

In `src-tauri/src/riot/api.rs`:

```rust
use reqwest::Client;

pub struct RiotApiClient {
    client: Client,
    api_key: String,
    region: String, // "na1", "euw1", etc.
    regional_route: String, // "americas", "europe", etc.
}

impl RiotApiClient {
    // Match v5 — get last N matches for a puuid
    pub async fn get_match_ids(&self, puuid: &str, count: u32) -> Result<Vec<String>>;
    
    // Match v5 — get full match data
    pub async fn get_match(&self, match_id: &str) -> Result<MatchDto>;
    
    // Summoner v4 — lookup by name
    pub async fn get_summoner_by_name(&self, name: &str) -> Result<SummonerDto>;
    
    // Champion Mastery v4 — get top masteries
    pub async fn get_top_masteries(&self, summoner_id: &str, count: u32) -> Result<Vec<ChampionMasteryDto>>;
    
    // League v4 — get rank data
    pub async fn get_league_entries(&self, summoner_id: &str) -> Result<Vec<LeagueEntryDto>>;
}
```

Rate limiting: implement a token bucket rate limiter respecting Riot's limits (20 requests/1s, 100 requests/2min per key). Cache all responses in SQLite with a 10-minute TTL.

---

## Step 4: Player Profile Analysis

In `src-tauri/src/analysis/player_profile.rs`, given a list of MatchDto objects for a player on a specific champion, compute:

```rust
pub struct PlayerProfile {
    pub summoner_name: String,
    pub rank: String,                       // e.g. "Gold II"
    pub champion_id: u32,
    pub mastery_level: u32,
    pub mastery_points: u64,
    pub win_rate: f32,                      // 0.0 to 1.0
    pub avg_kda: f32,
    pub avg_cs_per_min: f32,
    pub avg_vision_score: f32,
    pub avg_game_duration_mins: f32,
    pub most_common_build: Vec<u32>,        // Item IDs in order
    pub most_common_summoner_spells: (u32, u32), // Spell IDs
    pub playstyle_tag: PlaystyleTag,        // Aggressive / Farm-focused / Roam-heavy
    pub recent_form: RecentForm,            // WinStreak(n) / LossStreak(n) / Mixed
    pub skill_max_order: [u8; 3],           // [first, second, third] — 1=Q,2=W,3=E
    pub roam_tendency: RoamTendency,        // StaysLane / EarlyRoamer / LateRoamer
    pub threat_level: u8,                   // 1-5
}

pub enum PlaystyleTag {
    AggressiveEarly,
    FarmFocused,
    RoamHeavy,
    ObjectivePriority,
    Balanced,
}

pub fn build_player_profile(
    matches: &[MatchDto],
    summoner_puuid: &str,
    champion_id: u32,
    rank: &str,
    mastery: &ChampionMasteryDto,
) -> PlayerProfile { ... }
```

---

## Step 5: Win Condition Analysis

In `src-tauri/src/analysis/win_condition.rs`, given two team compositions (5 champion IDs each) and a region, compute:

```rust
pub struct WinConditionReport {
    pub ally_win_conditions: Vec<String>,       // Short coaching strings
    pub enemy_win_conditions: Vec<String>,
    pub ally_power_timeline: PowerTimeline,
    pub enemy_power_timeline: PowerTimeline,
    pub favorable_rating: FavorableRating,      // Favorable / Even / Unfavorable
    pub key_threats: Vec<u32>,                  // Top 2 enemy champion IDs to watch
    pub draft_tips: Vec<String>,                // 1-2 coaching tips
}

pub struct PowerTimeline {
    pub early_game: PowerLevel,   // Minutes 0-14
    pub mid_game: PowerLevel,     // Minutes 15-25
    pub late_game: PowerLevel,    // Minutes 25+
}

pub enum PowerLevel { Low, Medium, High }

// Derive win conditions from champion tags (Tank, Fighter, Mage, etc.)
// Use Riot Data Dragon champion data for tags
pub fn analyze_compositions(
    ally_champions: &[u32],
    enemy_champions: &[u32],
    dragon_data: &DragonChampionData,
) -> WinConditionReport { ... }
```

---

## Step 6: Tauri Commands (Frontend <-> Backend Bridge)

In `src-tauri/src/commands.rs`, expose these commands to React:

```rust
#[tauri::command]
pub async fn fetch_enemy_profiles(
    summoner_names: Vec<String>,
    champion_ids: Vec<u32>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<PlayerProfile>, String>

#[tauri::command]
pub async fn fetch_win_condition(
    ally_champions: Vec<u32>,
    enemy_champions: Vec<u32>,
    state: tauri::State<'_, AppState>,
) -> Result<WinConditionReport, String>

#[tauri::command]
pub async fn fetch_hunter_target(
    summoner_name: String,
    champion_id: u32,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerProfile, String>

#[tauri::command]
pub async fn fetch_post_game_review(
    match_id: String,
    player_puuid: String,
    state: tauri::State<'_, AppState>,
) -> Result<PostGameReview, String>

#[tauri::command]
pub async fn set_interactive_mode(
    interactive: bool,
    window: tauri::Window,
) -> Result<(), String>

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String>

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String>
```

---

## Step 7: React Frontend — Widget System

Each widget is a self-contained React component that:
- Listens to Tauri events for game phase changes
- Calls Tauri commands to fetch its data
- Renders conditionally based on game phase
- Is draggable (use `react-draggable`)
- Saves its position to localStorage

### LoadingPanel.tsx
```tsx
// Shows when game phase = Loading
// Displays 5 enemy player cards in a compact table
// Each card: champion icon, player name, rank badge, win rate, KDA
// Expandable on click to show full profile
// Auto-hides 10 seconds after game phase changes to InGame
```

### WinConditionPanel.tsx
```tsx
// Shows when game phase = Loading or InGame
// Collapsible — collapsed by default in-game
// Shows ally vs enemy win conditions, power timeline bars, key threats
// Toggle with Alt+W hotkey
```

### HunterFocus.tsx
```tsx
// Shows when game phase = InGame and a target is locked
// Toggle with Alt+H hotkey
// Target selection: click on enemy champion name in overlay
// Shows: target profile card, ability tendencies, positional tendencies, threat level, counter tips
// Switching targets re-fetches and re-renders instantly
```

### PostGameReview.tsx
```tsx
// Shows when game phase = EndGame
// Full-screen card with stat comparison
// Dismissible with Escape key or click
// Shows: your stats vs personal average, vs rank average, strength/weakness flags
// Hunter target outcome: did you counter them successfully?
```

---

## Step 8: Global State (Zustand)

```ts
interface GameStore {
    gamePhase: GamePhase;
    allyChampions: number[];
    enemyChampions: number[];
    enemyProfiles: PlayerProfile[];
    winCondition: WinConditionReport | null;
    hunterTarget: PlayerProfile | null;
    hunterTargetChampionId: number | null;
    postGameReview: PostGameReview | null;
    isInteractiveMode: boolean;
    settings: AppSettings;
    
    setGamePhase: (phase: GamePhase) => void;
    setEnemyProfiles: (profiles: PlayerProfile[]) => void;
    lockHunterTarget: (championId: number) => void;
    toggleInteractiveMode: () => void;
}
```

---

## Step 9: Visual Design Requirements

- **Color palette**: Dark theme by default — semi-transparent dark panels (rgba(10, 10, 15, 0.85)) over the game
- **Panel border**: 1px solid rgba(255, 255, 255, 0.1) with subtle backdrop-filter: blur(8px)
- **Typography**: Clean sans-serif, 12px for data, 14px for labels, 16px for headings
- **Threat colors**: Green (#4CAF50) = low threat, Amber (#FF9800) = medium, Red (#F44336) = high
- **Win condition colors**: Blue = favorable, Gray = even, Orange = unfavorable
- **Animations**: Panels fade in (opacity 0 → 1, 200ms ease). No bouncing or heavy animations — respects game performance
- **Icons**: Use champion icons from Riot Data Dragon CDN. Rank icons sourced from Data Dragon assets.
- **Compact by design**: Every panel must be readable without taking more than 25% of screen width or 30% of screen height

---

## Step 10: Settings Schema

```ts
interface AppSettings {
    apiKey: string;
    region: string;               // "na1" | "euw1" | "kr" | etc.
    hotkeys: {
        toggleInteractive: string;    // default: "Alt+H"
        toggleWinCondition: string;   // default: "Alt+W"
        dismissPostGame: string;      // default: "Escape"
    };
    widgets: {
        loadingPanel: { enabled: boolean; position: {x: number, y: number}; opacity: number };
        winCondition: { enabled: boolean; position: {x: number, y: number}; opacity: number };
        hunterFocus: { enabled: boolean; position: {x: number, y: number}; opacity: number };
        postGame: { enabled: boolean; opacity: number };
    };
    features: {
        hunterFocusUnlocked: boolean;  // false = free tier, true = premium
        extendedHistory: boolean;       // false = 10 matches, true = 50 matches
    };
    colorblindMode: boolean;
    language: string;              // "en" default
}
```

---

## What NOT to Do (Hard Constraints)

1. Never read game memory — no ReadProcessMemory calls, no Cheat Engine-style scanning
2. Never intercept game network traffic
3. Never inject code into the League of Legends game process itself — the DLL hook is for the rendering layer only (DirectX Present), not for game logic access
4. Never store user data on external servers without explicit consent
5. Never compute real-time combat stats (damage per second, lethal thresholds) — all insights are historical, not live
6. Never exceed Riot API rate limits — implement proper throttling and caching
7. Never hardcode the API key in source code

---

## Deliverables Expected

1. Full working Tauri 2.0 project with all Rust modules implemented
2. All React components implemented with TypeScript
3. SQLite schema and migration files
4. C++ DLL project for DirectX hook (with CMakeLists.txt)
5. Tauri config (tauri.conf.json) properly configured for transparent overlay
6. README with setup instructions, build steps, and API key configuration
7. Settings UI screen for first-time setup (API key, region, hotkey config)
