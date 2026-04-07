# Webhook Backend

An app with a Node.js backend that receives game events via webhooks and exposes processed data to the frontend widget.

## Structure

```
webhook-app/
├── manifest.json
├── index.html
└── server/
    ├── server.js
    └── package.json
```

## manifest.json

```json
{
  "id": "webhook-app",
  "name": "Webhook App",
  "version": "1.0.0",
  "entry_point": "index.html",
  "permissions": ["game.session", "windows.create"],
  "windows": [
    {
      "id": "panel",
      "url": "index.html",
      "default_position": { "x": 20, "y": 100 },
      "default_size": { "width": 280, "height": 200 },
      "draggable": true,
      "show_in_phases": ["InGame"]
    }
  ],
  "webhooks": [
    { "event": "game.phase_changed", "url": "http://localhost:3001/webhook/phase" },
    { "event": "game.session_updated", "url": "http://localhost:3001/webhook/session" }
  ],
  "backend": {
    "process": "server/server.exe",
    "port": 3001
  }
}
```

## server/server.js

```javascript
import express from "express";
const app = express();
app.use(express.json());

let state = { phase: "NotRunning", enemyCount: 0 };

app.post("/webhook/phase", (req, res) => {
  state.phase = req.body.current;
  res.sendStatus(200);
});

app.post("/webhook/session", (req, res) => {
  if (req.body.session) {
    state.enemyCount = req.body.session.enemyTeam?.length ?? 0;
  }
  res.sendStatus(200);
});

app.get("/api/state", (req, res) => res.json(state));

app.listen(3001, "127.0.0.1", () =>
  console.log("Backend ready on :3001")
);
```

## index.html

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin:0; padding:16px; background:rgba(0,0,0,0.85); color:white; font:13px Arial; border-radius:8px; }
    .row { display:flex; justify-content:space-between; padding:4px 0; }
    .val { color:#60a5fa; font-weight:bold; }
  </style>
</head>
<body>
  <div class="row"><span>Phase</span><span class="val" id="phase">—</span></div>
  <div class="row"><span>Enemies</span><span class="val" id="enemies">—</span></div>

  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script>
    LOLOverlay.init({ appId: "webhook-app", version: "1.0.0" });

    async function refresh() {
      const data = await fetch("http://localhost:3001/api/state").then(r => r.json());
      document.getElementById("phase").textContent = data.phase;
      document.getElementById("enemies").textContent = data.enemyCount;
    }

    // Poll backend every 2s
    setInterval(refresh, 2000);
    refresh();
  </script>
</body>
</html>
```

## Key Points

- The `backend.process` path is relative to the app's install directory
- Backend receives POST requests with JSON bodies from the platform
- Frontend communicates with backend via standard `fetch` on localhost
- Backend must bind to `127.0.0.1`, not `0.0.0.0`
