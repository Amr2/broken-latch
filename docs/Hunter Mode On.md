# Hunter Mode — App Specification
## Built on LOLOverlay Platform

---

## What Hunter Mode Is

Hunter Mode is a League of Legends coaching app that runs on top of the **LOLOverlay platform**. It is NOT a standalone application — it is a `.lolapp` package that installs into LOLOverlay and uses the LOLOverlay JavaScript SDK and HTTP API exclusively for all overlay rendering, window management, hotkeys, and game lifecycle events.

Hunter Mode is responsible for exactly one thing: **fetching data from the Riot Games API, analyzing it, and rendering coaching widgets through the LOLOverlay platform.**

Hunter Mode never touches:
- Windows API
- DirectX
- Process injection
- Game memory
- The League of Legends client directly

All of that is handled by LOLOverlay. Hunter Mode only builds on top of it.

---

## What Hunter Mode Does NOT Need to Build

Because LOLOverlay handles these, Hunter Mode never implements:

| Concern | Handled By |
|---|---|
| Transparent overlay window | LOLOverlay platform |
| DirectX hook / compositing | LOLOverlay platform |
| Game phase detection | LOLOverlay platform (`LOLOverlay.game.onPhaseChange`) |
| Global hotkey registration | LOLOverlay platform (`LOLOverlay.hotkeys.register`) |
| Draggable widget windows | LOLOverlay platform (`LOLOverlay.windows.create`) |
| Widget position persistence | LOLOverlay platform (`persistPosition: true`) |
| App auto-update | LOLOverlay platform (via `updateUrl` in manifest) |
| App distribution | LOLOverlay app store |

---

## Technology Stack

| Layer | Technology | Purpose |
|---|---|---|
| UI | React 18 + TypeScript + TailwindCSS | All widget panels |
| Build tool | Vite | Fast dev builds, hot reload via `loloverlay dev` |
| State management | Zustand | Global app state across panels |
| Platform integration | LOLOverlay JS SDK (`LOLOverlay.*`) | Windows, hotkeys, events, storage |
| Backend process | Node.js (Express) | Riot API calls, data analysis, caching |
| Backend HTTP | Express.js on port 45679 | Receives platform webhooks, serves data to frontend |
| Local cache | LowDB (JSON file, via backend) | Cache Riot API responses — no external DB needed |
| HTTP client | Axios (backend) | All Riot API calls |
| Package format | `.lolapp` (zip) | Distributed via LOLOverlay app store |

### Why Node.js for the backend?
Hunter Mode needs a backend process for Riot API calls (API key must never be in frontend JS), rate limiting, and caching. Node.js is the lightest option that app developers know well. The LOLOverlay platform manifest supports a `backend.process` field that auto-launches this process when the app starts.

---

## Project Folder Structure

```
hunter-mode/
├── manifest.json                  # LOLOverlay app manifest
├── frontend/                      # React app (rendered in LOLOverlay webview)
│   ├── index.html                 # App shell — loads SDK + initializes panels
│   ├── src/
│   │   ├── main.tsx               # React entry point
│   │   ├── App.tsx                # Root component — mounts all panels
│   │   ├── panels/
│   │   │   ├── LoadingPanel/
│   │   │   │   ├── LoadingPanel.tsx
│   │   │   │   ├── EnemyRow.tsx
│   │   │   │   ├── EnemyRowExpanded.tsx
│   │   │   │   └── LoadingPanel.css
│   │   │   ├── WinCondition/
│   │   │   │   ├── WinConditionPanel.tsx
│   │   │   │   ├── PowerTimeline.tsx
│   │   │   │   ├── KeyThreatBadge.tsx
│   │   │   │   └── WinConditionPanel.css
│   │   │   ├── HunterFocus/
│   │   │   │   ├── HunterFocusPanel.tsx
│   │   │   │   ├── TargetHeader.tsx
│   │   │   │   ├── AbilityTendency.tsx
│   │   │   │   ├── PositionalTendency.tsx
│   │   │   │   ├── CounterTips.tsx
│   │   │   │   ├── ThreatMeter.tsx
│   │   │   │   └── HunterFocusPanel.css
│   │   │   └── PostGame/
│   │   │       ├── PostGamePanel.tsx
│   │   │       ├── StatCompare.tsx
│   │   │       ├── HunterOutcome.tsx
│   │   │       └── PostGamePanel.css
│   │   ├── shared/
│   │   │   ├── ChampionIcon.tsx        # Loads from Data Dragon CDN
│   │   │   ├── RankBadge.tsx
│   │   │   ├── WinRatePill.tsx
│   │   │   ├── KdaDisplay.tsx
│   │   │   ├── StatBar.tsx             # Horizontal fill bar with label
│   │   │   ├── ThreatIndicator.tsx     # 1–5 dot indicator
│   │   │   └── LoadingSpinner.tsx
│   │   ├── hooks/
│   │   │   ├── useGamePhase.ts         # Subscribes to LOLOverlay phase events
│   │   │   ├── useEnemyProfiles.ts     # Fetches from backend on phase trigger
│   │   │   ├── useWinCondition.ts      # Fetches from backend on session ready
│   │   │   ├── useHunterTarget.ts      # Manages locked target state + fetch
│   │   │   └── usePostGame.ts          # Fetches from backend on EndGame
│   │   ├── store/
│   │   │   └── appStore.ts             # Zustand store — all app state
│   │   └── lib/
│   │       ├── platform.ts             # LOLOverlay SDK wrapper + initialization
│   │       └── constants.ts            # API base URL, Data Dragon URL, etc.
│   ├── package.json
│   └── vite.config.ts
├── backend/                       # Node.js backend process
│   ├── server.js                  # Express app entry point
│   ├── routes/
│   │   ├── enemies.js             # GET /enemies — fetch + analyze enemy profiles
│   │   ├── wincondition.js        # GET /win-condition — draft analysis
│   │   ├── hunter.js              # GET /hunter/:summonerName/:championId
│   │   ├── postgame.js            # GET /post-game/:matchId/:puuid
│   │   └── webhook.js             # POST /webhook — receives LOLOverlay events
│   ├── services/
│   │   ├── riotApi.js             # Riot API client with rate limiting
│   │   ├── playerProfile.js       # Profile analysis logic
│   │   ├── winConditionAnalyzer.js# Draft composition analysis
│   │   ├── postGameAnalyzer.js    # Post-game performance comparison
│   │   └── cache.js               # LowDB cache with TTL
│   ├── data/
│   │   └── cache.json             # LowDB persistent cache file
│   └── package.json
└── icons/
    ├── icon-32.png
    └── icon-256.png
```

