# Task 11-15: Final Hunter Mode Frontend Tasks

**Note:** These are the remaining 5 frontend tasks for Hunter Mode. Each follows the same pattern as Task 10.

---

## Task 11: Frontend - Win Condition Panel

### Files to Create:

- `frontend/src/panels/WinCondition/WinConditionPanel.tsx`
- `frontend/src/panels/WinCondition/PowerTimeline.tsx`
- `frontend/src/panels/WinCondition/KeyThreatBadge.tsx`
- `frontend/src/panels/WinCondition/WinConditionPanel.css`
- `frontend/src/hooks/useWinCondition.ts`

### Key Features:

- Renders when `gamePhase === "Loading"` or `gamePhase === "InGame"`
- Shows ally win conditions, enemy win conditions
- Power timeline bars (early/mid/late game)
- Key threats (top 2 enemy champions)
- Draft tips (1-2 coaching lines)
- Toggle with `Alt+W` hotkey
- Collapsible sidebar, right edge

### Data Flow:

1. Hook watches `enemyProfilesStatus === "ready"`
2. Calls `GET /win-condition` with both team compositions
3. Backend returns `WinConditionReport`
4. Renders with color coding (blue=favorable, gray=even, orange=unfavorable)

---

## Task 12: Frontend - Hunter Focus Panel

### Files to Create:

- `frontend/src/panels/HunterFocus/HunterFocusPanel.tsx`
- `frontend/src/panels/HunterFocus/TargetHeader.tsx`
- `frontend/src/panels/HunterFocus/AbilityTendency.tsx`
- `frontend/src/panels/HunterFocus/PositionalTendency.tsx`
- `frontend/src/panels/HunterFocus/CounterTips.tsx`
- `frontend/src/panels/HunterFocus/ThreatMeter.tsx`
- `frontend/src/panels/HunterFocus/HunterFocusPanel.css`
- `frontend/src/hooks/useHunterTarget.ts`

### Key Features:

- Renders when `gamePhase === "InGame"` AND `hunterPanelVisible === true`
- Target selection via `Alt+1` through `Alt+5` hotkeys
- Shows deep player profile for locked target
- Ability tendencies (skill max order)
- Positional tendencies (lane positions over time)
- Threat meter (1-5 dots, color-coded)
- Counter tips and recommended items
- Sidebar, left edge of screen

### Data Flow:

1. Hook watches `hunterTargetIndex` changes
2. Calls `GET /hunter/:summonerName/:championId`
3. Backend returns detailed `PlayerProfile`
4. Switching targets is instant (shows spinner then new data)

---

## Task 13: Frontend - Post-Game Panel

### Files to Create:

- `frontend/src/panels/PostGame/PostGamePanel.tsx`
- `frontend/src/panels/PostGame/StatCompare.tsx`
- `frontend/src/panels/PostGame/HunterOutcome.tsx`
- `frontend/src/panels/PostGame/PostGamePanel.css`
- `frontend/src/hooks/usePostGame.ts`

### Key Features:

- Renders when `gamePhase === "EndGame"` AND `postGameStatus === "ready"`
- Full-screen card overlay
- Your stats vs personal average (KDA Δ, CS Δ, vision Δ, damage Δ)
- Your stats vs rank average
- Key strength (highest positive delta)
- Key improvement area (highest negative delta)
- Hunter target outcome (did you shut them down?)
- Dismiss with `Escape` hotkey

### Data Flow:

1. Hook watches `gamePhase === "EndGame"`
2. Calls `GET /post-game/:matchId/:puuid`
3. Backend returns `PostGameReview`
4. Renders stat comparison bars with +/- indicators

---

## Task 14: Frontend - Settings & Setup Flow

### Files to Create:

- `frontend/src/components/Settings/SettingsPanel.tsx`
- `frontend/src/components/Settings/ApiKeyInput.tsx`
- `frontend/src/components/Settings/RegionSelector.tsx`
- `frontend/src/components/Settings/SetupWizard.tsx`
- `frontend/src/components/Settings/Settings.css`

