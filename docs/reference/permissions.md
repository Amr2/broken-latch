# Permissions Reference

## Full Permission List

| Permission | APIs Unlocked |
|---|---|
| `game.session` | `LOLOverlay.game.*` — phase, session, player data |
| `windows.create` | `LOLOverlay.windows.*` — create, show, hide, position windows |
| `hotkeys.register` | `LOLOverlay.hotkeys.*` — register and unregister global hotkeys |
| `storage` | `LOLOverlay.storage.*` — read/write persistent storage |
| `notify` | `LOLOverlay.notifications.*` — show toast notifications |
| `messaging` | `LOLOverlay.messaging.*` — send/receive inter-window messages |

## Declaring Permissions

```json
{
  "permissions": ["game.session", "windows.create"]
}
```

## What Happens Without a Permission

Calling an API without the required permission throws synchronously:

```
PermissionError: app "my-app" requires permission "storage"
```

## Common Combinations

| Use Case | Permissions |
|---|---|
| Minimal HUD | `game.session`, `windows.create` |
| Stats tracker | `game.session`, `windows.create`, `storage` |
| Hotkey macro | `hotkeys.register`, `windows.create` |
| Full-featured app | all permissions |
