# broken-latch Platform - Tasks Overview

## Task Dependencies Graph

```
01: Project Setup (Foundation)
    ↓
02: Core Overlay Window ←─────────┐
    ↓                              │
03: DirectX Hook DLL               │
    ↓                              │
04: Game Lifecycle Detector        │
    ↓                              │
05: Hotkey Manager                 │
    ↓                              │
06: Widget Window System ──────────┘
    ↓
07: App Lifecycle Manager
    ↓
08: HTTP API Server
    ↓
09: JavaScript SDK ←─ (YOU ARE HERE - Already created)
    ↓
10: Platform UI (Tray, App Manager)
    ↓
11: App Distribution System
    ↓
12: SDK Documentation
    ↓
13: Developer CLI (Optional - Last Priority)
```

## Task List with Status

### Phase 1: Core Platform (P0 - Critical Path)

- [x] **Task 01**: Project Setup & Foundation
  - **File**: `01-project-setup.md`
  - **Dependencies**: None
  - **Time**: 4-6 hours
  - **Description**: Initialize Tauri project, database schema, config system

- [x] **Task 02**: Core Overlay Window System
  - **File**: `02-core-overlay-window.md`
  - **Dependencies**: Task 01
  - **Time**: 6-8 hours
  - **Description**: Transparent fullscreen window with Windows API

- [ ] **Task 03**: DirectX Hook DLL
  - **File**: `03-directx-hook-dll.md`
  - **Dependencies**: Task 02
  - **Time**: 10-12 hours
  - **Description**: C++ DLL for hooking Present() and compositing overlay

- [ ] **Task 04**: Game Lifecycle Detector
  - **File**: `04-game-lifecycle-detector.md`
  - **Dependencies**: Task 01
  - **Time**: 6-8 hours
  - **Description**: Detect League phases via process monitoring + LCU WebSocket

- [ ] **Task 05**: Hotkey Manager
  - **File**: `05-hotkey-manager.md`
  - **Dependencies**: Task 01
  - **Time**: 4-6 hours
  - **Description**: Global hotkey registration with conflict resolution

### Phase 2: App Infrastructure (P0)

- [ ] **Task 06**: Widget Window System
  - **File**: `06-widget-window-system.md`
  - **Dependencies**: Task 02, 04
  - **Time**: 6-8 hours
  - **Description**: Per-app widget rendering, drag/resize, phase-based visibility

- [ ] **Task 07**: App Lifecycle Manager
  - **File**: `07-app-lifecycle-manager.md`
  - **Dependencies**: Task 01, 06
  - **Time**: 8-10 hours
  - **Description**: App installation, loading, sandboxing, manifest parsing

- [ ] **Task 08**: HTTP API Server
  - **File**: `08-http-api-server.md`
  - **Dependencies**: Task 01, 04, 05, 06, 07
  - **Time**: 8-10 hours
  - **Description**: Axum REST API + webhook system for apps

### Phase 3: Developer Experience (P0)

- [x] **Task 09**: JavaScript SDK
  - **File**: `09-javascript-sdk.md`
  - **Dependencies**: Task 01, 04, 05, 06, 08
  - **Time**: 8-10 hours
  - **Description**: LOLOverlay.js SDK with full API wrapping

### Phase 4: Platform UI (P1)

- [ ] **Task 10**: Platform UI (System Tray + App Manager)
  - **File**: `10-platform-ui.md`
  - **Dependencies**: Task 07, 09
  - **Time**: 10-12 hours
  - **Description**: React UI for app management, settings, tray icon

- [ ] **Task 11**: App Distribution System
  - **File**: `11-app-distribution-system.md`
  - **Dependencies**: Task 07, 10
  - **Time**: 8-10 hours
  - **Description**: .lolapp package format, app store browser, auto-update

### Phase 5: Documentation (P2 - After SDK is stable)

- [ ] **Task 12**: SDK Documentation
  - **File**: `12-sdk-documentation.md`
  - **Dependencies**: Task 09
  - **Time**: 6-8 hours
  - **Description**: Complete API reference, guides, examples

