# Game Lifecycle API

Provides methods to detect and respond to League of Legends game phases.

## Game Phases

| Phase | Description |
|---|---|
| `NotRunning` | League of Legends is not running |
| `Launching` | Game process detected, client loading |
| `InLobby` | Main client open, no active game |
| `ChampSelect` | Champion selection in progress |
| `Loading` | Loading screen before game starts |
| `InGame` | Active game running |
| `EndGame` | Post-game results screen |

---

## Methods

### `onPhaseChange(callback)`

Register a callback to be called when the game phase changes.

**Parameters:**
- `callback: (event: PhaseChangeEvent) => void`

**Returns:** `void`

```javascript
LOLOverlay.game.onPhaseChange((event) => {
  console.log(`Phase: ${event.previous} → ${event.current}`);

  if (event.current === "InGame") {
    LOLOverlay.windows.show("combat-tracker");
  } else if (event.current === "NotRunning") {
    LOLOverlay.windows.hide("combat-tracker");
  }
});
```

---

### `getPhase()`

Get the current game phase.

**Returns:** `Promise<GamePhase>`

```javascript
const phase = await LOLOverlay.game.getPhase();
if (phase === "InGame") {
  console.log("Game active!");
}
```

---

### `getSession()`

Get the current game session including all players.

**Returns:** `Promise<GameSession | null>` — `null` if no active session.

```javascript
const session = await LOLOverlay.game.getSession();
if (session) {
  console.log("Local player:", session.localPlayer.summonerName);
  session.enemyTeam.forEach(p =>
    console.log(`Enemy: ${p.championName} (${p.summonerName})`)
  );
}
```

---

## Types

### `PhaseChangeEvent`

```typescript
interface PhaseChangeEvent {
  previous: GamePhase;
  current: GamePhase;
  timestamp: number; // Unix ms
}
```

### `GameSession`

```typescript
interface GameSession {
  allyTeam: SessionPlayer[];
  enemyTeam: SessionPlayer[];
  localPlayer: SessionPlayer;
  gameMode: string;
  mapId: number;
}
```

### `SessionPlayer`

```typescript
interface SessionPlayer {
  summonerName: string;
  puuid: string;
  championId: number;
  championName: string;
  teamId: number;
  position?: string; // "TOP" | "JUNGLE" | "MID" | "BOTTOM" | "UTILITY"
}
```
