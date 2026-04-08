# Messaging API

Send messages between windows within your app, or subscribe to platform-wide events.

## Methods

### `send(target, event, data?)`

Send a message to another window or to your backend.

**Parameters:**
- `target: string` — window ID or `"backend"`
- `event: string` — event name
- `data?: unknown` — JSON-serializable payload

**Returns:** `Promise<void>`

```javascript
// Send to another window in your app
await LOLOverlay.messaging.send("settings-panel", "config-updated", {
  opacity: 0.8,
});

// Send to backend
await LOLOverlay.messaging.send("backend", "game-started", {
  mode: "CLASSIC",
});
```

---

### `on(event, callback)`

Listen for messages sent to the current window.

**Parameters:**
- `event: string`
- `callback: (data: unknown) => void`

**Returns:** `void`

```javascript
LOLOverlay.messaging.on("config-updated", (data) => {
  applyConfig(data);
});
```

---

### `off(event, callback?)`

Remove a message listener. Omit `callback` to remove all listeners for `event`.

**Returns:** `void`

---

## Platform Events

Subscribe to built-in platform events using the `platform:` prefix:

| Event | Payload | Description |
|---|---|---|
| `platform:update_available` | `{ version: string }` | Platform update found |
| `platform:overlays_toggle` | `{}` | User toggled all overlays |

```javascript
LOLOverlay.messaging.on("platform:update_available", ({ version }) => {
  LOLOverlay.notifications.show(`Platform update v${version} available`, {
    type: "info",
    duration: 8000,
  });
});
```