---

## Manifest (manifest.json)

```json
{
    "id": "hunter-mode",
    "name": "Hunter Mode",
    "version": "1.0.0",
    "description": "League of Legends coaching overlay — pre-game intel, draft analysis, and enemy tendency insights",
    "author": "ilaka",
    "updateUrl": "https://releases.huntermode.gg/latest.json",
    "entryPoint": "frontend/index.html",
    "permissions": [
        "game.session",
        "windows.create",
        "hotkeys.register",
        "storage",
        "notify"
    ],
    "windows": [
        {
            "id": "loading-panel",
            "url": "frontend/index.html?panel=loading",
            "defaultPosition": { "x": 960, "y": 20 },
            "defaultSize": { "width": 480, "height": 320 },
            "minSize": { "width": 360, "height": 200 },
            "draggable": true,
            "resizable": false,
            "persistPosition": true,
            "clickThrough": false,
            "opacity": 0.92,
            "showInPhases": ["Loading"]
        },
        {
            "id": "win-condition-panel",
            "url": "frontend/index.html?panel=wincondition",
            "defaultPosition": { "x": 20, "y": 20 },
            "defaultSize": { "width": 320, "height": 440 },
            "minSize": { "width": 260, "height": 300 },
            "draggable": true,
            "resizable": false,
            "persistPosition": true,
            "clickThrough": false,
            "opacity": 0.92,
            "showInPhases": ["Loading", "InGame"]
        },
        {
            "id": "hunter-focus-panel",
            "url": "frontend/index.html?panel=hunterfocus",
            "defaultPosition": { "x": 20, "y": 200 },
            "defaultSize": { "width": 300, "height": 520 },
            "minSize": { "width": 260, "height": 400 },
            "draggable": true,
            "resizable": false,
            "persistPosition": true,
            "clickThrough": false,
            "opacity": 0.92,
            "showInPhases": ["InGame"]
        },
        {
            "id": "post-game-panel",
            "url": "frontend/index.html?panel=postgame",
            "defaultPosition": { "x": 480, "y": 140 },
            "defaultSize": { "width": 640, "height": 520 },
            "minSize": { "width": 480, "height": 400 },
            "draggable": true,
            "resizable": false,
            "persistPosition": false,
            "clickThrough": false,
            "opacity": 0.96,
            "showInPhases": ["EndGame"]
        }
    ],
    "hotkeys": [
        {
            "id": "toggle-win-condition",
            "defaultKeys": "Alt+W",
            "description": "Toggle Win Condition panel"
        },
        {
            "id": "toggle-hunter-focus",
            "defaultKeys": "Alt+H",
            "description": "Toggle Hunter Focus panel"
        },
        {
            "id": "dismiss-post-game",
            "defaultKeys": "Escape",
            "description": "Dismiss post-game review"
        },
        {
            "id": "lock-target-1",
            "defaultKeys": "Alt+1",
            "description": "Lock Hunter Focus on enemy 1"
        },
        {
            "id": "lock-target-2",
            "defaultKeys": "Alt+2",
            "description": "Lock Hunter Focus on enemy 2"
        },
        {
            "id": "lock-target-3",
            "defaultKeys": "Alt+3",
            "description": "Lock Hunter Focus on enemy 3"
        },
        {
            "id": "lock-target-4",
            "defaultKeys": "Alt+4",
            "description": "Lock Hunter Focus on enemy 4"
        },
        {
            "id": "lock-target-5",
            "defaultKeys": "Alt+5",
            "description": "Lock Hunter Focus on enemy 5"
        }
    ],
    "webhooks": [
        { "event": "game_phase_changed", "url": "http://localhost:45679/webhook" },
        { "event": "game_session_ready", "url": "http://localhost:45679/webhook" }
    ],
    "backend": {
        "process": "backend/server.js",
        "runtime": "node",
        "port": 45679
    }
}
```

---

## Platform Integration — Frontend (platform.ts)

This is the single file that wraps all LOLOverlay SDK calls. Every other file in the app imports from here — never calling `LOLOverlay.*` directly, making it easy to mock in tests.

```typescript
// frontend/src/lib/platform.ts

declare const LOLOverlay: any;

export async function initPlatform(): Promise<void> {
    await LOLOverlay.init({
        appId: "hunter-mode",
        version: "1.0.0"
    });
}

// ─── GAME ────────────────────────────────────────────────────────
export function onPhaseChange(cb: (event: PhaseChangeEvent) => void): void {
    LOLOverlay.game.onPhaseChange(cb);
}

export async function getPhase(): Promise<GamePhase> {
    return LOLOverlay.game.getPhase();
}

export async function getSession(): Promise<GameSession> {
    return LOLOverlay.game.getSession();
}

// ─── WINDOWS ─────────────────────────────────────────────────────
export async function showPanel(id: string): Promise<void> {
    return LOLOverlay.windows.show(id);
}

export async function hidePanel(id: string): Promise<void> {
    return LOLOverlay.windows.hide(id);
}

export async function setPanelOpacity(id: string, opacity: number): Promise<void> {
    return LOLOverlay.windows.setOpacity(id, opacity);
}

// ─── HOTKEYS ─────────────────────────────────────────────────────
export function registerHotkey(id: string, keys: string, cb: () => void): void {
    LOLOverlay.hotkeys.register({ id, keys, onPress: cb });
}

// ─── STORAGE ─────────────────────────────────────────────────────
export async function saveToStorage(key: string, value: unknown): Promise<void> {
    return LOLOverlay.storage.set(key, value);
}

export async function loadFromStorage<T>(key: string): Promise<T | null> {
    return LOLOverlay.storage.get(key);
}

// ─── NOTIFICATIONS ───────────────────────────────────────────────
export function toast(message: string, type: "info"|"success"|"warning"|"error" = "info"): void {
    LOLOverlay.notify.toast({ message, type, duration: 3000, position: "top-right" });
}

// ─── TYPES ───────────────────────────────────────────────────────
export type GamePhase =
    | "NotRunning" | "InLobby" | "ChampSelect"
    | "Loading" | "InGame" | "EndGame";

export interface PhaseChangeEvent {
    previous: GamePhase;
    current: GamePhase;
    timestamp: number;
}

export interface GameSession {
    allyTeam: SessionPlayer[];
    enemyTeam: SessionPlayer[];
    localPlayer: SessionPlayer;
}

export interface SessionPlayer {
    summonerName: string;
    puuid: string;
    championId: number;
    championName: string;
    position: string;       // "TOP" | "JUNGLE" | "MID" | "ADC" | "SUPPORT"
    summonerSpells: [number, number];
}
```

