# Task 08: Frontend - Platform Integration

**App: Hunter Mode**  
**Dependencies:** Task 01  
**Estimated Complexity:** Medium  
**Priority:** P0 (Frontend foundation)

---

## Objective

Create the LOLOverlay platform integration layer — the single source of truth for all platform SDK interactions. This includes the platform SDK wrapper (`platform.ts`), app initialization (`main.tsx`), root component (`App.tsx`), and Zustand global state store (`appStore.ts`).

---

## Context

Hunter Mode NEVER directly calls `LOLOverlay.*` APIs. All platform integration happens through `frontend/src/lib/platform.ts`. This makes the codebase:

- Testable (easy to mock platform calls)
- Maintainable (one file to update if SDK changes)
- Clear (explicit platform boundaries)

This task creates:

1. Platform SDK wrapper with all LOLOverlay methods
2. App bootstrap logic (SDK loading + initialization)
3. Root component that routes to panels based on URL param
4. Zustand store for all global state

---

## What You Need to Build

### 1. Platform SDK Wrapper (`frontend/src/lib/platform.ts`)

```typescript
// frontend/src/lib/platform.ts

declare const LOLOverlay: any;

/**
 * Initialize the LOLOverlay SDK
 */
export async function initPlatform(): Promise<void> {
  await LOLOverlay.init({
    appId: "hunter-mode",
    version: "1.0.0",
  });
}

// ─── GAME ────────────────────────────────────────────────────────

export type GamePhase =
  | "NotRunning"
  | "InLobby"
  | "ChampSelect"
  | "Loading"
  | "InGame"
  | "EndGame";

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
  position: string; // "TOP" | "JUNGLE" | "MID" | "ADC" | "SUPPORT"
  summonerSpells: [number, number];
}

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

export async function setPanelOpacity(
  id: string,
  opacity: number,
): Promise<void> {
  return LOLOverlay.windows.setOpacity(id, opacity);
}

// ─── HOTKEYS ─────────────────────────────────────────────────────

export interface HotkeyConfig {
  id: string;
  keys: string;
  onPress: () => void;
}

export function registerHotkey(id: string, keys: string, cb: () => void): void {
  LOLOverlay.hotkeys.register({ id, keys, onPress: cb });
}

// ─── STORAGE ─────────────────────────────────────────────────────

export async function saveToStorage(
  key: string,
  value: unknown,
): Promise<void> {
  return LOLOverlay.storage.set(key, value);
}

export async function loadFromStorage<T>(key: string): Promise<T | null> {
  return LOLOverlay.storage.get(key);
}

// ─── NOTIFICATIONS ───────────────────────────────────────────────

export type ToastType = "info" | "success" | "warning" | "error";

export function toast(message: string, type: ToastType = "info"): void {
  LOLOverlay.notify.toast({
    message,
    type,
    duration: 3000,
    position: "top-right",
  });
}
```

### 2. App Initialization (`frontend/src/main.tsx`)

```tsx
// frontend/src/main.tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initPlatform, onPhaseChange } from "./lib/platform";
import { useAppStore } from "./store/appStore";
import "./index.css";

async function bootstrap() {
  try {
    // 1. Load LOLOverlay SDK from platform
    await loadScript("http://localhost:45678/sdk/loloverlay.js");

    // 2. Initialize platform connection
    await initPlatform();

    // 3. Subscribe to game phase changes
    onPhaseChange((event) => {
      useAppStore.getState().setGamePhase(event.current);
    });

    // 4. Mount React app
    ReactDOM.createRoot(document.getElementById("root")!).render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );

    console.log("[Hunter Mode] Initialized successfully");
  } catch (error) {
    console.error("[Hunter Mode] Initialization failed:", error);
    // Show error UI
    document.body.innerHTML = `
      <div style="color: white; padding: 20px; font-family: sans-serif;">
        <h1>Hunter Mode Failed to Load</h1>
        <p>Error: ${error}</p>
        <p>Please restart LOLOverlay.</p>
      </div>
    `;
  }
}

function loadScript(src: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = src;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error(`Failed to load script: ${src}`));
    document.head.appendChild(script);
  });
}

bootstrap();
```

### 3. Root Component (`frontend/src/App.tsx`)

