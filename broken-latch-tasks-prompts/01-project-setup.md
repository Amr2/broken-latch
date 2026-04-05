# Task 01: Project Setup & Foundation

**Platform: broken-latch**  
**Dependencies:** None (First task)  
**Estimated Complexity:** Medium  
**Priority:** P0 (Critical Path)

---

## Objective

Initialize the broken-latch platform project with Tauri 2.0, establish the Rust/React foundation, configure build tooling, and set up the SQLite database schema. This creates the skeleton that all future components will integrate into.

---

## Context

broken-latch is a Windows-only overlay platform for League of Legends apps. It's built with Tauri 2.0 (Rust backend, WebView2 frontend) to keep resource usage minimal (<30MB RAM idle). The platform runs as a single process that loads apps as isolated WebView2 instances.

This task establishes:

- Tauri 2.0 project structure
- TypeScript + React frontend for platform UI
- SQLite database for app registry and storage
- Build configuration
- Core project organization

---

## What You Need to Build

### 1. Tauri Project Initialization

Create a new Tauri 2.0 project named `broken-latch`:

```bash
npm create tauri-app@latest
# Project name: broken-latch
# Package manager: npm
# UI template: React + TypeScript
# UI flavor: React
```

**Expected structure:**

```
broken-latch/
├── src/                    # React frontend (platform UI)
├── src-tauri/              # Rust backend
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── tsconfig.json
```

### 2. Rust Project Structure

Inside `src-tauri/src/`, create this module structure:

```
src-tauri/src/
├── main.rs                 # Entry point
├── overlay/
│   ├── mod.rs             # Module export
│   ├── window.rs          # (stub for Task 02)
│   └── region.rs          # (stub for Task 02)
├── hook/
│   ├── mod.rs
│   ├── injector.rs        # (stub for Task 03)
│   └── pipe.rs            # (stub for Task 03)
├── game/
│   ├── mod.rs
│   ├── detect.rs          # (stub for Task 04)
│   ├── lcu.rs             # (stub for Task 04)
│   └── session.rs         # (stub for Task 04)
├── apps/
│   ├── mod.rs
│   ├── registry.rs        # (stub for Task 07)
│   ├── loader.rs          # (stub for Task 07)
│   ├── sandbox.rs         # (stub for Task 07)
│   └── updater.rs         # (stub for Task 11)
├── hotkey.rs              # (stub for Task 05)
├── widgets.rs             # (stub for Task 06)
├── http_api.rs            # (stub for Task 08)
├── sdk_server.rs          # (stub for Task 09)
├── db.rs                  # ✅ Implement in this task
├── tray.rs                # (stub for Task 10)
└── config.rs              # ✅ Implement in this task
```

**In main.rs**, set up module declarations:

```rust
mod overlay;
mod hook;
mod game;
mod apps;
mod hotkey;
mod widgets;
mod http_api;
mod sdk_server;
mod db;
mod tray;
mod config;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. Database Schema (`src-tauri/src/db.rs`)

Implement SQLite database initialization using `tauri-plugin-sql`.

**Add dependency to `Cargo.toml`:**

```toml
[dependencies]
tauri = { version = "2.0", features = ["shell-open"] }
tauri-plugin-sql = { version = "2.0", features = ["sqlite"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

**Database file location:** `%AppData%\broken-latch\db\platform.db`

**Schema (SQL):**

```sql
-- Apps registry
CREATE TABLE IF NOT EXISTS apps (
    id TEXT PRIMARY KEY,              -- e.g. "hunter-mode"
    name TEXT NOT NULL,               -- e.g. "Hunter Mode"
    version TEXT NOT NULL,            -- e.g. "1.0.0"
    description TEXT,
    author TEXT,
    icon_url TEXT,
    manifest_json TEXT NOT NULL,      -- Full manifest as JSON
    install_path TEXT NOT NULL,       -- %AppData%\broken-latch\apps\{id}\
    auth_token TEXT NOT NULL,         -- UUID for API auth
    enabled BOOLEAN DEFAULT 1,
    state TEXT DEFAULT 'installed',   -- installed, enabled, loading, running, suspended, stopped
    installed_at INTEGER NOT NULL,    -- Unix timestamp
    updated_at INTEGER NOT NULL
);

-- App storage (key-value per app)
CREATE TABLE IF NOT EXISTS app_storage (
    app_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,              -- JSON-encoded value
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (app_id, key),
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

-- Hotkey registry
CREATE TABLE IF NOT EXISTS hotkeys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    hotkey_id TEXT NOT NULL,          -- App-defined ID (e.g. "toggle-panel")
    keys TEXT NOT NULL,               -- e.g. "Alt+H"
    win_hotkey_id INTEGER UNIQUE,     -- Windows RegisterHotKey ID
    registered_at INTEGER NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE,
    UNIQUE(app_id, hotkey_id)
);

-- Widget positions (persisted layout)
CREATE TABLE IF NOT EXISTS widget_positions (
    app_id TEXT NOT NULL,
    widget_id TEXT NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    opacity REAL DEFAULT 1.0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (app_id, widget_id),
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

-- Webhook subscriptions
CREATE TABLE IF NOT EXISTS webhooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    event TEXT NOT NULL,              -- e.g. "game_phase_changed"
    url TEXT NOT NULL,                -- e.g. "http://localhost:45679/webhook"
    created_at INTEGER NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

CREATE INDEX idx_apps_enabled ON apps(enabled);
CREATE INDEX idx_webhooks_event ON webhooks(event);
```

**Implement in `db.rs`:**

```rust
use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "initial_schema",
            sql: include_str!("../migrations/001_initial.sql"),
            kind: MigrationKind::Up,
        }
    ]
}

// Helper functions for database operations
pub async fn init_db(app: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database with migrations
    // Returns connection pool
}
```

Create `src-tauri/migrations/001_initial.sql` with the schema above.

### 4. Configuration Management (`src-tauri/src/config.rs`)

Platform configuration stored in `%AppData%\broken-latch\config.toml`.

**Add dependency:**

```toml
[dependencies]
toml = "0.8"
```

**Configuration schema:**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub version: String,
    pub startup: StartupConfig,
    pub overlay: OverlayConfig,
    pub performance: PerformanceConfig,
    pub developer: DeveloperConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub launch_with_windows: bool,
    pub auto_update_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub default_opacity: f32,        // 0.0 to 1.0
    pub screen_capture_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub performance_mode: bool,      // Reduces compositing quality
    pub cpu_usage_limit: f32,        // Percentage (2.0 = 2%)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperConfig {
    pub debug_mode: bool,
    pub show_console_logs: bool,
    pub simulated_game_phases: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            startup: StartupConfig {
                launch_with_windows: false,
                auto_update_check: true,
            },
            overlay: OverlayConfig {
                default_opacity: 1.0,
                screen_capture_visible: false,
            },
            performance: PerformanceConfig {
                performance_mode: false,
                cpu_usage_limit: 2.0,
            },
            developer: DeveloperConfig {
                debug_mode: false,
                show_console_logs: false,
                simulated_game_phases: false,
            },
        }
    }
}

pub fn load_config() -> Result<PlatformConfig, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        let default_config = PlatformConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: PlatformConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &PlatformConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)?;
    std::fs::write(config_path, content)?;
    Ok(())
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = std::env::var("APPDATA")?;
    let config_dir = PathBuf::from(app_data).join("broken-latch");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.toml"))
}
```

### 5. Frontend Platform UI Foundation

Set up React app structure in `src/`:

```
src/
├── components/
│   ├── AppManager/
│   │   └── AppManager.tsx      # (stub for Task 10)
│   ├── Settings/
│   │   └── Settings.tsx        # (stub for Task 10)
│   └── AppBrowser/
│       └── AppBrowser.tsx      # (stub for Task 11)
├── App.tsx
├── main.tsx
└── index.css
```

**Install dependencies:**

```bash
npm install zustand @tanstack/react-query
npm install -D @types/node
```

**Basic App.tsx:**

```tsx
function App() {
  return (
    <div className="app">
      <h1>broken-latch Platform</h1>
      <p>Initialization successful</p>
    </div>
  );
}