---

## Platform Integration — App Initialization (main.tsx)

```typescript
// frontend/src/main.tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initPlatform, onPhaseChange } from "./lib/platform";
import { useAppStore } from "./store/appStore";

async function bootstrap() {
    // 1. Load the LOLOverlay SDK (served by platform on localhost)
    await loadScript("http://localhost:45678/sdk/loloverlay.js");

    // 2. Initialize the app with the platform
    await initPlatform();

    // 3. Subscribe to game phase changes — update Zustand store
    onPhaseChange((event) => {
        useAppStore.getState().setGamePhase(event.current);
    });

    // 4. Mount React
    ReactDOM.createRoot(document.getElementById("root")!).render(
        <React.StrictMode><App /></React.StrictMode>
    );
}

function loadScript(src: string): Promise<void> {
    return new Promise((resolve) => {
        const s = document.createElement("script");
        s.src = src;
        s.onload = () => resolve();
        document.head.appendChild(s);
    });
}

bootstrap();
```

---

## Zustand Store (appStore.ts)

```typescript
// frontend/src/store/appStore.ts
import { create } from "zustand";
import type { GamePhase, GameSession } from "../lib/platform";

interface AppStore {
    // ─── Platform state ───────────────────────────────────────────
    gamePhase: GamePhase;
    gameSession: GameSession | null;

    // ─── Enemy profiles ───────────────────────────────────────────
    enemyProfiles: PlayerProfile[];
    enemyProfilesStatus: "idle" | "loading" | "ready" | "error";

    // ─── Win condition ────────────────────────────────────────────
    winCondition: WinConditionReport | null;
    winConditionStatus: "idle" | "loading" | "ready" | "error";

    // ─── Hunter focus ─────────────────────────────────────────────
    hunterTarget: PlayerProfile | null;
    hunterTargetIndex: number | null;   // 0–4, index into enemyProfiles
    hunterFocusStatus: "idle" | "loading" | "ready" | "error";
    hunterPanelVisible: boolean;

    // ─── Post game ────────────────────────────────────────────────
    postGameReview: PostGameReview | null;
    postGameStatus: "idle" | "loading" | "ready" | "error";

    // ─── Settings ─────────────────────────────────────────────────
    settings: AppSettings;

    // ─── Actions ──────────────────────────────────────────────────
    setGamePhase: (phase: GamePhase) => void;
    setGameSession: (session: GameSession) => void;
    setEnemyProfiles: (profiles: PlayerProfile[], status: AppStore["enemyProfilesStatus"]) => void;
    setWinCondition: (report: WinConditionReport | null, status: AppStore["winConditionStatus"]) => void;
    lockTarget: (index: number) => void;
    setHunterTarget: (profile: PlayerProfile | null, status: AppStore["hunterFocusStatus"]) => void;
    toggleHunterPanel: () => void;
    setPostGameReview: (review: PostGameReview | null, status: AppStore["postGameStatus"]) => void;
    updateSettings: (partial: Partial<AppSettings>) => void;
    resetForNewGame: () => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
    gamePhase: "NotRunning",
    gameSession: null,
    enemyProfiles: [],
    enemyProfilesStatus: "idle",
    winCondition: null,
    winConditionStatus: "idle",
    hunterTarget: null,
    hunterTargetIndex: null,
    hunterFocusStatus: "idle",
    hunterPanelVisible: false,
    postGameReview: null,
    postGameStatus: "idle",
    settings: DEFAULT_SETTINGS,

    setGamePhase: (phase) => {
        set({ gamePhase: phase });
        // Auto-reset state on new game cycle
        if (phase === "ChampSelect") get().resetForNewGame();
    },

    setGameSession: (session) => set({ gameSession: session }),

    setEnemyProfiles: (profiles, status) =>
        set({ enemyProfiles: profiles, enemyProfilesStatus: status }),

    setWinCondition: (report, status) =>
        set({ winCondition: report, winConditionStatus: status }),

    lockTarget: (index) => {
        const profiles = get().enemyProfiles;
        if (profiles[index]) {
            set({ hunterTargetIndex: index });
        }
    },

    setHunterTarget: (profile, status) =>
        set({ hunterTarget: profile, hunterFocusStatus: status }),

    toggleHunterPanel: () =>
        set((s) => ({ hunterPanelVisible: !s.hunterPanelVisible })),

    setPostGameReview: (review, status) =>
        set({ postGameReview: review, postGameStatus: status }),

    updateSettings: (partial) =>
        set((s) => ({ settings: { ...s.settings, ...partial } })),

    resetForNewGame: () => set({
        enemyProfiles: [],
        enemyProfilesStatus: "idle",
        winCondition: null,
        winConditionStatus: "idle",
        hunterTarget: null,
        hunterTargetIndex: null,
        hunterFocusStatus: "idle",
        hunterPanelVisible: false,
        postGameReview: null,
        postGameStatus: "idle",
    }),
}));
```

---

## App.tsx — Root Component and Hotkey Registration

