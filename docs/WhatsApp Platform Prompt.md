# LOLOverlay — Overlay Platform Specification

## What LOLOverlay Is

LOLOverlay is a lightweight, standalone Windows overlay platform built specifically for League of Legends. It is NOT an app — it is the infrastructure that apps run on top of. Think of it the way Overwolf is a platform and Porofessor is an app — LOLOverlay is the platform, and Hunter Mode (or any future app) is the app that integrates with it.

LOLOverlay handles everything at the OS and rendering level so that app developers never have to touch DirectX hooks, Windows API, process injection, or system-level code. An app developer only needs to know HTML, JavaScript, and optionally make HTTP calls.

---

## Core Responsibilities of the Platform

1. Render transparent overlay windows on top of the game
2. Detect League of Legends game lifecycle (launched, champion select, loading, in-game, end-game)
3. Manage app registration, loading, lifecycle, and sandboxing
4. Expose a JavaScript API and a local HTTP API for apps to consume
5. Handle global hotkeys and forward them to the correct app
6. Manage draggable, resizable widget windows per app
7. Handle app distribution, auto-update, and versioning
8. Be the single process users install — apps plug into it

---

## Technical Architecture

### Platform Stack

| Layer | Technology | Purpose |
|---|---|---|
| Core runtime | Tauri 2.0 (Rust) | Lightweight desktop process, no bundled Chromium |
| Overlay window | Windows API via `windows-rs` | Transparent fullscreen always-on-top window |
| DirectX hook | C++ DLL (Detours + DXGI) | Hook Present() to render overlay over game frames |
| App renderer | WebView2 (system Edge) | Render each app's HTML/JS/CSS in isolated webview |
| JS API bridge | Tauri commands + custom JS SDK | Apps call `LOLOverlay.sdk.*` methods |
| HTTP API | Local HTTP server (Axum in Rust) | Apps call `http://localhost:45678/api/*` |
| IPC | Tauri event system | Internal platform event bus |
| App registry | SQLite via tauri-plugin-sql | Installed apps, metadata, permissions |
| Config | TOML files in AppData | Platform config and per-app widget positions |

---

## Platform Process Architecture

```
[LOLOverlay.exe]  ← Single user-installed process
       │
       ├── Core Runtime (Rust/Tauri)
       │     ├── Game Lifecycle Detector
       │     ├── App Registry & Loader
       │     ├── Hotkey Manager
       │     ├── HTTP API Server (port 45678)
       │     └── DirectX Hook Manager
       │
       ├── Overlay Window (transparent fullscreen)
       │     ├── App Widget: Hunter Mode
       │     ├── App Widget: [Future App 2]
       │     └── Platform UI (system tray, app manager)
       │
       └── DLL Injected into LeagueOfLegends.exe
             └── IDXGISwapChain::Present() hook
                   └── Composites WebView2 content over game frame
```

---

## Component 1: Transparent Overlay Window

### Behavior
- One fullscreen transparent window covers the entire screen at all times while LoL is running
- Window is click-through by default — all mouse and keyboard input passes through to the game
- When any app widget is in interactive mode, only that widget's bounding box captures input; the rest remains click-through
- Window z-order is always above the game window (HWND_TOPMOST)
- Window is invisible to screen capture by default (SetWindowDisplayAffinity with WDA_EXCLUDEFROMCAPTURE) — configurable per app

### Implementation (Rust, `src/overlay/window.rs`)

```rust
// Create the overlay window
pub fn create_overlay_window(app: &tauri::AppHandle) -> tauri::Window {
    tauri::WindowBuilder::new(app, "overlay", tauri::WindowUrl::App("overlay.html".into()))
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .fullscreen(true)
        .focused(false)
        .build()
        .unwrap()
}

// Set the window to fully click-through
pub fn set_click_through(hwnd: HWND, enabled: bool) {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if enabled {
            SetWindowLongW(hwnd, GWL_EXSTYLE,
                style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32);
        } else {
            SetWindowLongW(hwnd, GWL_EXSTYLE,
                style & !(WS_EX_TRANSPARENT.0 as i32));
        }
    }
}

// Set a specific region as interactive (only that widget captures input)
pub fn set_interactive_region(hwnd: HWND, regions: Vec<Rect>) {
    // Uses SetWindowRgn with combined HRGN from widget bounding boxes
    // Everything outside these regions remains click-through
}
```

