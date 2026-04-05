# Task 10: Frontend - Loading Panel

**App: Hunter Mode**  
**Dependencies:** Task 01, 04, 08, 09  
**Estimated Complexity:** High  
**Priority:** P0 (Core feature)

---

## Objective

Build the **Loading Screen Panel** — Hunter Mode's flagship pre-game feature. During champion loading, this panel displays 5 enemy profiles with win rates, KDA, mastery, playstyle tags, and threat levels. Users can click rows to expand full details.

---

## Context

When the game phase changes to "Loading", the `useEnemyProfiles` hook (created in this task) fetches enemy data from the backend. The panel renders:

- **Compact mode**: 5 rows showing summoner name, rank, win rate, KDA
- **Expanded mode**: Clicked row reveals CS/min, vision score, common build, summoner spells, playstyle tag, recent form, threat meter
- **Auto-hide**: Fades out 8 seconds after game phase changes to "InGame"

This panel is rendered at `frontend/index.html?panel=loading` as configured in the manifest.

---

## What You Need to Build

### 1. Data Hook (`frontend/src/hooks/useEnemyProfiles.ts`)

```typescript
import { useEffect } from "react";
import axios from "axios";
import { useAppStore } from "../store/appStore";
import { getSession } from "../lib/platform";
import { API_BASE_URL } from "../lib/constants";
import type { PlayerProfile } from "../store/appStore";

export function useEnemyProfiles() {
  const { gamePhase, setGameSession, setEnemyProfiles } = useAppStore();

  useEffect(() => {
    if (gamePhase !== "Loading") return;

    async function fetchProfiles() {
      setEnemyProfiles([], "loading");

      try {
        // 1. Get game session from platform
        const session = await getSession();
        setGameSession(session);

        // 2. Extract enemy data
        const summonerNames = session.enemyTeam
          .map((p) => p.summonerName)
          .join(",");
        const championIds = session.enemyTeam
          .map((p) => p.championId)
          .join(",");

        // 3. Call backend
        const response = await axios.get<{ profiles: PlayerProfile[] }>(
          `${API_BASE_URL}/enemies`,
          {
            params: {
              summonerNames,
              championIds,
              region: "na1", // TODO: Get from settings
            },
          },
        );

        setEnemyProfiles(response.data.profiles, "ready");
      } catch (error) {
        console.error("[useEnemyProfiles] Error:", error);
        setEnemyProfiles([], "error");
      }
    }

    fetchProfiles();
  }, [gamePhase]);
}
```

### 2. Loading Panel Component (`frontend/src/panels/LoadingPanel/LoadingPanel.tsx`)

```tsx
import { useEffect, useState } from "react";
import { useAppStore } from "../../store/appStore";
import { LoadingSpinner } from "../../shared";
import { EnemyRow } from "./EnemyRow";

export function LoadingPanel() {
  const { enemyProfiles, enemyProfilesStatus, gamePhase } = useAppStore();
  const [visible, setVisible] = useState(true);

  // Auto-hide 8 seconds after entering InGame
  useEffect(() => {
    if (gamePhase === "InGame") {
      const timer = setTimeout(() => setVisible(false), 8000);
      return () => clearTimeout(timer);
    } else if (gamePhase === "Loading") {
      setVisible(true);
    }
  }, [gamePhase]);

  if (!visible) return null;

  return (
    <div className="w-full h-full p-4 bg-hm-bg/90 backdrop-blur-md rounded-lg border border-hm-border shadow-2xl">
      {/* Header */}
      <div className="mb-3 flex items-center justify-between">
        <h2 className="text-lg font-bold text-hm-primary">Enemy Team</h2>
        {enemyProfilesStatus === "loading" && <LoadingSpinner size="sm" />}
      </div>

      {/* Enemy rows */}
      <div className="space-y-2">
        {enemyProfilesStatus === "loading" &&
          [0, 1, 2, 3, 4].map((i) => (
            <div
              key={i}
              className="h-12 bg-hm-border/20 rounded animate-pulse"
            />
          ))}

        {enemyProfilesStatus === "ready" &&
          enemyProfiles.map((profile, index) => (
            <EnemyRow key={index} profile={profile} index={index} />
          ))}

        {enemyProfilesStatus === "error" && (
          <div className="text-center py-8 text-hm-muted">
            Failed to load enemy profiles
          </div>
        )}
      </div>
    </div>
  );
}
```

