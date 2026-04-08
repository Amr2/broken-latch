# Hotkeys API

Register global keyboard shortcuts that fire even when League has focus.

## Methods

### `register(id, keys, callback)`

Register a global hotkey.

**Parameters:**
- `id: string` — unique hotkey ID within your app
- `keys: string` — key combination (see format below)
- `callback: () => void` — function to call when hotkey fires

**Returns:** `Promise<void>`

```javascript
LOLOverlay.hotkeys.register("toggle-panel", "Ctrl+Shift+O", () => {
  LOLOverlay.windows.toggle("main-panel");
});
```

---

### `unregister(id)`

Unregister a hotkey by ID.

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.hotkeys.unregister("toggle-panel");
```

---

### `isRegistered(keys)`

Check if a key combination is already taken.

**Returns:** `Promise<boolean>`

```javascript
const taken = await LOLOverlay.hotkeys.isRegistered("Ctrl+Shift+O");
```

---

## Key Format

Keys are specified as `Modifier+Key`:

| Modifier | Aliases |
|---|---|
| `Ctrl` | `Control` |
| `Alt` | — |
| `Shift` | — |
| `Win` | `Super` |

**Examples:**
- `Ctrl+Shift+O`
- `Alt+F1`
- `Ctrl+Alt+H`
- `F9`

## Manifest Hotkeys

You can define default hotkeys in `manifest.json`. Users can rebind them in their platform settings:

```json
{
  "hotkeys": [
    {
      "id": "toggle",
      "default_keys": "Ctrl+Shift+O",
      "description": "Toggle main panel"
    }
  ]
}
```

## Best Practice

Always check `isRegistered` before calling `register` to avoid conflicts with other apps or system shortcuts:

```javascript
const key = "Ctrl+Shift+O";
if (!(await LOLOverlay.hotkeys.isRegistered(key))) {
  LOLOverlay.hotkeys.register("toggle", key, () => { /* ... */ });
} else {
  console.warn("Hotkey conflict — using fallback");
}
```
