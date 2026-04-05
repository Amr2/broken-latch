# Hunter Mode — Master Product Prompt

## Product Vision
Hunter Mode is a lightweight, standalone League of Legends coaching overlay built without Overwolf dependency. It sits transparently on top of the game screen and delivers real-time pre-game intelligence, draft analysis, win condition coaching, and enemy tendency insights — all powered exclusively by the official Riot Games API (Match v5, Summoner v4, Champion Mastery v4, League v4). The product is designed to feel invisible until needed, with minimal RAM footprint and zero input interference.

---

## Technical Architecture

### Platform Stack
- **UI Layer**: React + TailwindCSS (widget-based, modular panels)
- **Desktop Framework**: Tauri 2.0 (Rust backend — lightweight, no bundled Chromium)
- **Overlay Rendering**: Transparent always-on-top borderless window via Windows API (WS_EX_LAYERED + WS_EX_TRANSPARENT flags)
- **DirectX Hook**: C++ DLL injected into the game process, hooking IDXGISwapChain::Present() to composite the Tauri webview content over the game frame
- **IPC Bridge**: Tauri's built-in event system for communication between React UI and Rust backend
- **API Layer**: Riot Games official REST API called from Rust backend (never from frontend)
- **Data Cache**: Local SQLite via Tauri's sql plugin — caches match history, champion data, and player profiles to minimize API calls

### Performance Targets
- Idle RAM usage: < 40MB
- Installer size: < 10MB
- Startup time: < 500ms
- Zero FPS impact during gameplay (overlay renders on a separate thread)
- No anti-cheat conflicts (no memory reading, no game process modification beyond DLL hook for rendering only)

### Window Behavior
- Transparent overlay window — click-through by default (mouse events pass through to game)
- Activates to interactive mode via configurable hotkey (e.g. Alt+H)
- Snaps back to click-through mode automatically when focus returns to game
- All widgets are draggable and resizable — layout persists between sessions via local config file
- Supports multiple monitor setups

---

## Core Features

### Feature 1: Pre-Game Loading Screen Panel
**Trigger**: Detected when game client enters champion loading screen state (polled via Riot API match detection)

**What it shows**:
- Each enemy player's summoner name, rank (solo/flex), and win rate on their selected champion
- Their last 10 games on that champion: KDA, CS/min, vision score, average game duration
- Their most common build path on that champion (top 3 item combinations from match history)
- Their most played summoner spells on that champion
- Historical matchup data: how that champion has performed against your champion in recent patches (win rate pulled from aggregated match data)
- Their overall recent form: win streak / loss streak, average KDA across all champions in last 20 games

**Data Sources**: Match v5 (match history), Summoner v4 (player lookup), Champion Mastery v4 (mastery level, points), League v4 (rank, LP)

**UI**: Compact 5-row table (one row per enemy), expandable on hover to show full stats. Positioned top-center of screen. Fades out 10 seconds after game starts.

---

### Feature 2: Win Condition Advisor
**Trigger**: End of champion select / start of loading screen

**What it shows**:
- **Enemy team win conditions**: Based on draft composition analysis (e.g. "Enemy team has 4 engage champions + a hypercarry — their win condition is to force teamfights after 25 minutes")
- **Your team win conditions**: Mirrored analysis (e.g. "Your team has strong poke + siege — win condition is to take objectives through poke without committing to fights")
- **Power spike timeline**: A simple timeline showing at what game minutes each team becomes stronger (early game, mid game, late game power ratings as Low / Medium / High)
- **Key threats**: Top 2 enemy champions flagged as highest priority threats based on their mastery, rank, and champion strength in current meta
- **Draft tips**: 1-2 short coaching lines (e.g. "Avoid fighting in river — 3 enemy champions have AoE engage that dominates narrow spaces")

**Data Sources**: Match v5 (composition outcomes), Champion data from Riot Data Dragon (champion tags, stats), aggregated win rate data by composition type

**UI**: Collapsible side panel, right edge of screen. Readable in 5 seconds. Uses color coding: green = favorable, amber = even, red = unfavorable. Persists throughout the game, accessible via hotkey.

---