### 3. Enemy Row Component (Collapsed) (`frontend/src/panels/LoadingPanel/EnemyRow.tsx`)

```tsx
import { useState } from "react";
import { ChampionIcon, RankBadge, WinRatePill, KdaDisplay } from "../../shared";
import { EnemyRowExpanded } from "./EnemyRowExpanded";
import type { PlayerProfile } from "../../store/appStore";

interface EnemyRowProps {
  profile: PlayerProfile;
  index: number;
}

export function EnemyRow({ profile, index }: EnemyRowProps) {
  const [expanded, setExpanded] = useState(false);

  if (profile.error) {
    return (
      <div className="p-3 bg-hm-border/10 rounded text-hm-muted text-sm">
        Profile unavailable for {profile.summonerName}
      </div>
    );
  }

  return (
    <div className="bg-hm-border/10 rounded overflow-hidden transition-all">
      {/* Collapsed row (always visible) */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full p-3 flex items-center gap-3 hover:bg-hm-border/20 transition-colors text-left"
      >
        {/* Champion icon */}
        <ChampionIcon championId={profile.championId} size={32} />

        {/* Summoner name */}
        <div className="flex-1 min-w-0">
          <div className="text-sm font-semibold text-hm-primary truncate">
            {profile.summonerName}
          </div>
          <div className="text-xs text-hm-secondary">
            Mastery {profile.masteryLevel} • {profile.gamesPlayed}G
          </div>
        </div>

        {/* Rank */}
        <RankBadge rank={profile.rank} size="sm" />

        {/* Win rate */}
        <WinRatePill
          winRate={profile.winRate}
          gamesPlayed={profile.gamesPlayed}
          size="sm"
        />

        {/* KDA */}
        <KdaDisplay kda={profile.avgKda} size="sm" />

        {/* Expand arrow */}
        <svg
          className={`w-4 h-4 text-hm-secondary transition-transform ${
            expanded ? "rotate-180" : ""
          }`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {/* Expanded details */}
      {expanded && <EnemyRowExpanded profile={profile} />}
    </div>
  );
}
```

### 4. Enemy Row Expanded (`frontend/src/panels/LoadingPanel/EnemyRowExpanded.tsx`)