### Key Features:

- First-run setup wizard (4 steps)
- API key input with validation
- Region dropdown
- Hotkey configuration
- Widget opacity sliders
- Feature toggles
- Stored via `LOLOverlay.storage.set("settings", ...)`

### Setup Flow:

1. Check if `riotApiKey === ""` on startup
2. If empty, show setup wizard instead of panels
3. Step 1: Welcome
4. Step 2: API key input + validation button
5. Step 3: Region selection
6. Step 4: Ready message
7. Save settings and show normal panels

---

## Task 15: Integration Testing & Packaging

### Files to Create:

- `tests/integration/backend.test.js`
- `tests/integration/frontend.test.js`
- `tests/e2e/full-flow.test.js`
- `scripts/package.sh`
- `README.md`

### Key Features:

- Backend integration tests (all routes)
- Frontend component tests (all panels)
- E2E test: install → game launch → all phases → verify panels
- .lolapp packaging script
- Documentation (setup, build, dev mode)

### E2E Test Flow:

1. Mock broken-latch platform running
2. Install hunter-mode.lolapp
3. Simulate phase changes: NotRunning → Loading → InGame → EndGame
4. Verify each panel renders correctly
5. Verify hotkeys work
6. Verify storage persists
7. Check RAM/CPU usage

### Packaging:

```bash
#!/bin/bash
# scripts/package.sh

# Build frontend
cd frontend
npm run build

# Build backend
cd ../backend
npm install --production

# Create .lolapp (zip)
cd ..
zip -r hunter-mode-1.0.0.lolapp \
  manifest.json \
  frontend/dist/ \
  backend/ \
  icons/

echo "✅ Package created: hunter-mode-1.0.0.lolapp"
```

---

## Quick Reference: All 5 Tasks

| Task | Focus             | Time   | Files   |
| ---- | ----------------- | ------ | ------- |
| 11   | Win Condition UI  | 6-8h   | 5 files |
| 12   | Hunter Focus UI   | 10-12h | 7 files |
| 13   | Post-Game UI      | 6-8h   | 5 files |
| 14   | Settings + Setup  | 6-8h   | 5 files |
| 15   | Testing + Package | 8-10h  | 5 files |

**Total: 36-48 hours**

---

## Implementation Order

1. **Task 11** (Win Condition Panel) - simpler, good practice
2. **Task 12** (Hunter Focus Panel) - most complex UI
3. **Task 13** (Post-Game Panel) - straightforward
4. **Task 14** (Settings Flow) - required for first run
5. **Task 15** (Integration Testing) - validates everything works

---

## Common Patterns Across Tasks 11-13

All panel components follow this structure:

```tsx
export function PanelName() {
  const { data, status } = useAppStore();

  if (status === "loading") return <LoadingSpinner />;
  if (status === "error") return <ErrorMessage />;
  if (!data) return null;

  return <div className="panel-container">{/* Panel content */}</div>;
}
```

All hooks follow this structure:

```typescript
export function useHookName() {
  const { trigger, setData } = useAppStore();

  useEffect(() => {
    if (!trigger) return;

    async function fetch() {
      setData(null, "loading");
      try {
        const response = await axios.get(`${API_BASE_URL}/endpoint`);
        setData(response.data, "ready");
      } catch (error) {
        setData(null, "error");
      }
    }

    fetch();
  }, [trigger]);
}
```

---

## Testing Each Panel

For each panel (Tasks 11-13):

1. **Unit Test**: Render with mock data, verify elements present
2. **Integration Test**: Hook fetches real data from backend
3. **Visual Test**: Screenshot comparison in different states
4. **Performance Test**: Render time <100ms

---

## You're Ready to Implement!

With Tasks 01-10 complete and these final 5 outlined, you have everything needed to build the complete Hunter Mode app. Follow the established patterns and you'll finish strong! 🚀
