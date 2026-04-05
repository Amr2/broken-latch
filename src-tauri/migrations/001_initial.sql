-- Apps registry
CREATE TABLE IF NOT EXISTS apps (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT,
    icon_url TEXT,
    manifest_json TEXT NOT NULL,
    install_path TEXT NOT NULL,
    auth_token TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    state TEXT DEFAULT 'installed',
    installed_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- App storage (key-value per app)
CREATE TABLE IF NOT EXISTS app_storage (
    app_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (app_id, key),
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE
);

-- Hotkey registry
CREATE TABLE IF NOT EXISTS hotkeys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    hotkey_id TEXT NOT NULL,
    keys TEXT NOT NULL,
    win_hotkey_id INTEGER UNIQUE,
    registered_at INTEGER NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE,
    UNIQUE (app_id, hotkey_id)
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
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE
);

-- Webhook subscriptions
CREATE TABLE IF NOT EXISTS webhooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    event TEXT NOT NULL,
    url TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE
);

CREATE INDEX idx_apps_enabled ON apps (enabled);

CREATE INDEX idx_webhooks_event ON webhooks(event);