export default App;
```

### 6. Tauri Configuration (`tauri.conf.json`)

Configure for Windows-only, system tray mode:

```json
{
  "productName": "broken-latch",
  "version": "0.1.0",
  "identifier": "gg.brokenlatch.platform",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    },
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "sql": {
      "preload": ["sqlite:platform.db"]
    }
  },
  "app": {
    "windows": [
      {
        "title": "broken-latch",
        "width": 1,
        "height": 1,
        "visible": false,
        "skipTaskbar": true
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

---

## Integration Points for Future Tasks

### Where Task 02 (Overlay Window) Will Integrate:

- `src-tauri/src/overlay/window.rs` will create the transparent fullscreen overlay
- Called from `main.rs` on platform startup
- Reads `OverlayConfig` from `config.rs`

### Where Task 04 (Game Detection) Will Integrate:

- `src-tauri/src/game/detect.rs` will poll for game state changes
- Stores current phase in app state
- Emits events to all registered apps (Task 07 will consume this)

### Where Task 07 (App Lifecycle) Will Integrate:

- `apps/registry.rs` reads from the `apps` table
- Uses `auth_token` for API authentication (Task 08)
- Launches app WebView2 instances

### Where Task 08 (HTTP API) Will Integrate:

- Reads from `webhooks` table to push events
- Validates `X-broken-latch-App-Token` header against `apps.auth_token`
- Serves storage API backed by `app_storage` table

---

## Testing Requirements

### Unit Tests

Create `src-tauri/src/db.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_initialization() {
        // Test migrations run successfully
    }

    #[tokio::test]
    async fn test_app_insert_and_query() {
        // Insert test app, query it back
    }

    #[test]
    fn test_config_default() {
        let config = PlatformConfig::default();
        assert_eq!(config.overlay.default_opacity, 1.0);
    }

    #[test]
    fn test_config_save_load() {
        // Save config, load it, verify equality
    }
}
```

### Integration Tests

Create `src-tauri/tests/integration_test.rs`:

```rust
#[tokio::test]
async fn test_database_and_config_integration() {
    // Initialize DB
    // Load config
    // Insert test app with config values
    // Verify integrity
}
```

### Manual Testing Checklist

- [ ] Run `npm run tauri dev` - app launches without errors
- [ ] Verify `%AppData%\broken-latch\` directory is created
- [ ] Verify `platform.db` file exists and contains tables
- [ ] Verify `config.toml` is created with defaults
- [ ] Verify app window is hidden (visible: false)
- [ ] Check Windows Task Manager - process is running

---

## Acceptance Criteria

✅ **Complete when:**

1. Project structure matches the specified layout exactly
2. All stub modules compile without errors
3. Database initializes and all tables are created
4. Config file loads/saves correctly with defaults
5. Frontend React app renders "Initialization successful"
6. `npm run tauri build` produces a working `.msi` installer
7. All unit tests pass
8. Integration test passes
9. Manual testing checklist is 100% complete

---

## Dependencies for Next Tasks

- **Task 02** depends on: `config.rs` for overlay settings
- **Task 04** depends on: Database for storing game session data
- **Task 07** depends on: `apps` table schema
- **Task 08** depends on: `webhooks`, `app_storage` tables
- **Task 10** depends on: Config and DB for platform UI

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/db.rs`
- `src-tauri/src/config.rs`
- `src-tauri/migrations/001_initial.sql`
- All stub module files (`overlay/mod.rs`, `game/mod.rs`, etc.)

### Modified Files:

- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src/App.tsx`
- `package.json`

---

## Expected Time: 4-6 hours

## Difficulty: Medium (Tauri setup + database design)
