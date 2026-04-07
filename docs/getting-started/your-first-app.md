# Your First App

This guide walks you through creating a minimal overlay app that shows enemy summoner names during a game.

## Project Structure

```
my-app/
├── manifest.json
├── index.html
└── app.js
```

## 1. Create the Manifest

`manifest.json` tells broken-latch about your app:

```json
{
  "id": "my-first-app",
  "name": "My First App",
  "version": "1.0.0",
  "description": "Shows enemy team info",
  "author": "Your Name",
  "entry_point": "index.html",
  "permissions": ["game.session", "windows.create"],
  "windows": [
    {
      "id": "panel",
      "url": "index.html",
      "default_position": { "x": 20, "y": 100 },
      "default_size": { "width": 280, "height": 320 },
      "draggable": true,
      "resizable": false,
      "persist_position": true,
      "opacity": 0.9,
      "show_in_phases": ["ChampSelect", "InGame"]
    }
  ]
}
```

## 2. Create the HTML

`index.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body {
      margin: 0;
      padding: 16px;
      background: rgba(0,0,0,0.85);
      color: white;
      font-family: Arial, sans-serif;
      font-size: 13px;
      border-radius: 8px;
    }
    h3 { margin: 0 0 12px; font-size: 14px; color: #60a5fa; }
    .player { padding: 4px 0; border-bottom: 1px solid #1f2937; }
    .champ { font-weight: bold; }
    .name { color: #9ca3af; font-size: 12px; }
  </style>
</head>
<body>
  <h3>Enemy Team</h3>
  <div id="players">Waiting for game…</div>
  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script src="app.js"></script>
</body>
</html>
```

## 3. Add Logic

`app.js`:

```javascript
LOLOverlay.init({ appId: "my-first-app", version: "1.0.0" });

const playersEl = document.getElementById("players");

LOLOverlay.game.onPhaseChange(async (event) => {
  if (event.current === "ChampSelect" || event.current === "InGame") {
    const session = await LOLOverlay.game.getSession();
    if (session) {
      playersEl.innerHTML = session.enemyTeam.map(p => `
        <div class="player">
          <div class="champ">${p.championName}</div>
          <div class="name">${p.summonerName}</div>
        </div>
      `).join("");
    }
  } else {
    playersEl.textContent = "Waiting for game…";
  }
});
```

## 4. Package and Install

```bash
# ZIP the files as .lolapp
zip my-first-app.lolapp manifest.json index.html app.js
```

Then in broken-latch → **Manage Apps** → **Install App** → select `my-first-app.lolapp`.

## 5. Test It

Start League of Legends and enter champion select. Your panel will appear automatically.

## Next Steps

- [Understand all manifest options](/getting-started/manifest)
- [Add a hotkey toggle](/examples/hotkey-toggle)
- [Use persistent storage](/guides/persistent-storage)
