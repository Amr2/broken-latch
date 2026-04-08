# Multi-Panel App

An app with separate windows for champion select and in-game, each showing different information.

## Structure

```
multi-panel/
├── manifest.json
├── champ-select.html
├── in-game.html
└── app.js
```

## manifest.json

```json
{
  "id": "multi-panel",
  "name": "Multi Panel",
  "version": "1.0.0",
  "entry_point": "champ-select.html",
  "permissions": ["game.session", "windows.create", "hotkeys.register"],
  "windows": [
    {
      "id": "champ-select",
      "url": "champ-select.html",
      "default_position": { "x": 20, "y": 80 },
      "default_size": { "width": 260, "height": 360 },
      "draggable": true,
      "show_in_phases": ["ChampSelect"]
    },
    {
      "id": "in-game",
      "url": "in-game.html",
      "default_position": { "x": 20, "y": 80 },
      "default_size": { "width": 240, "height": 300 },
      "draggable": true,
      "show_in_phases": ["InGame"]
    }
  ]
}
```

Using `show_in_phases`, the platform automatically shows/hides each window at the right time.

## champ-select.html

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin:0; padding:16px; background:rgba(0,0,0,0.85); color:white; font:13px Arial; border-radius:8px; }
    h3 { margin:0 0 10px; color:#60a5fa; font-size:14px; }
    .player { padding:4px 0; border-bottom:1px solid #1f2937; }
    .role { color:#9ca3af; font-size:11px; }
  </style>
</head>
<body>
  <h3>Champion Select</h3>
  <div id="picks">Waiting...</div>
  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script>
    LOLOverlay.init({ appId: "multi-panel", version: "1.0.0" });

    LOLOverlay.game.onPhaseChange(async (e) => {
      if (e.current === "ChampSelect") {
        const session = await LOLOverlay.game.getSession();
        if (!session) return;

        document.getElementById("picks").innerHTML =
          session.enemyTeam.map(p =>
            `<div class="player">
              <strong>${p.championName || "?"}</strong>
              <span class="role">${p.summonerName}</span>
            </div>`
          ).join("");
      }
    });
  </script>
</body>
</html>
```

## in-game.html

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin:0; padding:16px; background:rgba(0,0,0,0.8); color:white; font:13px Arial; border-radius:8px; }
    h3 { margin:0 0 10px; color:#4ade80; font-size:14px; }
    .stat { display:flex; justify-content:space-between; padding:3px 0; }
    .val { color:#d1d5db; }
  </style>
</head>
<body>
  <h3>In Game</h3>
  <div id="info">
    <div class="stat"><span>Phase</span><span class="val" id="phase">—</span></div>
    <div class="stat"><span>Mode</span><span class="val" id="mode">—</span></div>
    <div class="stat"><span>Map</span><span class="val" id="map">—</span></div>
  </div>
  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script>
    LOLOverlay.init({ appId: "multi-panel", version: "1.0.0" });

    LOLOverlay.game.onPhaseChange(async (e) => {
      document.getElementById("phase").textContent = e.current;
      if (e.current === "InGame") {
        const session = await LOLOverlay.game.getSession();
        if (session) {
          document.getElementById("mode").textContent = session.gameMode;
          document.getElementById("map").textContent = session.mapId;
        }
      }
    });
  </script>
</body>
</html>
```

## Package

```bash
zip multi-panel.lolapp manifest.json champ-select.html in-game.html
```
