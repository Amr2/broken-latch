# Hunter Mode App - Tasks Overview

## What is Hunter Mode?

Hunter Mode is a League of Legends coaching overlay app that **runs on top of the broken-latch platform**. It provides:

- Pre-game enemy intel (loading screen panel)
- Win condition analysis (draft composition insights)
- Hunter Focus (in-game enemy player deep-dive)
- Post-game performance review

**Critical**: Hunter Mode NEVER touches Windows API, DirectX, or game memory. All platform integration happens via the LOLOverlay JavaScript SDK.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  broken-latch Platform (provides infrastructure)    │
│  - Overlay window                                    │
│  - Game detection                                    │
│  - Hotkeys, widgets, storage                        │
│  - LOLOverlay.js SDK                                 │
└─────────────────────────────────────────────────────┘
                        ↑
                        │ LOLOverlay SDK
                        ↓
┌─────────────────────────────────────────────────────┐
│  Hunter Mode App (this is what you're building)     │
│  ┌──────────────┐  ┌──────────────────────────────┐ │
│  │  Frontend    │  │  Backend (Node.js)           │ │
│  │  React + TS  │  │  - Riot API client           │ │
│  │  4 panels    │  │  - Analysis services         │ │
│  └──────────────┘  │  - Express server (port 45679)│ │
│                    └──────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

---

## Task Dependencies Graph

```
01: Project Setup & Manifest
    ↓
02: Backend - Riot API Client ←──────┐
    ↓                                 │
03: Backend - Caching System          │
    ↓                                 │
04: Backend - Enemy Profiles Feature ─┤
    ↓                                 │
05: Backend - Win Condition Feature ──┤
    ↓                                 │
06: Backend - Hunter Focus Feature ───┤
    ↓                                 │
07: Backend - Post-Game Feature ──────┘
    ↓
08: Frontend - Platform Integration (LOLOverlay SDK wrapper)
    ↓
09: Frontend - Shared Components
    ↓
10: Frontend - Loading Panel ←───┐
    ↓                             │
11: Frontend - Win Condition Panel│
    ↓                             │
12: Frontend - Hunter Focus Panel │
    ↓                             │
13: Frontend - Post-Game Panel ───┘
    ↓
14: Frontend - Settings & Setup Flow
    ↓
15: Integration Testing & .lolapp Packaging
```

---

## Task List with Status

### Phase 1: Foundation (P0)

- [ ] **Task 01**: Project Setup & Manifest
  - **File**: `01-project-setup.md`
  - **Time**: 3-4 hours
  - **Description**: Create manifest.json, folder structure, install dependencies

### Phase 2: Backend Services (P0)

- [ ] **Task 02**: Riot API Client
  - **File**: `02-backend-riot-api-client.md`
  - **Time**: 6-8 hours
  - **Description**: Axios client with rate limiting for all Riot API endpoints

- [ ] **Task 03**: Caching System
  - **File**: `03-backend-caching-system.md`
  - **Time**: 4-5 hours
  - **Description**: LowDB cache with TTL for API responses

- [ ] **Task 04**: Enemy Profiles Feature
  - **File**: `04-backend-enemy-profiles-feature.md`
  - **Time**: 8-10 hours
  - **Description**: Route + service for loading screen enemy analysis

- [ ] **Task 05**: Win Condition Feature
  - **File**: `05-backend-win-condition-feature.md`
  - **Time**: 6-8 hours
  - **Description**: Draft composition analyzer using Data Dragon

- [ ] **Task 06**: Hunter Focus Feature
  - **File**: `06-backend-hunter-focus-feature.md`
  - **Time**: 8-10 hours
  - **Description**: Deep player/champion analysis for locked target

- [ ] **Task 07**: Post-Game Feature
  - **File**: `07-backend-postgame-feature.md`
  - **Time**: 5-6 hours
  - **Description**: Performance comparison vs averages

### Phase 3: Frontend Foundation (P0)

- [ ] **Task 08**: Platform Integration Layer
  - **File**: `08-frontend-platform-integration.md`
  - **Time**: 4-5 hours
  - **Description**: LOLOverlay SDK wrapper in platform.ts

- [ ] **Task 09**: Shared Components
  - **File**: `09-frontend-shared-components.md`
  - **Time**: 6-8 hours
  - **Description**: ChampionIcon, RankBadge, ThreatMeter, etc.

### Phase 4: Feature Panels (P0)

- [ ] **Task 10**: Loading Panel
  - **File**: `10-frontend-loading-panel.md`
  - **Time**: 8-10 hours
  - **Description**: Enemy profiles table with expand/collapse

- [ ] **Task 11**: Win Condition Panel
  - **File**: `11-frontend-win-condition-panel.md`
  - **Time**: 6-8 hours
  - **Description**: Draft analysis sidebar

- [ ] **Task 12**: Hunter Focus Panel
  - **File**: `12-frontend-hunter-focus-panel.md`
  - **Time**: 10-12 hours
  - **Description**: Target deep-dive sidebar

- [ ] **Task 13**: Post-Game Panel
  - **File**: `13-frontend-postgame-panel.md`
  - **Time**: 6-8 hours
  - **Description**: Performance review card

### Phase 5: Polish & Distribution (P1)

- [ ] **Task 14**: Settings & Setup Flow
  - **File**: `14-frontend-settings-flow.md`
  - **Time**: 6-8 hours
  - **Description**: First-run API key setup, region selection

- [ ] **Task 15**: Integration Testing & Packaging
  - **File**: `15-integration-testing.md`
  - **Time**: 8-10 hours
  - **Description**: E2E tests, .lolapp package creation

---

## Total Estimated Time: 90-110 hours

---

## Key Differences from Platform Development

| Concern        | Platform (broken-latch)       | Hunter Mode App                    |
| -------------- | ----------------------------- | ---------------------------------- |
| Windows API    | ✅ Directly uses              | ❌ Never touches                   |
| DirectX        | ✅ C++ hook DLL               | ❌ Never touches                   |
| Game detection | ✅ Implements                 | ❌ Consumes via SDK                |
| Hotkeys        | ✅ RegisterHotKey API         | ❌ Registers via SDK               |
| Overlay window | ✅ Creates transparent window | ❌ Renders into provided container |
| Riot API       | ❌ Not involved               | ✅ All game data from here         |
| Data analysis  | ❌ Just infrastructure        | ✅ Core functionality              |

---

## Technology Stack

### Backend

- **Runtime**: Node.js 20+
- **HTTP Framework**: Express.js
- **HTTP Client**: Axios
- **Cache**: LowDB (JSON file-based)
- **Rate Limiting**: Custom token bucket
- **API**: Riot Games Official API (Match v5, Summoner v4, Champion Mastery v4, League v4)

### Frontend

- **UI Framework**: React 18
- **Language**: TypeScript
- **Build Tool**: Vite
- **State Management**: Zustand
- **Styling**: TailwindCSS
- **Platform Integration**: LOLOverlay SDK (from broken-latch)

### Distribution

- **Package Format**: `.lolapp` (zip with manifest.json)
- **Installed To**: `%AppData%\broken-latch\apps\hunter-mode\`
- **Backend Process**: Launched automatically by platform via manifest

---

## Manifest Structure (Created in Task 01)

```json
{
  "id": "hunter-mode",
  "name": "Hunter Mode",
  "version": "1.0.0",
  "description": "League of Legends coaching overlay with enemy intel and win condition analysis",
  "author": "Your Name",
  "permissions": [
    "game_lifecycle",
    "storage",
    "hotkeys",
    "windows",
    "notifications"
  ],
  "backend": {
    "process": "backend/server.js",
    "runtime": "node",
    "args": []
  },
  "widgets": [
    {
      "id": "loading-panel",
      "title": "Enemy Profiles",
      "width": 800,
      "height": 250,
      "defaultPosition": { "x": "center", "y": 50 },
      "draggable": true,
      "showInPhases": ["Loading"]
    },
    {
      "id": "win-condition-panel",
      "title": "Win Conditions",
      "width": 350,
      "height": 500,
      "defaultPosition": { "x": "right-10", "y": "center" },
      "draggable": true,
      "showInPhases": ["Loading", "InGame"]
    },
    {
      "id": "hunter-focus-panel",
      "title": "Hunter Focus",
      "width": 320,
      "height": 600,
      "defaultPosition": { "x": 10, "y": "center" },
      "draggable": true,
      "showInPhases": ["InGame"]
    },
    {
      "id": "post-game-panel",
      "title": "Game Review",
      "width": 700,
      "height": 500,
      "defaultPosition": { "x": "center", "y": "center" },
      "draggable": false,
      "showInPhases": ["EndGame"]
    }
  ],
  "hotkeys": [
    {
      "id": "toggle-hunter-focus",
      "keys": "Alt+H",
      "description": "Toggle Hunter Focus panel"
    },
    {
      "id": "toggle-win-condition",
      "keys": "Alt+W",
      "description": "Toggle Win Condition panel"
    },
    {
      "id": "lock-target-1",
      "keys": "Alt+1",
      "description": "Lock Hunter Focus on enemy 1"
    },
    {
      "id": "lock-target-2",
      "keys": "Alt+2",
      "description": "Lock Hunter Focus on enemy 2"
    },
    {
      "id": "lock-target-3",
      "keys": "Alt+3",
      "description": "Lock Hunter Focus on enemy 3"
    },
    {
      "id": "lock-target-4",
      "keys": "Alt+4",
      "description": "Lock Hunter Focus on enemy 4"
    },
    {
      "id": "lock-target-5",
      "keys": "Alt+5",
      "description": "Lock Hunter Focus on enemy 5"
    },
    {
      "id": "dismiss-post-game",
      "keys": "Escape",
      "description": "Close post-game review"
    }
  ],
  "webhooks": [
    {
      "event": "game_phase_changed",
      "url": "http://localhost:45679/webhook"
    },
    {
      "event": "game_session_ready",
      "url": "http://localhost:45679/webhook"
    }
  ]
}
```

---

## Folder Structure (Created in Task 01)

```
hunter-mode/
├── manifest.json
├── frontend/
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── panels/
│   │   │   ├── LoadingPanel/
│   │   │   ├── WinCondition/
│   │   │   ├── HunterFocus/
│   │   │   └── PostGame/
│   │   ├── shared/
│   │   │   ├── ChampionIcon.tsx
│   │   │   ├── RankBadge.tsx
│   │   │   ├── etc...
│   │   ├── hooks/
│   │   │   ├── useGamePhase.ts
│   │   │   ├── useEnemyProfiles.ts
│   │   │   ├── etc...
│   │   ├── store/
│   │   │   └── appStore.ts
│   │   └── lib/
│   │       ├── platform.ts      # LOLOverlay SDK wrapper
│   │       └── constants.ts
│   └── public/
│       └── ...
├── backend/
│   ├── package.json
│   ├── server.js
│   ├── routes/
│   │   ├── enemies.js
│   │   ├── wincondition.js
│   │   ├── hunter.js
│   │   ├── postgame.js
│   │   └── webhook.js
│   ├── services/
│   │   ├── riotApi.js
│   │   ├── playerProfile.js
│   │   ├── winConditionAnalyzer.js
│   │   ├── postGameAnalyzer.js
│   │   └── cache.js
│   └── data/
│       └── cache.json
└── icons/
    ├── icon-32.png
    └── icon-256.png
```

---

## Data Flow Example

### Loading Screen Panel Flow:

```
1. broken-latch detects game phase = "Loading"
2. Platform sends webhook to http://localhost:45679/webhook
3. Hunter Mode backend receives event
4. Frontend calls useEnemyProfiles() hook (triggered by phase change from SDK)
5. Hook calls backend GET /enemies
6. Backend:
   - Calls Riot API for each enemy summoner
   - Calls Match v5 for last 10 games per champion
   - Analyzes data (win rate, KDA, build path, playstyle)
   - Returns PlayerProfile[] array
7. Frontend stores in Zustand
8. LoadingPanel renders 5 EnemyRow components
9. Platform automatically shows widget (showInPhases: ["Loading"])
10. After 8 seconds in InGame phase, platform auto-hides it
```

---

## Testing Strategy

### Unit Tests (Per Task)

- Backend services: Jest tests for each route and analyzer
- Frontend components: React Testing Library
- Hooks: Test state changes and API calls

### Integration Tests (Task 15)

- Full backend API flow (mock Riot API responses)
- Frontend + backend together
- Platform SDK integration

### E2E Tests (Task 15)

- Install .lolapp into broken-latch
- Simulate game phase changes
- Verify all panels render
- Verify hotkeys work
- Check storage persistence

---

## Performance Requirements

| Metric                 | Target                   |
| ---------------------- | ------------------------ |
| Backend RAM (idle)     | <40MB                    |
| Backend RAM (active)   | <100MB                   |
| Backend CPU            | <1%                      |
| Frontend RAM per panel | <15MB                    |
| Riot API call latency  | <2s for 5 enemy profiles |
| Panel render time      | <100ms                   |
| Hotkey response        | <50ms                    |

---

## Next Steps for You

1. **Wait for broken-latch platform Task 09 (SDK) to be complete**
   - You need the LOLOverlay SDK to integrate
2. **Start with Hunter Mode Task 01** (project setup)

3. **Build all backend tasks (02-07)** first
   - Can be tested independently with Postman/curl
4. **Then frontend tasks (08-14)**
   - Needs platform to be running to test

5. **Finally integration testing (15)**

---

## Important Notes

### Riot API Key

- User provides their own key via settings (free tier: 20 req/sec, 100 req/2min)
- Stored in environment variable: `HUNTER_MODE_RIOT_API_KEY`
- **NEVER hardcoded in source**

### ToS Compliance

- ✅ Uses only official Riot API
- ✅ Historical data only (no live in-game calculations)
- ✅ No memory reading
- ✅ No packet sniffing
- ✅ Must be approved in Riot Developer Portal before public release

### Freemium Model (Not implemented in these tasks, future enhancement)

- Free tier: Basic features
- Premium "Hunter Pass": Hunter Focus unlocked, extended history

---

## Files Generated by All Tasks

See folder structure above. Total: ~60-70 files.

---

## Questions?

Each task prompt is self-contained with:

- ✅ Full implementation code
- ✅ Integration points with previous tasks
- ✅ Testing requirements
- ✅ Acceptance criteria

**Platform must be working before starting Hunter Mode development!**