```tsx
// frontend/src/App.tsx
import { useEffect } from "react";
import { useAppStore } from "./store/appStore";
import { registerHotkey, showPanel, hidePanel, getSession, toast } from "./lib/platform";
import { LoadingPanel } from "./panels/LoadingPanel/LoadingPanel";
import { WinConditionPanel } from "./panels/WinCondition/WinConditionPanel";
import { HunterFocusPanel } from "./panels/HunterFocus/HunterFocusPanel";
import { PostGamePanel } from "./panels/PostGame/PostGamePanel";
import { useEnemyProfiles } from "./hooks/useEnemyProfiles";
import { useWinCondition } from "./hooks/useWinCondition";
import { useHunterTarget } from "./hooks/useHunterTarget";
import { usePostGame } from "./hooks/usePostGame";

export default function App() {
    const { gamePhase, hunterPanelVisible, toggleHunterPanel, lockTarget } = useAppStore();

    // Data hooks — each watches game phase and fetches at the right moment
    useEnemyProfiles();
    useWinCondition();
    useHunterTarget();
    usePostGame();

    // Register all hotkeys with LOLOverlay platform on mount
    useEffect(() => {
        registerHotkey("toggle-win-condition", "Alt+W", () => {
            const store = useAppStore.getState();
            if (store.gamePhase === "InGame" || store.gamePhase === "Loading") {
                // Platform handles show/hide — we just call it
                const visible = store.winCondition !== null;
                visible ? hidePanel("win-condition-panel") : showPanel("win-condition-panel");
            }
        });

        registerHotkey("toggle-hunter-focus", "Alt+H", () => {
            toggleHunterPanel();
            const panelId = "hunter-focus-panel";
            useAppStore.getState().hunterPanelVisible
                ? hidePanel(panelId)
                : showPanel(panelId);
        });

        registerHotkey("dismiss-post-game", "Escape", () => {
            hidePanel("post-game-panel");
        });

        // Alt+1 through Alt+5 lock hunter target
        [0, 1, 2, 3, 4].forEach((i) => {
            registerHotkey(`lock-target-${i + 1}`, `Alt+${i + 1}`, () => {
                lockTarget(i);
                toast(`Hunter target locked: enemy ${i + 1}`, "info");
            });
        });
    }, []);

    // Determine which panel to render based on URL query param
    // LOLOverlay creates separate webview instances per window entry
    // Each window URL has ?panel=xxx — React reads it and renders only that panel
    const params = new URLSearchParams(window.location.search);
    const panel = params.get("panel");

    if (panel === "loading") return <LoadingPanel />;
    if (panel === "wincondition") return <WinConditionPanel />;
    if (panel === "hunterfocus") return <HunterFocusPanel />;
    if (panel === "postgame") return <PostGamePanel />;

    return null; // No panel matched
}
```

---

## Data Hooks

### useEnemyProfiles.ts
```typescript
// Watches for game phase = Loading
// When triggered: fetches game session from platform, calls backend /enemies
// Populates enemyProfiles in Zustand store

export function useEnemyProfiles() {
    const { gamePhase, setGameSession, setEnemyProfiles } = useAppStore();

    useEffect(() => {
        if (gamePhase !== "Loading") return;

        async function fetch() {
            setEnemyProfiles([], "loading");
            try {
                // 1. Get session data from LOLOverlay platform
                const session = await getSession();
                setGameSession(session);

                // 2. Call Hunter Mode backend with enemy summoner names + champion IDs
                const res = await axios.get("http://localhost:45679/enemies", {
                    params: {
                        names: session.enemyTeam.map(p => p.summonerName).join(","),
                        championIds: session.enemyTeam.map(p => p.championId).join(",")
                    }
                });
                setEnemyProfiles(res.data.profiles, "ready");
            } catch (e) {
                setEnemyProfiles([], "error");
                toast("Could not load enemy profiles", "warning");
            }
        }

        fetch();
    }, [gamePhase]);
}
```

### useWinCondition.ts
```typescript
// Watches for enemyProfilesStatus = "ready" (profiles loaded)
// When triggered: calls backend /win-condition with both team compositions
// Populates winCondition in Zustand store

export function useWinCondition() {
    const { enemyProfilesStatus, gameSession, setWinCondition } = useAppStore();

    useEffect(() => {
        if (enemyProfilesStatus !== "ready" || !gameSession) return;

        async function fetch() {
            setWinCondition(null, "loading");
            try {
                const res = await axios.get("http://localhost:45679/win-condition", {
                    params: {
                        allyChamions: gameSession.allyTeam.map(p => p.championId).join(","),
                        enemyChampions: gameSession.enemyTeam.map(p => p.championId).join(",")
                    }
                });
                setWinCondition(res.data.report, "ready");
            } catch (e) {
                setWinCondition(null, "error");
            }
        }

        fetch();
    }, [enemyProfilesStatus]);
}
```

### useHunterTarget.ts
```typescript
// Watches for hunterTargetIndex changes
// When index changes: calls backend /hunter/:summonerName/:championId
// Populates hunterTarget in Zustand store

export function useHunterTarget() {
    const { hunterTargetIndex, enemyProfiles, gameSession, setHunterTarget } = useAppStore();

    useEffect(() => {
        if (hunterTargetIndex === null || !gameSession) return;

        const targetPlayer = gameSession.enemyTeam[hunterTargetIndex];
        if (!targetPlayer) return;

        async function fetch() {
            setHunterTarget(null, "loading");
            try {
                const res = await axios.get(
                    `http://localhost:45679/hunter/${targetPlayer.summonerName}/${targetPlayer.championId}`
                );
                setHunterTarget(res.data.profile, "ready");
            } catch (e) {
                setHunterTarget(null, "error");
                toast("Could not load hunter target data", "warning");
            }
        }

        fetch();
    }, [hunterTargetIndex]);
}
```

### usePostGame.ts
```typescript
// Watches for game phase = EndGame
// When triggered: calls backend /post-game with last match ID + local player puuid