---

## Component 2: DirectX Hook (C++ DLL)

### Purpose
Injects into the League of Legends process and hooks into DirectX 11/12's `IDXGISwapChain::Present()` call. At this hook point — the exact moment each frame is about to be drawn — the platform composites the WebView2 overlay content on top of the game frame.

### Implementation (`overlay-hook/`)

```cpp
// overlay-hook/dx_hook.cpp

typedef HRESULT(__stdcall* Present_t)(IDXGISwapChain*, UINT, UINT);
Present_t original_Present = nullptr;

HRESULT __stdcall hk_Present(IDXGISwapChain* pSwapChain, UINT SyncInterval, UINT Flags) {
    // 1. Get the back buffer render target
    // 2. Set render target to game's back buffer
    // 3. Composite the shared texture from WebView2 onto the back buffer
    // 4. Restore render state
    // 5. Call original Present
    return original_Present(pSwapChain, SyncInterval, Flags);
}

// DLL entry point — hooks Present on attach
BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved) {
    if (reason == DLL_PROCESS_ATTACH) {
        // Use Microsoft Detours to hook IDXGISwapChain::Present
        // Signal ready to LOLOverlay.exe via named pipe
    }
}
```

### DLL Injection Strategy
- LOLOverlay.exe monitors for `LeagueOfLegends.exe` process creation
- On detection, injects `LOLOverlay_hook.dll` using `CreateRemoteThread` + `LoadLibraryA`
- DLL communicates back to LOLOverlay.exe via a named pipe (`\\.\pipe\loloverlay`)
- On game exit, DLL detaches and unhooks cleanly

