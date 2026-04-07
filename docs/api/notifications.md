# Notifications API

Show non-intrusive toast notifications to the user.

## Methods

### `show(message, options?)`

Display a toast notification.

**Parameters:**
- `message: string` — notification text
- `options?: NotificationOptions`

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.notifications.show("Enemy jungler spotted!", {
  duration: 3000,
  type: "warning",
});
```

---

## `NotificationOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `duration` | `number` | `3000` | Display time in milliseconds |
| `type` | `"info" \| "success" \| "warning" \| "error"` | `"info"` | Notification style |
| `title` | `string` | — | Optional bold title above message |

## Examples

```javascript
// Simple info
await LOLOverlay.notifications.show("Game phase changed to InGame");

// Warning with title
await LOLOverlay.notifications.show("Baron spawning in 30 seconds", {
  title: "Objective Alert",
  type: "warning",
  duration: 5000,
});

// Success after save
await LOLOverlay.storage.set("config", newConfig);
await LOLOverlay.notifications.show("Settings saved", { type: "success", duration: 1500 });
```

## Best Practices

- Keep messages short (under 80 characters)
- Use `warning`/`error` types sparingly — they're more visually disruptive
- Don't show notifications during champion select loading (players are focused)
