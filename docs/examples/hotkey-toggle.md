# Hotkey Toggle

An overlay panel that can be shown and hidden with a global hotkey.

## manifest.json

```json
{
  "id": "hotkey-example",
  "name": "Hotkey Example",
  "version": "1.0.0",
  "entry_point": "index.html",
  "permissions": ["windows.create", "hotkeys.register"],
  "windows": [
    {
      "id": "panel",
      "url": "index.html",
      "default_position": { "x": 50, "y": 150 },
      "default_size": { "width": 300, "height": 200 },
      "draggable": true
    }
  ],
  "hotkeys": [
    {
      "id": "toggle",
      "default_keys": "Ctrl+Shift+H",
      "description": "Toggle panel visibility"
    }
  ]
}
```

## index.html

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      margin: 0; padding: 20px;
      background: rgba(15,23,42,0.9);
      color: white;
      font-family: Arial, sans-serif;
      border-radius: 8px;
      border: 1px solid rgba(255,255,255,0.1);
    }
    h3 { margin: 0 0 8px; color: #60a5fa; }
    p { margin: 0; color: #94a3b8; font-size: 13px; }
  </style>
</head>
<body>
  <h3>Hotkey Panel</h3>
  <p>Press <kbd>Ctrl+Shift+H</kbd> to toggle this panel.</p>
  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script>
    LOLOverlay.init({ appId: "hotkey-example", version: "1.0.0" });

    // Register toggle hotkey
    LOLOverlay.hotkeys.register("toggle", "Ctrl+Shift+H", async () => {
      await LOLOverlay.windows.toggle("panel");
    });
  </script>
</body>
</html>
```

## Key Points

- Hotkeys fire even when League has focus
- `windows.toggle()` shows if hidden, hides if visible
- Users can rebind hotkeys in broken-latch Settings
- Always declare hotkeys in `manifest.json` so the platform knows about them
