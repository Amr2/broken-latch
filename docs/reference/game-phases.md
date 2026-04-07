# Game Phases Reference

## Phase Values

| Phase | Value | Trigger |
|---|---|---|
| `NotRunning` | League not detected | Process exits or was never running |
| `Launching` | League process found | `LeagueClient.exe` appears in process list |
| `InLobby` | Client fully loaded | Home screen visible, no active queue |
| `ChampSelect` | Champ select open | Queue popped and accepted |
| `Loading` | Loading screen | Game found, map loading |
| `InGame` | In game | Match active |
| `EndGame` | Post-game screen | Match ended, stats visible |

## Phase Transition Diagram

```
             restart
    ┌─────────────────────────────────────────────────┐
    ▼                                                 │
NotRunning → Launching → InLobby → ChampSelect → Loading → InGame → EndGame
                                       │                               │
                                       └───────────────────────────────┘
                                              dodge / leave queue
```

## Detection Method

The platform detects phases by polling for the `LeagueClient.exe` and `League of Legends.exe` processes using `sysinfo`. Phase transitions emit a `game_phase_changed` event on the Tauri event bus, which the SDK translates into `onPhaseChange` callbacks.

## `show_in_phases` Usage

Windows with `show_in_phases: ["ChampSelect", "InGame"]` are automatically shown when those phases become active and hidden otherwise.

```json
{ "show_in_phases": ["InGame"] }          // only during active game
{ "show_in_phases": ["ChampSelect"] }     // only during draft
{ "show_in_phases": [] }                  // always visible (default)
```