export function usePostGame() {
    const { gamePhase, gameSession, setPostGameReview } = useAppStore();

    useEffect(() => {
        if (gamePhase !== "EndGame" || !gameSession) return;

        async function fetch() {
            setPostGameReview(null, "loading");
            try {
                // Get last match ID for local player via backend
                const res = await axios.get(
                    `http://localhost:45679/post-game/latest/${gameSession.localPlayer.puuid}`
                );
                setPostGameReview(res.data.review, "ready");
                showPanel("post-game-panel");
            } catch (e) {
                setPostGameReview(null, "error");
            }
        }

        fetch();
    }, [gamePhase]);
}
```

---

## Backend — Node.js Express Server (server.js)

```javascript
// backend/server.js
const express = require("express");
const app = express();
app.use(express.json());

const enemyRoutes = require("./routes/enemies");
const winConditionRoutes = require("./routes/wincondition");
const hunterRoutes = require("./routes/hunter");
const postGameRoutes = require("./routes/postgame");
const webhookRoutes = require("./routes/webhook");

app.use("/enemies", enemyRoutes);
app.use("/win-condition", winConditionRoutes);
app.use("/hunter", hunterRoutes);
app.use("/post-game", postGameRoutes);
app.use("/webhook", webhookRoutes);

app.listen(45679, "127.0.0.1", () => {
    console.log("[Hunter Mode Backend] Running on port 45679");
});
```

---

## Backend — Riot API Service (services/riotApi.js)

```javascript
// backend/services/riotApi.js
const axios = require("axios");
const cache = require("./cache");

const API_KEY = process.env.HUNTER_MODE_RIOT_API_KEY; // Never hardcoded
const REGIONS = {
    na1: { platform: "na1", regional: "americas" },
    euw1: { platform: "euw1", regional: "europe" },
    kr: { platform: "kr", regional: "asia" },
    // ... all regions
};

// Token bucket rate limiter — respects Riot's 20/1s and 100/120s limits
class RateLimiter {
    constructor() {
        this.shortQueue = [];  // 20 per 1s
        this.longQueue = [];   // 100 per 120s
    }
    async throttle() { /* ... */ }
}

const limiter = new RateLimiter();

async function riotGet(url, cacheKey, ttlMinutes = 10) {
    // 1. Check cache first
    const cached = cache.get(cacheKey);
    if (cached) return cached;

    // 2. Rate limit
    await limiter.throttle();

    // 3. Make request
    const res = await axios.get(url, {
        headers: { "X-Riot-Token": API_KEY }
    });

    // 4. Cache result
    cache.set(cacheKey, res.data, ttlMinutes);
    return res.data;
}

module.exports = {
    // Summoner v4
    async getSummonerByName(name, region) {
        const { platform } = REGIONS[region];
        return riotGet(
            `https://${platform}.api.riotgames.com/lol/summoner/v4/summoners/by-name/${encodeURIComponent(name)}`,
            `summoner:${region}:${name}`
        );
    },

    // Match v5 — get match IDs
    async getMatchIds(puuid, region, count = 10) {
        const { regional } = REGIONS[region];
        return riotGet(
            `https://${regional}.api.riotgames.com/lol/match/v5/matches/by-puuid/${puuid}/ids?count=${count}`,
            `match-ids:${puuid}:${count}`,
            5  // shorter TTL — match list changes often
        );
    },

    // Match v5 — full match data
    async getMatch(matchId, region) {
        const { regional } = REGIONS[region];
        return riotGet(
            `https://${regional}.api.riotgames.com/lol/match/v5/matches/${matchId}`,
            `match:${matchId}`,
            60  // longer TTL — past matches never change
        );
    },

    // Champion Mastery v4
    async getTopMasteries(summonerId, region, count = 5) {
        const { platform } = REGIONS[region];
        return riotGet(
            `https://${platform}.api.riotgames.com/lol/champion-mastery/v4/champion-masteries/by-summoner/${summonerId}/top?count=${count}`,
            `mastery:${summonerId}`
        );
    },

    // League v4
    async getLeagueEntries(summonerId, region) {
        const { platform } = REGIONS[region];
        return riotGet(
            `https://${platform}.api.riotgames.com/lol/league/v4/entries/by-summoner/${summonerId}`,
            `league:${summonerId}`
        );
    }
};
```

---

## Backend — Player Profile Analyzer (services/playerProfile.js)

```javascript
// backend/services/playerProfile.js

// Given a summoner + champion ID, fetch their last N games on that champion
// and derive a full PlayerProfile object

