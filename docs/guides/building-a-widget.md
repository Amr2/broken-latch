# Building a Widget

This guide walks you through creating a simple overlay widget that shows enemy summoner names during a game.

## Step 1: Create the HTML

`widget.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      margin: 0;
      padding: 20px;
      background: rgba(0, 0, 0, 0.8);
      color: white;
      font-family: Arial, sans-serif;
    }
    .header {
      font-size: 16px;
      font-weight: bold;
      margin-bottom: 10px;
      cursor: move;
    }
    .content { font-size: 13px; }
  </style>
</head>
<body>
  <div class="header">My Widget</div>
  <div class="content" id="content">Waiting for game...</div>
  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script src="widget.js"></script>
</body>
</html>
```

## Step 2: Add JavaScript Logic

`widget.js`:

```javascript
LOLOverlay.init({ appId: "my-widget-app", version: "1.0.0" });

const contentEl = document.getElementById("content");

LOLOverlay.game.onPhaseChange(async (event) => {
  if (event.current === "InGame") {
    const session = await LOLOverlay.game.getSession();
    if (session) {
      const enemyNames = session.enemyTeam
        .map(p => `${p.championName} — ${p.summonerName}`)
        .join("<br>");
      contentEl.innerHTML = `<strong>Enemies:</strong><br>${enemyNames}`;
    }
  } else {
    contentEl.textContent = "Waiting for game...";
  }
});
```

## Step 3: Create Manifest

`manifest.json`:

```json
{
  "id": "my-widget-app",
  "name": "My Widget",
  "version": "1.0.0",
  "description": "A simple overlay widget",
  "author": "Your Name",
  "entry_point": "widget.html",
  "permissions": ["game.session", "windows.create"],
  "windows": [
    {
      "id": "main",
      "url": "widget.html",
      "default_position": { "x": 20, "y": 100 },
      "default_size": { "width": 250, "height": 300 },
      "draggable": true,
      "resizable": false,
      "persist_position": true,
      "opacity": 0.9,
      "show_in_phases": ["InGame"]
    }
  ]
}
```

## Step 4: Package and Install

```bash
zip my-widget-app.lolapp manifest.json widget.html widget.js
```

Install via **Manage Apps → Install App**.

## Next Steps

- [Add hotkeys](/examples/hotkey-toggle)
- [Persist user settings](/guides/persistent-storage)
- [Connect to a backend](/guides/backend-integration)