- [ ] **Task 13**: Developer CLI
  - **File**: `13-developer-cli.md`
  - **Dependencies**: Task 09, 12
  - **Time**: 8-10 hours
  - **Description**: `broken-latch new`, `broken-latch dev`, `broken-latch publish`

---

## Total Estimated Time: 90-120 hours

## Critical Path (Must complete in order):

1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11

## Can work in parallel after Task 01:

- Tasks 02, 04, 05 (different modules, minimal overlap)

---

## Performance Targets Summary

| Task                | RAM Impact  | CPU Impact   | Critical Metric       |
| ------------------- | ----------- | ------------ | --------------------- |
| 02 - Overlay Window | <5MB        | <0.5%        | <16ms show/hide       |
| 03 - DirectX Hook   | <3MB        | <2%          | <0.3ms per frame      |
| 04 - Game Detector  | <2MB        | <0.3%        | <3s detection lag     |
| 05 - Hotkeys        | <1MB        | <0.1%        | <10ms hotkey response |
| 06 - Widgets        | <5MB/widget | <0.5%/widget | <16ms render          |
| 07 - App Loader     | <20MB/app   | <1%/app      | <500ms app startup    |

**Total Platform Overhead**: <30MB idle, <2% CPU in-game

---

## Testing Strategy

### Per-Task Testing:

- **Unit tests**: All Rust modules, TypeScript SDK
- **Integration tests**: Cross-module interactions
- **Manual tests**: Checklist in each task prompt

### End-to-End Testing (After Task 09):

1. Install platform
2. Install test app (Hunter Mode)
3. Launch League of Legends
4. Verify all lifecycle phases detected
5. Verify app widgets render correctly
6. Verify hotkeys work
7. Verify storage persists
8. Measure RAM/CPU usage
9. Run for 5+ games - check stability

---

## Files Generated by All Tasks

```
broken-latch/
├── src/                          (Task 01, 10)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              (Task 01, all tasks modify)
│   │   ├── overlay/             (Task 02)
│   │   ├── hook/                (Task 03)
│   │   ├── game/                (Task 04)
│   │   ├── hotkey.rs            (Task 05)
│   │   ├── widgets.rs           (Task 06)
│   │   ├── apps/                (Task 07, 11)
│   │   ├── http_api.rs          (Task 08)
│   │   ├── sdk_server.rs        (Task 09)
│   │   ├── db.rs                (Task 01)
│   │   ├── config.rs            (Task 01)
│   │   └── tray.rs              (Task 10)
│   ├── migrations/              (Task 01)
│   ├── Cargo.toml               (Task 01, all tasks modify)
│   └── tauri.conf.json          (Task 01, 10)
├── overlay-hook/                 (Task 03)
│   ├── dllmain.cpp
│   ├── dx_hook.cpp
│   └── CMakeLists.txt
├── sdk/                          (Task 09)
│   ├── src/
│   └── dist/
│       └── loloverlay.js
├── cli/                          (Task 13)
│   └── src/
└── docs/                         (Task 12)
    ├── SDK_REFERENCE.md
    ├── QUICKSTART.md
    └── API_GUIDE.md
```

---

## Next Steps for You

1. **Complete Task 01** first (foundation)
2. **Then Task 02** (overlay window - critical)
3. **Then Task 03** (DirectX hook - most complex)
4. **Then Task 04** (game detection - enables testing)
5. Continue in order through Task 11
6. Tasks 12-13 can wait until platform is functional

---

## Questions or Blockers?

Each task prompt contains:

- ✅ Full context and objective
- ✅ Exact code to implement
- ✅ Integration points with other tasks
- ✅ Testing requirements
- ✅ Acceptance criteria

**If you get stuck:**

- Check "Integration Points" section in task prompt
- Review dependencies in this overview
- Each task is self-contained - you have all the context needed

---

## Platform vs App Development

**broken-latch (Platform)**: These 13 tasks build the infrastructure
**Hunter Mode (App)**: Separate task prompts in `../hunter-mode-tasks-prompts/`

Platform must be complete before Hunter Mode can be built, since Hunter Mode depends on the SDK (Task 09).