async function buildPlayerProfile(summonerName, championId, region, matchCount = 10) {
    const riot = require("./riotApi");

    // 1. Get summoner base data
    const summoner = await riot.getSummonerByName(summonerName, region);

    // 2. Get rank
    const leagueEntries = await riot.getLeagueEntries(summoner.id, region);
    const soloEntry = leagueEntries.find(e => e.queueType === "RANKED_SOLO_5x5");
    const rank = soloEntry ? `${soloEntry.tier} ${soloEntry.rank}` : "Unranked";

    // 3. Get mastery for this champion
    const masteries = await riot.getTopMasteries(summoner.id, region);
    const mastery = masteries.find(m => m.championId === championId) || { masteryLevel: 0, championPoints: 0 };

    // 4. Get last matchCount matches on this champion
    const allMatchIds = await riot.getMatchIds(summoner.puuid, region, 50);
    const championMatches = [];
    for (const matchId of allMatchIds) {
        if (championMatches.length >= matchCount) break;
        const match = await riot.getMatch(matchId, region);
        const participant = match.info.participants.find(p => p.puuid === summoner.puuid);
        if (participant && participant.championId === championId) {
            championMatches.push(participant);
        }
    }

    if (championMatches.length === 0) {
        return buildFallbackProfile(summoner, champion, rank, mastery);
    }

    // 5. Compute stats from matches
    const wins = championMatches.filter(p => p.win).length;
    const winRate = wins / championMatches.length;

    const avgKda = championMatches.reduce((sum, p) => {
        const kda = p.deaths === 0
            ? (p.kills + p.assists)
            : (p.kills + p.assists) / p.deaths;
        return sum + kda;
    }, 0) / championMatches.length;

    const avgCsPerMin = championMatches.reduce((sum, p) => {
        const mins = p.timePlayed / 60;
        return sum + (p.totalMinionsKilled + p.neutralMinionsKilled) / mins;
    }, 0) / championMatches.length;

    const avgVisionScore = championMatches.reduce((sum, p) =>
        sum + p.visionScore, 0) / championMatches.length;

    const avgGameDuration = championMatches.reduce((sum, p) =>
        sum + p.timePlayed / 60, 0) / championMatches.length;

    // 6. Derive most common build (items appearing most across games)
    const itemFrequency = {};
    championMatches.forEach(p => {
        [p.item0, p.item1, p.item2, p.item3, p.item4, p.item5]
            .filter(id => id > 0)
            .forEach(id => { itemFrequency[id] = (itemFrequency[id] || 0) + 1; });
    });
    const mostCommonBuild = Object.entries(itemFrequency)
        .sort(([,a], [,b]) => b - a)
        .slice(0, 6)
        .map(([id]) => parseInt(id));

    // 7. Derive summoner spells
    const spellFrequency = {};
    championMatches.forEach(p => {
        const key = `${p.summoner1Id}:${p.summoner2Id}`;
        spellFrequency[key] = (spellFrequency[key] || 0) + 1;
    });
    const topSpells = Object.entries(spellFrequency).sort(([,a],[,b]) => b-a)[0][0].split(":");
    const mostCommonSpells = [parseInt(topSpells[0]), parseInt(topSpells[1])];

    // 8. Playstyle tag — derived from early game stats
    const avgEarlyKills = championMatches.reduce((sum, p) => sum + p.kills, 0) / championMatches.length;
    const avgEarlyDeaths = championMatches.reduce((sum, p) => sum + p.deaths, 0) / championMatches.length;
    let playstyleTag = "Balanced";
    if (avgEarlyKills > 5) playstyleTag = "AggressiveEarly";
    else if (avgCsPerMin > 8) playstyleTag = "FarmFocused";
    else if (avgVisionScore > 35) playstyleTag = "RoamHeavy";

    // 9. Recent form
    const last5 = championMatches.slice(0, 5).map(p => p.win);
    const recentWins = last5.filter(Boolean).length;
    let recentForm = "Mixed";
    if (last5.every(Boolean)) recentForm = `WinStreak(${last5.length})`;
    else if (!last5.some(Boolean)) recentForm = `LossStreak(${last5.length})`;

    // 10. Threat level (1–5)
    const tierWeights = { CHALLENGER: 5, GRANDMASTER: 5, MASTER: 4, DIAMOND: 4, EMERALD: 3, PLATINUM: 3, GOLD: 2, SILVER: 1, BRONZE: 1, IRON: 1 };
    const rankThreat = tierWeights[soloEntry?.tier] || 1;
    const formThreat = recentWins >= 4 ? 2 : recentWins >= 2 ? 1 : 0;
    const masteryThreat = mastery.masteryLevel >= 6 ? 1 : 0;
    const threatLevel = Math.min(5, rankThreat + formThreat + masteryThreat);

    return {
        summonerName,
        puuid: summoner.puuid,
        rank,
        championId,
        masteryLevel: mastery.masteryLevel,
        masteryPoints: mastery.championPoints,
        winRate,
        avgKda,
        avgCsPerMin,
        avgVisionScore,
        avgGameDuration,
        mostCommonBuild,
        mostCommonSpells,
        playstyleTag,
        recentForm,
        threatLevel,
        gamesAnalyzed: championMatches.length,
    };
}

module.exports = { buildPlayerProfile };
```

---

## Backend — Win Condition Analyzer (services/winConditionAnalyzer.js)

```javascript
// backend/services/winConditionAnalyzer.js
// Uses Riot Data Dragon champion tags to analyze team compositions

const CHAMPION_ROLE_WEIGHTS = {
    Tank:       { earlyGame: 1, midGame: 2, lateGame: 3, teamfight: 3, poke: 0, siege: 1, splitpush: 1 },
    Fighter:    { earlyGame: 3, midGame: 3, lateGame: 2, teamfight: 2, poke: 0, siege: 1, splitpush: 3 },
    Mage:       { earlyGame: 1, midGame: 3, lateGame: 3, teamfight: 3, poke: 2, siege: 2, splitpush: 0 },
    Assassin:   { earlyGame: 3, midGame: 3, lateGame: 2, teamfight: 1, poke: 0, siege: 0, splitpush: 2 },
    Marksman:   { earlyGame: 1, midGame: 2, lateGame: 3, teamfight: 3, poke: 2, siege: 3, splitpush: 1 },
    Support:    { earlyGame: 1, midGame: 2, lateGame: 2, teamfight: 2, poke: 1, siege: 1, splitpush: 0 },
};

function scoreTeam(championIds, dragonData) {
    const scores = { earlyGame: 0, midGame: 0, lateGame: 0, teamfight: 0, poke: 0, siege: 0, splitpush: 0 };
    championIds.forEach(id => {
        const champ = dragonData[id];
        if (!champ) return;
        const tag = champ.tags[0]; // primary tag
        const weights = CHAMPION_ROLE_WEIGHTS[tag] || CHAMPION_ROLE_WEIGHTS.Fighter;
        Object.keys(scores).forEach(k => { scores[k] += weights[k]; });
    });
    return scores;
}

function powerLevel(score) {
    if (score >= 12) return "High";
    if (score >= 7) return "Medium";
    return "Low";
}

function deriveWinConditions(scores, isAlly) {
    const conditions = [];
    if (scores.teamfight >= 12) conditions.push("Force teamfights — your composition excels in 5v5 engagements");
    if (scores.poke >= 8) conditions.push("Win through sustained poke before committing to fights");
    if (scores.siege >= 10) conditions.push("Siege objectives — your ranged pressure makes towers vulnerable");
    if (scores.splitpush >= 10) conditions.push("Apply split pressure — force map responses with side-lane threats");
    if (scores.earlyGame >= 12) conditions.push("Snowball through early game — win before late-game scaling kicks in");
    if (scores.lateGame >= 12) conditions.push("Scale to late game — survive early and win extended fights");
    if (conditions.length === 0) conditions.push("Well-rounded composition — adapt to game state");
    return conditions.slice(0, 3);
}

