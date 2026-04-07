# Storage API

Persistent key-value storage scoped to your app. Data survives app restarts and platform updates.

## Methods

### `get(key)`

Read a stored value.

**Returns:** `Promise<unknown | null>` — `null` if key doesn't exist

```javascript
const settings = await LOLOverlay.storage.get("user-settings");
if (settings) {
  applySettings(settings);
}
```

---

### `set(key, value)`

Write a value. Values must be JSON-serializable.

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.storage.set("user-settings", {
  theme: "dark",
  opacity: 0.8,
  position: { x: 20, y: 100 },
});
```

---

### `delete(key)`

Remove a stored key.

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.storage.delete("user-settings");
```

---

### `clear()`

Delete all storage for your app.

**Returns:** `Promise<void>`

---

## Storage Scope

Each app has its own isolated storage namespace. Apps cannot read each other's data. Keys are prefixed internally as `{appId}:{key}`.

## Size Limits

- Maximum value size: 1 MB per key
- Maximum total storage: 50 MB per app

## Example: Settings Persistence

```javascript
const DEFAULT_SETTINGS = { opacity: 0.9, position: { x: 20, y: 100 } };

async function loadSettings() {
  const saved = await LOLOverlay.storage.get("settings");
  return { ...DEFAULT_SETTINGS, ...(saved ?? {}) };
}

async function saveSettings(settings) {
  await LOLOverlay.storage.set("settings", settings);
}
```