```tsx
// frontend/src/App.tsx
import { useEffect } from "react";
import { useAppStore } from "./store/appStore";
import { registerHotkey, showPanel, hidePanel, toast } from "./lib/platform";

// Panel components (will be created in later tasks)
import { LoadingPanel } from "./panels/LoadingPanel/LoadingPanel";
import { WinConditionPanel } from "./panels/WinCondition/WinConditionPanel";
import { HunterFocusPanel } from "./panels/HunterFocus/HunterFocusPanel";
import { PostGamePanel } from "./panels/PostGame/PostGamePanel";

// Hooks (will be created in later tasks)
import { useEnemyProfiles } from "./hooks/useEnemyProfiles";
import { useWinCondition } from "./hooks/useWinCondition";
import { useHunterTarget } from "./hooks/useHunterTarget";
import { usePostGame } from "./hooks/usePostGame";

export default function App() {
  const { toggleHunterPanel, lockTarget } = useAppStore();

  // Data hooks — each watches game phase and fetches at the right moment
  useEnemyProfiles();
  useWinCondition();
  useHunterTarget();
  usePostGame();

  // Register hotkeys on mount
  useEffect(() => {
    // Toggle Win Condition panel (Alt+W)
    registerHotkey("toggle-win-condition", "Alt+W", () => {
      const visible = useAppStore.getState().winCondition !== null;
      visible
        ? hidePanel("win-condition-panel")
        : showPanel("win-condition-panel");
    });

    // Toggle Hunter Focus panel (Alt+H)
    registerHotkey("toggle-hunter-focus", "Alt+H", () => {
      toggleHunterPanel();
      const panelVisible = useAppStore.getState().hunterPanelVisible;
      panelVisible
        ? showPanel("hunter-focus-panel")
        : hidePanel("hunter-focus-panel");
    });

    // Dismiss post-game panel (Escape)
    registerHotkey("dismiss-post-game", "Escape", () => {
      hidePanel("post-game-panel");
    });

    // Lock Hunter targets (Alt+1 through Alt+5)
    for (let i = 0; i < 5; i++) {
      registerHotkey(`lock-target-${i + 1}`, `Alt+${i + 1}`, () => {
        lockTarget(i);
        toast(`Hunter target locked: enemy ${i + 1}`, "info");
      });
    }
  }, []);

  // Determine which panel to render based on URL query param
  // LOLOverlay creates separate webview instances per window entry
  const params = new URLSearchParams(window.location.search);
  const panel = params.get("panel");

  if (panel === "loading") return <LoadingPanel />;
  if (panel === "wincondition") return <WinConditionPanel />;
  if (panel === "hunterfocus") return <HunterFocusPanel />;
  if (panel === "postgame") return <PostGamePanel />;

  // No panel matched — should not happen in production
  return null;
}
```

### 4. Zustand Global Store (`frontend/src/store/appStore.ts`)