```tsx
import { StatBar, ThreatIndicator, ItemIcon } from "../../shared";
import type { PlayerProfile } from "../../store/appStore";

interface EnemyRowExpandedProps {
  profile: PlayerProfile;
}

export function EnemyRowExpanded({ profile }: EnemyRowExpandedProps) {
  return (
    <div className="px-3 pb-3 pt-2 space-y-3 border-t border-hm-border/20">
      {/* Stats grid */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div>
          <span className="text-hm-secondary">CS/min:</span>{" "}
          <span className="text-hm-primary font-semibold">
            {profile.avgCsPerMin.toFixed(1)}
          </span>
        </div>
        <div>
          <span className="text-hm-secondary">Vision:</span>{" "}
          <span className="text-hm-primary font-semibold">
            {profile.avgVisionScore.toFixed(1)}
          </span>
        </div>
        <div>
          <span className="text-hm-secondary">Avg Duration:</span>{" "}
          <span className="text-hm-primary font-semibold">
            {profile.avgGameDurationMins.toFixed(0)}m
          </span>
        </div>
      </div>

      {/* Common build */}
      <div>
        <div className="text-xs text-hm-secondary mb-1">Common Build:</div>
        <div className="flex gap-1">
          {profile.mostCommonBuild.slice(0, 6).map((itemId, i) => (
            <ItemIcon key={i} itemId={itemId} size={24} />
          ))}
        </div>
      </div>

      {/* Summoner spells */}
      <div>
        <div className="text-xs text-hm-secondary mb-1">Summoner Spells:</div>
        <div className="flex gap-1">
          {profile.mostCommonSummonerSpells.map((spellId, i) => (
            <div
              key={i}
              className="w-6 h-6 bg-hm-border/30 rounded text-xs flex items-center justify-center text-hm-muted"
            >
              {spellId}
            </div>
          ))}
        </div>
      </div>

      {/* Playstyle & Form */}
      <div className="flex items-center gap-2 text-xs">
        <span className="px-2 py-1 bg-hm-favorable/20 text-hm-favorable rounded">
          {profile.playstyleTag}
        </span>
        <span
          className={`px-2 py-1 rounded ${
            profile.recentForm.includes("Win")
              ? "bg-hm-win/20 text-hm-win"
              : profile.recentForm.includes("Loss")
                ? "bg-hm-loss/20 text-hm-loss"
                : "bg-hm-even/20 text-hm-even"
          }`}
        >
          {profile.recentForm}
        </span>
      </div>

      {/* Threat level */}
      <div>
        <div className="text-xs text-hm-secondary mb-1">Threat Level:</div>
        <ThreatIndicator level={profile.threatLevel} showLabel />
      </div>
    </div>
  );
}
```

### 5. Panel Styles (`frontend/src/panels/LoadingPanel/LoadingPanel.css`)

```css
/* Fade-in animation */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.loading-panel {
  animation: fadeIn 0.2s ease-out;
}

/* Fade-out animation */
@keyframes fadeOut {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

.loading-panel.hiding {
  animation: fadeOut 0.3s ease-in forwards;
}
```

---

## Integration Points

### From Task 04 (Backend Enemy Profiles):

- Calls `GET /enemies?summonerNames=...&championIds=...&region=...`
- Receives `{ profiles: PlayerProfile[] }`

### From Task 08 (Platform Integration):

- Uses `getSession()` to get enemy team data
- Uses `useAppStore` for state management

### From Task 09 (Shared Components):

- Uses ChampionIcon, RankBadge, WinRatePill, KdaDisplay, StatBar, ThreatIndicator, ItemIcon

---

## Testing Requirements

### Manual Testing

1. **Loading state**:
   - Set `enemyProfilesStatus` to "loading"
   - Verify 5 skeleton rows appear

2. **Data display**:
   - Mock profile data in store
   - Verify all fields render correctly

3. **Expand/collapse**:
   - Click row
   - Verify expanded view shows
   - Click again to collapse

4. **Auto-hide**:
   - Change game phase to "InGame"
   - Wait 8 seconds
   - Verify panel fades out

---

## Acceptance Criteria

✅ **Complete when:**

1. useEnemyProfiles hook fetches on Loading phase
2. Panel renders 5 enemy rows
3. Loading state shows skeleton placeholders
4. Error state shows error message
5. Collapsed rows show name, rank, win rate, KDA
6. Clicking row expands to show full stats
7. Expanded view shows CS/min, vision, build, spells, playstyle, form, threat
8. Panel auto-hides 8 seconds after entering InGame
9. All animations are smooth (60fps)
10. No TypeScript errors
11. No console warnings

---

## Performance Requirements

- **Render time**: <100ms for 5 rows
- **Expand animation**: 60fps
- **Memory**: <2MB for panel

---

## Files to Create

### New Files:

- `frontend/src/hooks/useEnemyProfiles.ts`
- `frontend/src/panels/LoadingPanel/LoadingPanel.tsx`
- `frontend/src/panels/LoadingPanel/EnemyRow.tsx`
- `frontend/src/panels/LoadingPanel/EnemyRowExpanded.tsx`
- `frontend/src/panels/LoadingPanel/LoadingPanel.css`

---

## Expected Time: 8-10 hours

## Difficulty: High (Complex UI with expand/collapse logic)