function generateDraftTips(allyScores, enemyScores) {
    const tips = [];
    if (enemyScores.teamfight > allyScores.teamfight + 5)
        tips.push("Avoid grouped teamfights — split the map instead");
    if (enemyScores.earlyGame > allyScores.earlyGame + 4)
        tips.push("Play safe early — your team scales stronger");
    if (allyScores.siege > enemyScores.siege + 4)
        tips.push("Prioritize tower pressure over kills");
    return tips.slice(0, 2);
}

function analyzeCompositions(allyChampionIds, enemyChampionIds, dragonData) {
    const allyScores = scoreTeam(allyChampionIds, dragonData);
    const enemyScores = scoreTeam(enemyChampionIds, dragonData);

    const allyTotal = Object.values(allyScores).reduce((a, b) => a + b, 0);
    const enemyTotal = Object.values(enemyScores).reduce((a, b) => a + b, 0);
    const diff = allyTotal - enemyTotal;

    return {
        allyWinConditions: deriveWinConditions(allyScores, true),
        enemyWinConditions: deriveWinConditions(enemyScores, false),
        allyPowerTimeline: {
            earlyGame: powerLevel(allyScores.earlyGame),
            midGame: powerLevel(allyScores.midGame),
            lateGame: powerLevel(allyScores.lateGame),
        },
        enemyPowerTimeline: {
            earlyGame: powerLevel(enemyScores.earlyGame),
            midGame: powerLevel(enemyScores.midGame),
            lateGame: powerLevel(enemyScores.lateGame),
        },
        favorableRating: diff > 5 ? "Favorable" : diff < -5 ? "Unfavorable" : "Even",
        draftTips: generateDraftTips(allyScores, enemyScores),
    };
}

module.exports = { analyzeCompositions };
```

---

## Backend — Webhook Route (routes/webhook.js)

```javascript
// backend/routes/webhook.js
// Receives events pushed by LOLOverlay platform
// This allows the backend to react to game phase changes and pre-fetch data

const express = require("express");
const router = express.Router();

router.post("/", async (req, res) => {
    const { event, data } = req.body;

    if (event === "game_phase_changed") {
        const { current } = data;
        // Pre-warm cache: if entering Loading phase, the frontend
        // will imminently request enemy profiles — we can start fetching now
        if (current === "Loading") {
            console.log("[Hunter Mode Backend] Loading phase detected — ready for profile requests");
        }
        if (current === "EndGame") {
            console.log("[Hunter Mode Backend] EndGame detected — ready for post-game requests");
        }
    }

    if (event === "game_session_ready") {
        // Session data available — optionally pre-fetch all enemy profiles in background
        const { enemyTeam } = data;
        console.log(`[Hunter Mode Backend] Session ready — ${enemyTeam.length} enemies detected`);
    }

    res.json({ ok: true });
});

module.exports = router;
```

---

## Panel Specifications

### LoadingPanel
- **Renders when**: `gamePhase === "Loading"`
- **Data dependency**: `enemyProfiles` from Zustand store
- **Auto-hides**: 8 seconds after `gamePhase` changes to `InGame`
- **Layout**: Vertical list of 5 `EnemyRow` components
- **EnemyRow (collapsed)**: Champion icon (32px) | Summoner name | Rank badge | Win rate pill (color-coded) | KDA (3-number format)
- **EnemyRow (expanded on click)**: All collapsed fields + avg CS/min | vision score | most common build (item icons 24px each) | summoner spells | playstyle tag | recent form | threat meter
- **Loading state**: Each row shows skeleton placeholder while `enemyProfilesStatus === "loading"`
- **Error state**: Row shows "Profile unavailable" in muted text

### WinConditionPanel
- **Renders when**: `gamePhase === "Loading"` or `gamePhase === "InGame"`
- **Data dependency**: `winCondition` from Zustand store
- **Toggle**: `Alt+W` or platform hotkey
- **Layout**:
  - Header: "Win Conditions" label + favorable rating badge (blue/gray/orange)
  - Section "Your Team": bullet list of 2–3 win condition strings
  - Section "Enemy Team": bullet list of 2–3 win condition strings
  - PowerTimeline: Two side-by-side rows of Early/Mid/Late labels with color-coded dots (green=High, amber=Medium, gray=Low) for ally and enemy
  - KeyThreats: Up to 2 champion icons with "Priority target" label beneath
  - DraftTips: 1–2 italic coaching tip strings in a callout box

### HunterFocusPanel
- **Renders when**: `gamePhase === "InGame"` AND `hunterPanelVisible === true`
- **Data dependency**: `hunterTarget` from Zustand store
- **Toggle**: `Alt+H` hotkey or LOLOverlay platform hotkey event
- **Target lock**: `Alt+1` through `Alt+5` — each updates `hunterTargetIndex` in store
- **Layout**:
  - TargetHeader: Champion icon (48px) | summoner name | rank | mastery level (badge) | threat level (1–5 dot indicator, color-coded)
  - WinRate bar: horizontal fill bar, label "Win rate on this champion: X%"
  - Playstyle tag chip: e.g. "Aggressive Early" in colored pill
  - RecentForm chip: e.g. "Win streak (3)" in green or "Loss streak (2)" in red
  - AbilityTendency section: skill max order displayed as Q/W/E icons in order (1st, 2nd, 3rd)
  - PositionalTendency section: roam tendency tag + objective priority tag
  - CounterTips section: 2–3 bullet tips in muted italic text
  - Loading state: target header shows spinner while `hunterFocusStatus === "loading"`
  - No target state: shows "Press Alt+1–5 to lock a target" instruction text
  - Switching targets: instant — old data replaced by spinner then new data

### PostGamePanel
- **Renders when**: `gamePhase === "EndGame"` AND `postGameStatus === "ready"`
- **Data dependency**: `postGameReview` from Zustand store
- **Dismiss**: `Escape` hotkey calls `hidePanel("post-game-panel")`
- **Layout**:
  - Header: "Game Review" title + match result badge (Victory/Defeat)
  - YourStats section: 3-column table (Stat | Your Value | Your Average | Rank Average) for KDA, CS/min, Vision Score, Damage
  - Delta indicators: green arrow up / red arrow down next to each value showing deviation from average
  - Strength callout: green box with "Best this game: [stat name]"
  - Improvement callout: amber box with "Focus area: [stat name]"
  - HunterOutcome section (only shown if a target was locked): "You tracked [champion name]" header + their final KDA + whether their performance was above or below their historical average
  - Dismiss prompt: small text at bottom "Press Escape to close"

---

## Visual Design Tokens

```css
/* frontend/src/tokens.css */