### Feature 3: Hunter Focus (Signature Feature)
**Trigger**: Player manually locks focus onto a specific enemy champion via hotkey or overlay click during the game

**Core Concept**: The player designates one enemy as their "Hunt Target." Hunter Mode then aggregates everything known about that specific player and champion from historical data and surfaces it as a live coaching sidebar.

**What it shows when locked onto a target**:

#### Target Profile Card
- Champion name, player rank, mastery level
- Their win rate on this champion this season
- Their most common playstyle tag (e.g. "Aggressive early", "Farm-focused", "Roam-heavy") — derived from average early game stats in match history

#### Ability Tendency Panel
- Which abilities they max first (derived from match history item/time patterns)
- Their average skill order (Q first, E second, etc.)
- Known combo patterns for that champion (sourced from static champion data + community data)
- Their average time before using Ultimate in fights (early vs late fight tendency)

#### Positional Tendency Panel
- Their most common lane position at 5 min, 10 min, 15 min (derived from match history ward patterns and objective interaction data)
- Whether they tend to roam early or stay in lane
- Their most common objective priority (Dragon vs Baron vs towers)

#### Threat Level Indicator
- A simple 1–5 threat rating based on: current rank, champion mastery, recent form, champion tier in current meta
- Color coded: green (manageable), amber (respect), red (high priority)

#### How to Win This Matchup
- 2–3 coaching tips specific to countering that champion, sourced from static champion counter data
- Key item to build against them (based on their most common build path)

**Important ToS note**: All data in Hunter Focus is derived entirely from historical match records via the official Riot API. No live in-game memory reading, no real-time combat calculations, no lethality detection. This is a preparation and awareness tool — equivalent to a coach showing you a scouting report before a match.

**UI**: Compact sidebar panel, left edge of screen. Toggle on/off with hotkey. Switching targets is instant — press hotkey and click a different enemy nameplate (detected via overlay click coordinates mapped to champion positions in loading screen data).

---

### Feature 4: Post-Game Performance Review
**Trigger**: Detected when match ends (Riot API match completion)

**What it shows**:
- Your performance vs your historical average on that champion: KDA delta, CS delta, vision score delta, damage delta
- Your performance vs average for your rank on that champion (benchmarked against aggregated data)
- One key strength this game (highest positive delta stat)
- One key improvement area (highest negative delta stat)
- Whether your pre-game win condition prediction was accurate (retrospective comparison)
- Your Hunter Focus target's actual performance vs their historical average (did you successfully shut them down?)

**UI**: Full-screen card overlay displayed on end-game screen. Dismissible. Option to save review to local history log.

---

### Feature 5: Settings & Customization
- Hotkey configuration for all features
- Widget position lock / unlock
- Toggle individual features on/off
- Opacity control per widget (10%–100%)
- Language support (English first, expandable)
- API key management (user inputs their own Riot API key or app uses registered production key)
- Data refresh rate control (to manage API rate limits)
- Colorblind mode (replaces green/amber/red with accessible palette)

---

## Monetization Model (Freemium)
**Free tier**:
- Pre-game loading screen panel (5 enemy profiles)
- Win condition advisor (basic version)
- Post-game performance review (last 5 games history)

**Premium tier** (Hunter Pass):
- Full Hunter Focus feature (all panels unlocked)
- Extended match history depth (last 50 games vs 10)
- Advanced win condition advisor (full detail mode)
- Unlimited post-game history
- Priority API refresh rate
- Custom widget themes

---

## Data Privacy & Compliance
- All API calls use the player's own account data or publicly available match data via official Riot API
- No data is sold or shared with third parties
- Player data is cached locally only — no external database storing user data
- Complies fully with Riot Games API Terms of Use and General Policies
- Product registered and approved through Riot Developer Portal before public release
- Free tier always available as required by Riot's monetization policies

---

## Distribution
- Direct installer download from official product website
- No Overwolf dependency — fully standalone
- Auto-update system via Tauri's built-in updater (delta updates)
- Windows 10/11 only (initial release)
- Installer size target: < 10MB
- Requires: Windows WebView2 runtime (pre-installed on Windows 11, auto-downloaded on Windows 10)
