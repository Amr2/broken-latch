# Permissions

Apps declare required permissions in their manifest. Users are shown the permission list before installing.

## Available Permissions

| Permission | Description |
|---|---|
| `game.session` | Read game phase, session, and player data |
| `windows.create` | Create and manage overlay widget windows |
| `hotkeys.register` | Register global keyboard shortcuts |
| `storage` | Read/write persistent key-value storage |
| `notify` | Show toast notifications |
| `messaging` | Send/receive inter-window messages |

## Declaring Permissions

In `manifest.json`:

```json
{
  "permissions": ["game.session", "windows.create", "storage"]
}
```

Only declare permissions your app actually uses. Requesting unnecessary permissions reduces user trust.

## Permission Errors

If your app calls an API without the required permission, the SDK throws:

```
PermissionError: app "my-app" lacks permission "game.session"
```

Always declare all needed permissions before publishing.

## Principle of Least Privilege

Request only what you need:

| App type | Typical permissions |
|---|---|
| Game stats widget | `game.session`, `windows.create`, `storage` |
| Hotkey macro pad | `hotkeys.register`, `windows.create` |
| Coaching tool with backend | `game.session`, `windows.create`, `notify`, `messaging` |
| Minimal HUD | `game.session`, `windows.create` |
