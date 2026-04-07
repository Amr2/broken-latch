# Backend Integration

Apps can ship a native backend process (any executable) that runs alongside the front-end widgets.

## Manifest Config

```json
{
  "backend": {
    "process": "server.exe",
    "port": 3000
  }
}
```

The platform launches `server.exe` when your app starts and kills it when the app stops.

## Communication

### Frontend → Backend (HTTP)

Make standard `fetch` calls to your backend:

```javascript
const stats = await fetch("http://localhost:3000/api/stats").then(r => r.json());
```

### Backend → Frontend (Webhooks)

Register webhook URLs in your manifest to receive platform events in your backend:

```json
{
  "webhooks": [
    { "event": "game.phase_changed", "url": "http://localhost:3000/webhook/phase" },
    { "event": "game.session_updated", "url": "http://localhost:3000/webhook/session" }
  ]
}
```

The platform sends a POST request with a JSON body to your URL on each event.

## Example: Node.js Backend

```javascript
// server.js (run with node)
import express from "express";
const app = express();
app.use(express.json());

let currentPhase = "NotRunning";

app.post("/webhook/phase", (req, res) => {
  currentPhase = req.body.current;
  console.log("Phase:", currentPhase);
  res.sendStatus(200);
});

app.get("/api/phase", (req, res) => {
  res.json({ phase: currentPhase });
});

app.listen(3000, () => console.log("Backend ready on :3000"));
```

## Security

- Backend processes run as the same user as the platform (no elevation)
- Only bind to `localhost` — never expose ports to external networks
- The platform does not sandbox backend processes; they have full system access
