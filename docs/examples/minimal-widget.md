# Minimal Widget

The smallest possible broken-latch app: a transparent overlay that shows the current game phase.

## Files

### `manifest.json`

```json
{
  "id": "phase-display",
  "name": "Phase Display",
  "version": "1.0.0",
  "description": "Shows current game phase",
  "author": "Example",
  "entry_point": "index.html",
  "permissions": ["game.session"],
  "windows": [
    {
      "id": "main",
      "url": "index.html",
      "default_position": { "x": 10, "y": 10 },
      "default_size": { "width": 180, "height": 50 },
      "draggable": true,
      "opacity": 0.85
    }
  ]
}
```

### `index.html`

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      margin: 0;
      padding: 10px 14px;
      background: rgba(0,0,0,0.75);
      color: #e5e7eb;
      font-family: monospace;
      font-size: 13px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .dot {
      width: 8px; height: 8px;
      border-radius: 50%;
      background: #22c55e;
      flex-shrink: 0;
    }
    .dot.inactive { background: #6b7280; }
  </style>
</head>
<body>
  <div class="dot inactive" id="dot"></div>
  <span id="phase">—</span>
  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script>
    LOLOverlay.init({ appId: "phase-display", version: "1.0.0" });

    const dot = document.getElementById("dot");
    const phaseEl = document.getElementById("phase");

    async function updatePhase(phase) {
      phaseEl.textContent = phase;
      dot.className = "dot" + (phase === "InGame" ? "" : " inactive");
    }

    // Check phase on load
    LOLOverlay.game.getPhase().then(updatePhase);

    // Update on change
    LOLOverlay.game.onPhaseChange((e) => updatePhase(e.current));
  </script>
</body>
</html>
```

## Install

```bash
zip phase-display.lolapp manifest.json index.html
```

Install via **Manage Apps → Install App**.