/* Base panel appearance */
--hm-panel-bg: rgba(8, 8, 14, 0.88);
--hm-panel-border: 1px solid rgba(255, 255, 255, 0.08);
--hm-panel-radius: 10px;
--hm-panel-blur: backdrop-filter: blur(10px);
--hm-panel-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);

/* Typography */
--hm-font: "Inter", "Segoe UI", sans-serif;
--hm-text-primary: rgba(255, 255, 255, 0.92);
--hm-text-secondary: rgba(255, 255, 255, 0.55);
--hm-text-muted: rgba(255, 255, 255, 0.30);
--hm-text-xs: 11px;
--hm-text-sm: 12px;
--hm-text-base: 13px;
--hm-text-md: 14px;
--hm-text-lg: 16px;

/* Threat colors */
--hm-threat-1: #4CAF50;  /* Low */
--hm-threat-2: #8BC34A;
--hm-threat-3: #FF9800;  /* Medium */
--hm-threat-4: #FF5722;
--hm-threat-5: #F44336;  /* High */

/* Outcome colors */
--hm-favorable: #4FC3F7;
--hm-even: #90A4AE;
--hm-unfavorable: #FF7043;

/* Power level colors */
--hm-power-high: #66BB6A;
--hm-power-medium: #FFA726;
--hm-power-low: #546E7A;

/* Win/Loss */
--hm-win: #4CAF50;
--hm-loss: #EF5350;

/* Animations */
--hm-fade-in: opacity 0.15s ease, transform 0.15s ease;
```

All panels use these tokens. Panels mount with `opacity: 0; transform: translateY(-4px)` and transition to `opacity: 1; transform: translateY(0)` on mount. No bouncing, no heavy transforms — respects game performance.

---

## Settings Schema (stored via LOLOverlay.storage)

```typescript
interface AppSettings {
    riotApiKey: string;
    region: "na1" | "euw1" | "kr" | "eun1" | "tr1" | "br1" | "la1" | "la2" | "oc1" | "ru";
    tier: "free" | "premium";   // Controls feature access
    matchHistoryDepth: 10 | 50; // free = 10, premium = 50
    panels: {
        loadingPanel: { enabled: boolean; opacity: number };
        winCondition: { enabled: boolean; opacity: number };
        hunterFocus: { enabled: boolean; opacity: number };
        postGame: { enabled: boolean; opacity: number };
    };
    colorblindMode: boolean;
}

const DEFAULT_SETTINGS: AppSettings = {
    riotApiKey: "",
    region: "na1",
    tier: "free",
    matchHistoryDepth: 10,
    panels: {
        loadingPanel: { enabled: true, opacity: 0.92 },
        winCondition: { enabled: true, opacity: 0.92 },
        hunterFocus: { enabled: true, opacity: 0.92 },
        postGame: { enabled: true, opacity: 0.96 },
    },
    colorblindMode: false,
};
```

Settings are loaded on startup via `LOLOverlay.storage.get("settings")` and saved on any change via `LOLOverlay.storage.set("settings", newSettings)`. The platform persists this between sessions automatically.

---

## First-Run Setup Flow

On first launch (when `riotApiKey === ""`), Hunter Mode shows a setup screen instead of any overlay panels:

1. Step 1 — Welcome screen: brief description of Hunter Mode, "Get Started" button
2. Step 2 — API Key: input field, link to Riot Developer Portal, validation button (calls backend to test the key)
3. Step 3 — Region: dropdown selector
4. Step 4 — Ready: "Hunter Mode is ready. Launch a League of Legends game to begin."

Setup screen is rendered as an interactive widget window (`clickThrough: false`) centered on screen.

---

## What Hunter Mode Is Responsible For — Summary

| Responsibility | Hunter Mode |
|---|---|
| Riot API calls | YES — backend/services/riotApi.js |
| Player profile analysis | YES — backend/services/playerProfile.js |
| Win condition analysis | YES — backend/services/winConditionAnalyzer.js |
| Post-game performance comparison | YES — backend/services/postGameAnalyzer.js |
| React UI panels | YES — frontend/src/panels/ |
| Zustand state management | YES — frontend/src/store/appStore.ts |
| Platform SDK integration | YES — frontend/src/lib/platform.ts |
| API response caching | YES — backend/services/cache.js (LowDB) |
| Settings persistence | YES — via LOLOverlay.storage |
| Overlay window creation | NO — LOLOverlay manifest |
| Drag / resize / position persistence | NO — LOLOverlay platform |
| Game phase detection | NO — LOLOverlay platform |
| Hotkey system | NO — LOLOverlay platform (registered via SDK) |
| DirectX hook | NO — LOLOverlay platform |
| App auto-update | NO — LOLOverlay platform |
| Distribution / installation | NO — LOLOverlay app store |

---

## Deliverables

1. `manifest.json` — fully configured for all 4 windows, 8 hotkeys, 2 webhooks, backend process
2. `frontend/` — full React + TypeScript + Vite project with all 4 panels implemented
3. `backend/` — full Node.js + Express project with all routes and services implemented
4. `frontend/src/lib/platform.ts` — complete LOLOverlay SDK wrapper
5. First-run setup flow (API key + region input)
6. Settings panel accessible from platform UI
7. README — setup instructions, environment variable config (`HUNTER_MODE_RIOT_API_KEY`), build steps, dev mode instructions (`loloverlay dev`)
