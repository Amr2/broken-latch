# Manifest Schema

Full JSON Schema for `manifest.json`.

## Root Object

```typescript
interface AppManifest {
  id: string;                    // required: lowercase, alphanumeric, hyphens
  name: string;                  // required: max 100 chars
  version: string;               // required: semver e.g. "1.0.0"
  description?: string;
  author?: string;
  update_url?: string;           // URL returning AppUpdateInfo JSON
  entry_point: string;           // required: main HTML file
  permissions?: Permission[];
  windows?: WindowManifest[];
  hotkeys?: HotkeyManifest[];
  webhooks?: WebhookManifest[];
  backend?: BackendManifest;
}
```

## Permission Values

```typescript
type Permission =
  | "game.session"
  | "windows.create"
  | "hotkeys.register"
  | "storage"
  | "notify"
  | "messaging";
```

## WindowManifest

```typescript
interface WindowManifest {
  id: string;
  url: string;
  default_position: { x: number; y: number };
  default_size: { width: number; height: number };
  min_size?: { width: number; height: number };
  max_size?: { width: number; height: number };
  draggable?: boolean;        // default: true
  resizable?: boolean;        // default: false
  persist_position?: boolean; // default: false
  click_through?: boolean;    // default: false
  opacity?: number;           // default: 1.0, range: 0.0–1.0
  show_in_phases?: GamePhase[];
}
```

## HotkeyManifest

```typescript
interface HotkeyManifest {
  id: string;
  default_keys: string;   // e.g. "Ctrl+Shift+O"
  description: string;
}
```

## WebhookManifest

```typescript
interface WebhookManifest {
  event: string;   // e.g. "game.phase_changed"
  url: string;     // must be localhost
}
```

## BackendManifest

```typescript
interface BackendManifest {
  process: string;  // path relative to app directory
  port: number;
}
```

## Webhook Event Names

| Event | Description |
|---|---|
| `game.phase_changed` | Game phase transition |
| `game.session_updated` | Session data changed |
