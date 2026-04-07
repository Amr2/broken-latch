# Local Development

## Dev Workflow

You don't need to package your app to test it. Point the platform at a local dev server:

### 1. Start a dev server

Any static server works. Using Vite:

```bash
npm create vite@latest my-app -- --template vanilla
cd my-app
npm install
npm run dev  # starts on http://localhost:5173
```

### 2. Sideload the app

In broken-latch → **Manage Apps** → **Install App**, select a manifest that points to your dev server:

```json
{
  "id": "my-app-dev",
  "name": "My App (Dev)",
  "version": "0.0.1",
  "entry_point": "http://localhost:5173/index.html",
  "permissions": ["game.session", "windows.create"],
  "windows": [
    {
      "id": "main",
      "url": "http://localhost:5173/index.html",
      "default_position": { "x": 20, "y": 100 },
      "default_size": { "width": 320, "height": 400 },
      "draggable": true
    }
  ]
}
```

### 3. Enable Debug Mode

In broken-latch **Settings → Developer → Debug mode**, enable:
- **Debug mode** — shows diagnostic overlays
- **Show console logs** — prints SDK logs to devtools
- **Simulate game phases** — cycles through phases without a live game

### 4. Open DevTools

With debug mode enabled, right-click any widget window to open Chrome DevTools.

## Hot Reload

With Vite (or any HMR-capable server), edits to your HTML/JS/CSS will hot-reload inside the widget window automatically.

## Platform HTTP API

You can test API calls directly from curl or Postman:

```bash
# Get current game phase
curl http://localhost:45678/game/phase

# Get installed apps
curl http://localhost:45678/apps

# SDK
curl http://localhost:45678/sdk/loloverlay.js
```

## Simulated Game Phases

With **Simulate game phases** enabled, the platform cycles through all game phases every 10 seconds. Useful for testing `onPhaseChange` handlers without a live League client.

You can also trigger phases manually via the API:

```bash
# Not yet implemented — check platform version for availability
```