```typescript
// frontend/src/store/appStore.ts
import { create } from "zustand";
import type { GamePhase, GameSession } from "../lib/platform";

// ─── Types ───────────────────────────────────────────────────────

export interface PlayerProfile {
  summonerName: string;
  puuid: string | null;
  rank: string;
  championId: number;
  masteryLevel: number;
  masteryPoints: number;
  gamesPlayed: number;
  winRate: number;
  avgKda: number;
  avgCsPerMin: number;
  avgVisionScore: number;
  avgGameDurationMins: number;
  mostCommonBuild: number[];
  mostCommonSummonerSpells: [number, number];
  playstyleTag: string;
  recentForm: string;
  threatLevel: number;
  error?: boolean;
  errorMessage?: string;
}

export interface WinConditionReport {
  allyWinConditions: string[];
  enemyWinConditions: string[];
  allyPowerTimeline: {
    earlyGame: "Low" | "Medium" | "High";
    midGame: "Low" | "Medium" | "High";
    lateGame: "Low" | "Medium" | "High";
  };
  enemyPowerTimeline: {
    earlyGame: "Low" | "Medium" | "High";
    midGame: "Low" | "Medium" | "High";
    lateGame: "Low" | "Medium" | "High";
  };
  keyThreats: Array<{ championId: number; championName: string }>;
  draftTips: string[];
  favorableRating: "Favorable" | "Even" | "Unfavorable";
}

export interface HunterProfile extends PlayerProfile {
  abilityTendencies: {
    skillMaxOrder: string[];
    preferredCombo: string;
    ultimateUsagePattern: string;
  };
  positionalTendencies: {
    roamingFrequency: "Low" | "Medium" | "High";
    objectivePriority: string;
    wardingPattern: string;
  };
  counterTips: string[];
  recommendedCounterItems: number[];
}

export interface PostGameReview {
  matchId: string;
  matchResult: "Victory" | "Defeat";
  championId: number;
  championName: string;
  gameDurationMinutes: number;
  currentStats: {
    kda: number;
    kills: number;
    deaths: number;
    assists: number;
    csPerMin: number;
    visionScore: number;
    damageDealt: number;
    goldEarned: number;
  };
  historicalAvg: {
    kda: number;
    csPerMin: number;
    visionScore: number;
    damageDealt: number;
    goldEarned: number;
  };
  comparison: Record<
    string,
    {
      current: number;
      historical: number;
      delta: number;
      deltaPercent: number;
    }
  >;
  bestStat: { statName: string; improvement: string };
  improvementArea: { statName: string; deficit: string } | null;
}

export interface AppSettings {
  riotApiKey: string;
  region: string;
  tier: "free" | "premium";
  matchHistoryDepth: 10 | 50;
  panels: {
    loadingPanel: { enabled: boolean; opacity: number };
    winCondition: { enabled: boolean; opacity: number };
    hunterFocus: { enabled: boolean; opacity: number };
    postGame: { enabled: boolean; opacity: number };
  };
  colorblindMode: boolean;
}

type Status = "idle" | "loading" | "ready" | "error";

// ─── Store ───────────────────────────────────────────────────────

interface AppStore {
  // Platform state
  gamePhase: GamePhase;
  gameSession: GameSession | null;

  // Enemy profiles (Loading Screen)
  enemyProfiles: PlayerProfile[];
  enemyProfilesStatus: Status;

  // Win condition
  winCondition: WinConditionReport | null;
  winConditionStatus: Status;

  // Hunter focus
  hunterTarget: HunterProfile | null;
  hunterTargetIndex: number | null;
  hunterFocusStatus: Status;
  hunterPanelVisible: boolean;

  // Post game
  postGameReview: PostGameReview | null;
  postGameStatus: Status;

  // Settings
  settings: AppSettings;

  // Actions
  setGamePhase: (phase: GamePhase) => void;
  setGameSession: (session: GameSession) => void;
  setEnemyProfiles: (profiles: PlayerProfile[], status: Status) => void;
  setWinCondition: (report: WinConditionReport | null, status: Status) => void;
  lockTarget: (index: number) => void;
  setHunterTarget: (profile: HunterProfile | null, status: Status) => void;
  toggleHunterPanel: () => void;
  setPostGameReview: (review: PostGameReview | null, status: Status) => void;
  updateSettings: (partial: Partial<AppSettings>) => void;
  resetForNewGame: () => void;
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

export const useAppStore = create<AppStore>((set, get) => ({
  // Initial state
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

  // Actions
  setGamePhase: (phase) => {
    set({ gamePhase: phase });
    // Auto-reset state on new game cycle
    if (phase === "ChampSelect") {
      get().resetForNewGame();
    }
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

  resetForNewGame: () =>
    set({
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

### 5. Basic CSS (`frontend/src/index.css`)

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: "Inter", "Segoe UI", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#root {
  width: 100%;
  height: 100%;
}
```

---

## Integration Points

### From Task 01 (Project Setup):

- Uses manifest-defined window IDs and hotkey IDs
- Loads SDK from localhost:45678 (LOLOverlay platform)

### For All Frontend Tasks (09-14):

- All components import from `lib/platform.ts`
- All state managed through `store/appStore.ts`
- Hooks will be created in later tasks

---

## Testing Requirements

### Manual Testing

1. **Platform SDK loads**:
   - Open browser console
   - Check for: `[Hunter Mode] Initialized successfully`

2. **Store initializes**:

   ```javascript
   // In browser console
   window.__ZUSTAND__; // Should show store state
   ```

3. **Hotkeys register** (will need Task 09+ for full test):
   - Press Alt+W — should log hotkey event
   - Press Alt+H — should log hotkey event

---

## Acceptance Criteria

✅ **Complete when:**

1. platform.ts exports all LOLOverlay SDK methods
2. main.tsx loads SDK and initializes platform
3. App.tsx registers all hotkeys on mount
4. App.tsx routes to correct panel based on URL param
5. appStore.ts manages all global state with Zustand
6. Store has proper TypeScript types
7. Game phase changes update store
8. No TypeScript errors
9. No runtime errors on load

---

## Performance Requirements

- **SDK load time**: <200ms
- **Platform init time**: <100ms
- **Store initialization**: <10ms

---

## Files to Create

### New Files:

- `frontend/src/lib/platform.ts`
- `frontend/src/main.tsx`
- `frontend/src/App.tsx`
- `frontend/src/store/appStore.ts`
- `frontend/src/index.css`

---

## Expected Time: 4-5 hours

## Difficulty: Medium (Configuration and integration setup)