### Supported DirectX Versions
- DirectX 11 (primary — LoL's current renderer)
- DirectX 12 (secondary — future-proofing)
- Falls back gracefully if hook fails — overlay still works as a transparent topmost window (slightly less smooth compositing)

---

## Component 3: Game Lifecycle Detector

### Purpose
Detect what phase League of Legends is currently in and broadcast phase change events to all running apps.

### Game Phases
```rust
pub enum GamePhase {
    NotRunning,      // LeagueOfLegends.exe not detected
    Launching,       // Process detected, client loading
    InLobby,         // Main client open, no active game
    ChampSelect,     // Champion selection in progress
    Loading,         // Loading screen (game created, not started)
    InGame,          // Active game running
    EndGame,         // Post-game results screen
}
```

### Detection Strategy (`src/game_detect.rs`)
- **Process detection**: Poll for `LeagueOfLegends.exe` every 3 seconds using Windows `EnumProcesses`
- **Phase detection**: Query `https://127.0.0.1:2999/liveclientdata/gamestats` (Riot's local live client endpoint) every 2 seconds when process is running
  - Not reachable = InLobby or ChampSelect (use LCU WebSocket for finer detection)
  - Reachable, `gameTime == 0` = Loading
  - Reachable, `gameTime > 0` = InGame
- **LCU WebSocket**: Connect to League Client WebSocket for ChampSelect events (provides champion picks, summoner names of all players)
- **End game detection**: Monitor for game process exit while InGame = EndGame

### Phase Event Broadcasting
When phase changes, the platform broadcasts to all registered apps:
```json
{
  "event": "game_phase_changed",
  "data": {
    "previous": "Loading",
    "current": "InGame",
    "timestamp": 1712345678
  }
}
```
Delivered via: JS API event listener AND HTTP webhook to registered app endpoints.

---

## Component 4: JavaScript SDK (App-Facing API)

Apps load the platform SDK from a local URL served by LOLOverlay:
```html
<script src="http://localhost:45678/sdk/loloverlay.js"></script>
```

### Full SDK Reference

```javascript
// ─── INITIALIZATION ──────────────────────────────────────────────
LOLOverlay.init({
    appId: "hunter-mode",
    version: "1.0.0"
});

// ─── GAME LIFECYCLE ──────────────────────────────────────────────
// Listen for game phase changes
LOLOverlay.game.onPhaseChange((event) => {
    // event.previous, event.current, event.timestamp
});

// Get current phase
const phase = await LOLOverlay.game.getPhase();
// Returns: "NotRunning" | "InLobby" | "ChampSelect" | "Loading" | "InGame" | "EndGame"

// Get current game session data (available from ChampSelect onwards)
const session = await LOLOverlay.game.getSession();
// Returns: { allyTeam: [...], enemyTeam: [...], localPlayer: {...} }

// ─── WINDOW MANAGEMENT ───────────────────────────────────────────
// Create a draggable widget window
const widget = await LOLOverlay.windows.create({
    id: "hunter-focus-panel",
    url: "panels/hunter_focus.html",
    defaultPosition: { x: 20, y: 200 },
    defaultSize: { width: 320, height: 500 },
    minSize: { width: 200, height: 300 },
    draggable: true,
    resizable: true,
    persistPosition: true,     // Remember position between sessions
    clickThrough: false,       // Widget captures mouse by default
    opacity: 0.9,
    showInPhases: ["InGame"]   // Auto show/hide based on game phase
});

// Show / hide a widget
await LOLOverlay.windows.show("hunter-focus-panel");
await LOLOverlay.windows.hide("hunter-focus-panel");

// Set widget opacity
await LOLOverlay.windows.setOpacity("hunter-focus-panel", 0.75);

// Toggle click-through on a widget
await LOLOverlay.windows.setClickThrough("hunter-focus-panel", true);

// Get widget state
const state = await LOLOverlay.windows.getState("hunter-focus-panel");
// Returns: { visible, position, size, opacity, clickThrough }

// ─── HOTKEYS ─────────────────────────────────────────────────────
// Register a global hotkey
LOLOverlay.hotkeys.register({
    id: "toggle-hunter-focus",
    keys: "Alt+H",
    onPress: () => { /* callback */ }
});

// Unregister
LOLOverlay.hotkeys.unregister("toggle-hunter-focus");

// Check if a hotkey is already taken by another app
const taken = await LOLOverlay.hotkeys.isRegistered("Alt+H");

// ─── STORAGE ─────────────────────────────────────────────────────
// Per-app persistent key-value storage
await LOLOverlay.storage.set("hunter_target_champion_id", 64);
const val = await LOLOverlay.storage.get("hunter_target_champion_id");
await LOLOverlay.storage.delete("hunter_target_champion_id");
await LOLOverlay.storage.clear(); // Clear all storage for this app

// ─── NOTIFICATIONS ───────────────────────────────────────────────
// Show a toast notification in the overlay
LOLOverlay.notify.toast({
    message: "Hunter Mode ready",
    type: "info",           // "info" | "success" | "warning" | "error"
    duration: 3000,
    position: "top-right"
});

// ─── INTER-APP MESSAGING ─────────────────────────────────────────
// Send a message to another app (if installed and running)
LOLOverlay.messaging.send("another-app-id", { type: "ping", data: {} });

// Listen for messages from other apps
LOLOverlay.messaging.on((sender, message) => { });

// ─── PLATFORM INFO ───────────────────────────────────────────────
const info = await LOLOverlay.platform.getInfo();
// Returns: { version, installedApps, currentApp }
```

---

## Component 5: HTTP API (Local REST Server)

The platform runs a local HTTP server on `http://localhost:45678`. Apps can call it from any context — including from a Rust/Python/Node backend process bundled with the app.

### Endpoints

```
GET  /api/platform/info
     → { version, installedApps }

GET  /api/game/phase
     → { phase: "InGame", since: 1712345678 }

GET  /api/game/session
     → { allyTeam: [...], enemyTeam: [...], localPlayer: {...} }

POST /api/windows/show
     Body: { appId, windowId }

POST /api/windows/hide
     Body: { appId, windowId }

POST /api/windows/set-position
     Body: { appId, windowId, x, y }

POST /api/windows/set-opacity
     Body: { appId, windowId, opacity }

POST /api/notify/toast
     Body: { appId, message, type, duration }

GET  /api/storage/:appId/:key
POST /api/storage/:appId/:key  Body: { value }
DEL  /api/storage/:appId/:key

POST /api/webhooks/register
     Body: { appId, event, url }
     → Register a webhook URL to receive platform events

POST /api/webhooks/unregister
     Body: { appId, event }
```

### Webhook Events (pushed to app's registered URL)
```json
{ "event": "game_phase_changed", "data": { "previous": "...", "current": "..." } }
{ "event": "game_session_ready", "data": { "allyTeam": [...], "enemyTeam": [...] } }
{ "event": "app_hotkey_pressed", "data": { "hotkeyId": "toggle-hunter-focus" } }
```

### Authentication
All HTTP API calls must include the app's token in the header:
```
X-LOLOverlay-App-Token: <token_issued_at_registration>
```

---

## Component 6: App Lifecycle Manager

### App Manifest Format
Every app ships with a `manifest.json` at its root:

```json
{
    "id": "hunter-mode",
    "name": "Hunter Mode",
    "version": "1.0.0",
    "description": "League of Legends coaching overlay",
    "author": "ilaka",
    "entryPoint": "index.html",
    "permissions": [
        "game.session",
        "windows.create",
        "hotkeys.register",
        "storage",
        "notify"
    ],
    "windows": [
        {
            "id": "main-panel",
            "url": "index.html",
            "defaultPosition": { "x": 20, "y": 100 },
            "defaultSize": { "width": 340, "height": 600 },
            "showInPhases": ["Loading", "InGame"]
        }
    ],
    "hotkeys": [
        { "id": "toggle-main", "defaultKeys": "Alt+H", "description": "Toggle Hunter Mode" }
    ],
    "webhooks": [
        { "event": "game_phase_changed", "url": "http://localhost:45679/webhook" }
    ],
    "backend": {
        "process": "hunter-mode-backend.exe",
        "port": 45679
    }
}
```

### App Lifecycle States
```
Installed → Enabled → Loading → Running → Suspended → Running
                                                    ↓
                                                 Stopped
```

- **Installed**: App files present in `%AppData%\LOLOverlay\apps\{appId}\`
- **Enabled**: App is toggled on by user
- **Loading**: Platform is initializing the app's webview and backend process
- **Running**: App is active and receiving events
- **Suspended**: Game not running — app paused to save resources
- **Stopped**: App manually stopped or crashed

### App Sandboxing
- Each app runs in its own WebView2 instance
- Apps can only access their own storage namespace
- Apps cannot access other apps' storage without explicit inter-app messaging
- HTTP API calls are authenticated per-app with unique tokens
- Apps declare required permissions in manifest — platform enforces them at runtime
- Apps cannot call Windows APIs directly — all system access goes through the platform SDK

### App Installation Process
1. User downloads an `.lolapp` package (zip archive with manifest.json + app files)
2. Platform validates manifest, checks permissions
3. Files extracted to `%AppData%\LOLOverlay\apps\{appId}\`
4. App registered in SQLite registry with its token
5. App appears in platform UI (system tray → Manage Apps)

---

## Component 7: Hotkey Manager

### Responsibilities
- Register global hotkeys that work even when the game window has focus
- Route hotkey presses to the correct app
- Prevent hotkey conflicts between apps
- Expose hotkey configuration UI per app

### Implementation (`src/hotkey.rs`)
```rust
// Uses Windows RegisterHotKey API for global hotkeys
// Each hotkey is registered system-wide with a unique ID
// WM_HOTKEY messages are handled in a dedicated message loop thread
// On press: look up which app registered that hotkey, emit event to that app

pub struct HotkeyManager {
    registered: HashMap<u32, HotkeyEntry>,  // Windows hotkey ID -> app entry
}

pub struct HotkeyEntry {
    pub app_id: String,
    pub hotkey_id: String,
    pub keys: String,
}

impl HotkeyManager {
    pub fn register(&mut self, app_id: &str, hotkey_id: &str, keys: &str) -> Result<(), HotkeyError>;
    pub fn unregister(&mut self, app_id: &str, hotkey_id: &str) -> Result<(), HotkeyError>;
    pub fn is_taken(&self, keys: &str) -> bool;
    pub fn get_all_for_app(&self, app_id: &str) -> Vec<HotkeyEntry>;
}
```

### Conflict Resolution
- If an app tries to register a hotkey already taken by another app, the platform returns a `HotkeyConflict` error with the name of the conflicting app
- User is shown a conflict dialog in the platform UI to reassign
- Platform hotkeys (reserved): `Alt+Shift+L` = toggle platform UI, `Alt+Shift+Q` = quit platform

---

## Component 8: Widget Window System

### Widget Properties
Every widget window managed by the platform has:

```rust
pub struct WidgetWindow {
    pub id: String,
    pub app_id: String,
    pub url: String,
    pub position: Point,
    pub size: Size,
    pub min_size: Size,
    pub opacity: f32,           // 0.0 to 1.0
    pub visible: bool,
    pub click_through: bool,
    pub draggable: bool,
    pub resizable: bool,
    pub persist_position: bool,
    pub show_in_phases: Vec<GamePhase>,
}
```

### Drag System
- Platform injects a drag handle listener into each app's webview
- Apps declare draggable regions via CSS: `data-loloverlay-drag="true"` on any element
- Platform intercepts mousedown on these regions and moves the window at OS level
- Position saved to config on mouseup if `persistPosition: true`

### Phase-Based Visibility
- Platform automatically shows/hides widgets based on `showInPhases` in manifest
- Apps can override this at runtime via `LOLOverlay.windows.show/hide()`
- Transitions use a 150ms opacity fade (handled by platform, not app)

---

## Component 9: Distribution System

### App Store (Phase 1 — local)
- Platform ships with a built-in app browser accessible from system tray
- Apps listed in a curated JSON registry hosted at `https://apps.loloverlay.gg/registry.json`
- Each entry: appId, name, description, version, downloadUrl, author, screenshots
- One-click install: platform downloads `.lolapp`, validates, installs

### App Store (Phase 2 — full)
- Developer portal at `https://apps.loloverlay.gg`
- Developers submit apps via CLI: `loloverlay publish --package hunter-mode.lolapp`
- Platform team reviews and approves
- Users can rate and review installed apps

### Auto-Update System
- Platform checks for its own updates on startup via `https://releases.loloverlay.gg/latest.json`
- Each installed app's manifest includes an `updateUrl` pointing to latest version metadata
- Platform checks all app update URLs on launch and after game sessions
- Updates download in background and apply on next app restart
- Delta updates supported (only changed files re-downloaded)

### `.lolapp` Package Format
```
hunter-mode-1.0.0.lolapp  (renamed .zip)
├── manifest.json
├── index.html
├── assets/
│   ├── main.js
│   └── styles.css
├── icons/
│   ├── icon-32.png
│   └── icon-256.png
└── backend/               (optional — if app has a backend process)
    └── hunter-mode-backend.exe
```

---

## Component 10: Platform UI

### System Tray
- LOLOverlay runs as a system tray icon (no main window)
- Right-click tray icon shows menu:
  - Manage Apps (open app manager)
  - Settings
  - Enable / Disable All Overlays
  - Quit LOLOverlay

### App Manager Window
- List of installed apps with toggle switches
- Per-app settings button (opens that app's settings panel)
- Hotkey viewer/editor per app
- Install new app (opens app browser or file picker for .lolapp)
- Uninstall app

### Platform Settings
- Startup behavior (launch with Windows)
- Default overlay opacity
- Performance mode (reduces compositing quality to save GPU)
- Excluded games (if platform ever expands beyond LoL)
- Debug mode (shows app console logs)

---

## Performance Requirements

| Metric | Target |
|---|---|
| Platform idle RAM | < 30MB |
| Per-app RAM overhead | < 20MB per app |
| CPU usage (no game) | < 0.5% |
| CPU usage (in-game) | < 2% |
| Frame time added by hook | < 0.3ms per frame |
| Platform installer size | < 8MB |
| Startup time | < 1 second |
| Game phase detection latency | < 3 seconds |

---

## Project Folder Structure

```
loloverlay/
├── src/                          # Platform Rust backend
│   ├── main.rs
│   ├── overlay/
│   │   ├── window.rs             # Transparent window management
│   │   └── region.rs             # Interactive region tracking
│   ├── hook/
│   │   ├── injector.rs           # DLL injection into LoL process
│   │   └── pipe.rs               # Named pipe comm with hook DLL
│   ├── game/
│   │   ├── detect.rs             # Game phase detection
│   │   ├── lcu.rs                # LCU WebSocket client
│   │   └── session.rs            # Game session data model
│   ├── apps/
│   │   ├── registry.rs           # App install/uninstall/list
│   │   ├── loader.rs             # App lifecycle management
│   │   ├── sandbox.rs            # Permission enforcement
│   │   └── updater.rs            # App auto-update logic
│   ├── hotkey.rs                 # Global hotkey manager
│   ├── widgets.rs                # Widget window manager
│   ├── http_api.rs               # Local HTTP server (Axum)
│   ├── sdk_server.rs             # Serves loloverlay.js SDK file
│   ├── db.rs                     # SQLite operations
│   ├── tray.rs                   # System tray UI
│   └── config.rs                 # Platform config (TOML)
├── platform-ui/                  # Tauri frontend (app manager, settings)
│   ├── src/
│   │   ├── AppManager.tsx
│   │   ├── Settings.tsx
│   │   └── AppBrowser.tsx
│   └── package.json
├── sdk/                          # JavaScript SDK (served to apps)
│   └── loloverlay.js             # The SDK file apps load
├── overlay-hook/                 # C++ DirectX hook DLL
│   ├── dllmain.cpp
│   ├── dx_hook.cpp
│   ├── render.cpp
│   └── CMakeLists.txt
├── cli/                          # Developer CLI tool
│   └── src/
│       └── main.rs               # loloverlay publish, loloverlay dev
├── src-tauri/
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

## Developer Experience (For App Builders)

### Local Development Mode
```bash
# Install LOLOverlay CLI
cargo install loloverlay-cli

# Create a new app from template
loloverlay new my-app

# Start dev mode — hot reloads app in the live platform
loloverlay dev

# Package app for distribution
loloverlay build

# Publish to app registry
loloverlay publish
```

### Dev Mode Features
- Hot reload — app UI updates instantly on file save without restarting the platform
- Platform DevTools — inspect WebView2 content for any running app
- Event log — see all platform events in real time
- API log — see all JS SDK and HTTP API calls with timing
- Simulated game phases — trigger phase changes without needing an actual game running (for UI testing)

### App Template Structure (generated by `loloverlay new`)
```
my-app/
├── manifest.json         # App identity and permissions
├── index.html            # App entry point
├── src/
│   ├── main.js           # App logic (uses LOLOverlay SDK)
│   └── styles.css
└── icons/
    ├── icon-32.png
    └── icon-256.png
```

---

## Security Model

- Apps run in sandboxed WebView2 instances — no Node.js access, no filesystem access except through platform APIs
- Each app gets a unique token on installation — all API calls authenticated
- Permissions declared in manifest are the only permissions the app can use — enforced at runtime by the platform, not just on install
- Apps cannot communicate with each other except through the explicit inter-app messaging API, and only if both apps have the `messaging` permission
- The DLL hook only composites visual content — it does not expose any game memory or process data to apps
- Platform verifies `.lolapp` package checksums before installation
- All local HTTP API traffic is localhost-only — not accessible from the network

---

## What LOLOverlay Does NOT Do

- Does NOT read League of Legends game memory
- Does NOT intercept game network packets
- Does NOT provide game data to apps — apps must use official Riot API themselves
- Does NOT modify game files
- Does NOT guarantee compliance with Riot ToS for the apps built on it — each app is responsible for its own ToS compliance
- Does NOT run on macOS or Linux (Windows only, current version)
