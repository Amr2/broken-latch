# Game Phase Tracking

## Phase Lifecycle

```
NotRunning → Launching → InLobby → ChampSelect → Loading → InGame → EndGame → InLobby
```

## Responding to Every Phase

```javascript
LOLOverlay.init({ appId: "my-app", version: "1.0.0" });

LOLOverlay.game.onPhaseChange(async (event) => {
  switch (event.current) {
    case "NotRunning":
      // League closed — hide everything
      await LOLOverlay.windows.hide("main");
      break;

    case "ChampSelect":
      // Show draft helper
      await LOLOverlay.windows.show("draft-helper");
      const session = await LOLOverlay.game.getSession();
      if (session) renderDraftInfo(session);
      break;

    case "Loading":
      // Loading screen — show tips
      await LOLOverlay.windows.hide("draft-helper");
      await LOLOverlay.windows.show("loading-tips");
      break;

    case "InGame":
      // Game active — show main HUD
      await LOLOverlay.windows.hide("loading-tips");
      await LOLOverlay.windows.show("main");
      break;

    case "EndGame":
      // Post-game — show summary
      await LOLOverlay.windows.hide("main");
      await LOLOverlay.windows.show("summary");
      break;
  }
});
```

## Using `show_in_phases` in Manifest

Alternatively, let the platform auto-manage visibility via the manifest. The `show_in_phases` field auto-shows/hides windows on phase transitions:

```json
{
  "windows": [
    { "id": "champ-helper", "show_in_phases": ["ChampSelect"] },
    { "id": "in-game-hud", "show_in_phases": ["InGame"] },
    { "id": "post-game",   "show_in_phases": ["EndGame"] }
  ]
}
```

This is simpler and recommended when you don't need async data on transition.

## Initial Phase Check

On startup, check the current phase immediately rather than waiting for a change:

```javascript
LOLOverlay.init({ appId: "my-app", version: "1.0.0" });

// Handle current phase on load
const phase = await LOLOverlay.game.getPhase();
handlePhase(phase);

// Then listen for future changes
LOLOverlay.game.onPhaseChange((e) => handlePhase(e.current));

function handlePhase(phase) { /* ... */ }
```
