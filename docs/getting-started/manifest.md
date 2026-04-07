# App Manifest

Every broken-latch app needs a `manifest.json` in its root directory.

## Full Schema

```json
{
  "id": "my-app",
  "name": "My App",
  "version": "1.0.0",
  "description": "What your app does",
  "author": "Your Name",
  "update_url": "https://example.com/my-app/update.json",
  "entry_point": "index.html",
  "permissions": ["game.session", "windows.create", "hotkeys.register", "storage", "notify", "messaging"],
  "windows": [
    {
      "id": "main",
      "url": "index.html",
      "default_position": { "x": 20, "y": 100 },
      "default_size": { "width": 320, "height": 480 },
      "min_size": { "width": 200, "height": 200 },
      "max_size": { "width": 600, "height": 800 },
      "draggable": true,
      "resizable": true,
      "persist_position": true,
      "click_through": false,
      "opacity": 0.9,
      "show_in_phases": ["InGame"]
    }
  ],
  "hotkeys": [
    {
      "id": "toggle",
      "default_keys": "Ctrl+Shift+O",
      "description": "Toggle main panel"
    }
  ],
  "webhooks": [
    {
      "event": "game.phase_changed",
      "url": "http://localhost:3000/webhook/phase"
    }
  ],
  "backend": {
    "process": "backend.exe",
    "port": 3000
  }
}
```

## Required Fields

| Field | Type | Description |
|---|---|---|
| `id` | `string` | Unique app identifier. Lowercase alphanumeric + hyphens only. |
| `name` | `string` | Display name (max 100 chars) |
| `version` | `string` | Semantic version (e.g. `1.0.0`) |
| `entry_point` | `string` | Main HTML file path |

## Optional Fields

| Field | Type | Description |
|---|---|---|
| `description` | `string` | Short description shown in App Manager |
| `author` | `string` | Author name |
| `update_url` | `string` | URL to check for app updates |
| `permissions` | `string[]` | Required permissions (see [Permissions](/reference/permissions)) |
| `windows` | `Window[]` | Widget window definitions |
| `hotkeys` | `Hotkey[]` | Default hotkey bindings |
| `webhooks` | `Webhook[]` | Backend webhook subscriptions |
| `backend` | `Backend` | Native backend process config |

## Window Config

| Field | Type | Default | Description |
|---|---|---|---|
| `id` | `string` | — | Unique window ID within the app |
| `url` | `string` | — | HTML file to load |
| `default_position` | `{x, y}` | `{0, 0}` | Initial screen position |
| `default_size` | `{width, height}` | — | Initial window size |
| `draggable` | `boolean` | `true` | Allow user to drag the window |
| `resizable` | `boolean` | `false` | Allow user to resize the window |
| `persist_position` | `boolean` | `false` | Save position across sessions |
| `click_through` | `boolean` | `false` | Mouse clicks pass through the window |
| `opacity` | `number` | `1.0` | Window opacity (0.0–1.0) |
| `show_in_phases` | `string[]` | `[]` | Auto-show in these game phases (empty = always) |
