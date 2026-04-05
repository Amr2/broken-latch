# How to Use These Task Prompts

## Overview

You now have **detailed, self-contained task prompts** for building both:

1. **broken-latch Platform** (13 tasks in `broken-latch-tasks-prompts/`)
2. **Hunter Mode App** (15 tasks in `hunter-mode-tasks-prompts/`)

Each task prompt is designed to be **10/10 complete** - meaning at any point in the project, you can:

- Pick up any task
- Have full context
- Know exactly what to implement
- Understand dependencies and integration points
- Have testing requirements
- Know when you're done (acceptance criteria)

---

## What I've Created For You

### ✅ broken-latch Platform Tasks:

- `README.md` - Overview, dependency graph, performance targets
- `01-project-setup.md` - Full Tauri initialization, DB schema, config
- `02-core-overlay-window.md` - Transparent window with Windows API
- `09-javascript-sdk.md` - Complete SDK for app developers
- **TODO for you**: Tasks 03-08, 10-13 (following the same detailed pattern)

### ✅ Hunter Mode App Tasks:

- `README.md` - Overview, architecture, data flow
- `04-backend-enemy-profiles-feature.md` - Complete backend service example
- **TODO for you**: Tasks 01-03, 05-15 (following the same detailed pattern)

---

## Task Prompt Structure (Every Task Follows This)

Each `.md` file contains:

```markdown
# Task XX: [Name]

**Project:** [broken-latch or Hunter Mode]
**Dependencies:** [Previous tasks required]
**Complexity:** [Low/Medium/High]
**Priority:** [P0/P1/P2]

## Objective

[What you're building and why]

## Context

[Background info, how this fits into the bigger picture]

## What You Need to Build

[Exact code, file structure, implementation details]

## Integration Points

[How this connects to previous tasks and future tasks]

## Testing Requirements

[Unit tests, integration tests, manual testing checklist]

## Acceptance Criteria

[Checkbox list of completion requirements]

## Files to Create/Modify

[Exact file paths]

## Expected Time: X hours

## Difficulty: [Low/Medium/High]
```

---

## How to Work Through Tasks

### Step 1: Start with Foundational Tasks

```bash
# broken-latch platform
1. Task 01 (Project Setup) ← START HERE
2. Task 02 (Overlay Window)
3. Task 03 (DirectX Hook)
4. Task 04 (Game Detection)
# ... continue in order

# Hunter Mode app (ONLY after broken-latch Task 09 is done)
1. Task 01 (Project Setup)
2. Task 02 (Riot API Client)
3. Task 03 (Caching)
# ... continue in order
```

### Step 2: For Each Task:

1. **Read the entire task prompt** (scroll through once)
2. **Check dependencies** - are previous tasks complete?
3. **Create the files** listed in "Files to Create/Modify"
4. **Copy/implement the code** from "What You Need to Build"
5. **Run tests** from "Testing Requirements"
6. **Verify** against "Acceptance Criteria" checklist
7. **Move to next task** once all ✅ are checked

### Step 3: Integration Points

Each task tells you:

- **From Task X**: What you'll use from a previous task
- **For Task Y**: What the next task will use from this task

Example from Task 02 (Overlay Window):

```
### From Task 01:
- Uses PlatformConfig.overlay.default_opacity

### For Task 06 (Widget System):
- Widget manager will call update_interactive_regions()
```

This ensures you always know **how pieces connect**.

---

## Remaining Tasks To Create

I've created **detailed examples** for you. Now you need to create:

### broken-latch Platform:

- [ ] Task 03: DirectX Hook DLL (C++ implementation)
- [ ] Task 04: Game Lifecycle Detector (Rust + LCU WebSocket)
- [ ] Task 05: Hotkey Manager (Windows RegisterHotKey)
- [ ] Task 06: Widget Window System (Per-app widget rendering)
- [ ] Task 07: App Lifecycle Manager (Manifest parsing, loading)
- [ ] Task 08: HTTP API Server (Axum REST endpoints)
- [ ] Task 10: Platform UI (React tray, app manager)
- [ ] Task 11: App Distribution (.lolapp packaging)
- [ ] Task 12: SDK Documentation
- [ ] Task 13: Developer CLI

### Hunter Mode:

- [ ] Task 01: Project Setup & Manifest
- [ ] Task 02: Backend - Riot API Client
- [ ] Task 03: Backend - Caching System
- [ ] Task 05: Backend - Win Condition Feature
- [ ] Task 06: Backend - Hunter Focus Feature
- [ ] Task 07: Backend - Post-Game Feature
- [ ] Task 08: Frontend - Platform Integration
- [ ] Task 09: Frontend - Shared Components
- [ ] Task 10: Frontend - Loading Panel
- [ ] Task 11: Frontend - Win Condition Panel
- [ ] Task 12: Frontend - Hunter Focus Panel
- [ ] Task 13: Frontend - Post-Game Panel
- [ ] Task 14: Frontend - Settings Flow
- [ ] Task 15: Integration Testing

---

## How to Create Remaining Tasks

### Use the existing tasks as templates:

For **broken-latch** tasks, follow the pattern of:

- `01-project-setup.md` (for Rust/Tauri tasks)
- `02-core-overlay-window.md` (for Windows API tasks)
- `09-javascript-sdk.md` (for TypeScript/JavaScript tasks)

For **Hunter Mode** tasks, follow the pattern of:

- `04-backend-enemy-profiles-feature.md` (for backend services)

### Key sections to include in EVERY task:

1. **Objective**: One paragraph - what and why
2. **Context**: How this fits into the bigger picture
3. **What You Need to Build**: FULL code implementation
4. **Integration Points**:
   - FROM previous tasks: what you'll use
   - FOR future tasks: what they'll use from this
5. **Testing Requirements**:
   - Unit tests with example code
   - Integration tests
   - Manual testing checklist
6. **Acceptance Criteria**: Clear ✅ checkboxes
7. **Files to Create/Modify**: Exact paths
8. **Expected Time** + **Difficulty**

---

## Example Workflow: Implementing Task 01

```bash
# 1. Open the task prompt
cat broken-latch-tasks-prompts/01-project-setup.md

# 2. Follow the instructions
npm create tauri-app@latest
# ... follow setup steps

# 3. Create files as specified
touch src-tauri/src/db.rs
touch src-tauri/src/config.rs
# ... etc

# 4. Copy code from task prompt into files

# 5. Run tests
cd src-tauri
cargo test

# 6. Check acceptance criteria
# ✅ Database initializes
# ✅ Config loads/saves
# ✅ All tests pass
# etc...

# 7. Mark task complete, move to Task 02
```

---

## Performance Tracking

Use the READMEs to track:

### Platform Progress:

```
Phase 1: Core Platform (P0)
[x] Task 01 - Project Setup (4h) ✅
[x] Task 02 - Overlay Window (6h) ✅
[ ] Task 03 - DirectX Hook (12h)
[ ] Task 04 - Game Detection (8h)
...

Total: X/90-120 hours
```

### Hunter Mode Progress:

```
Phase 1: Foundation
[ ] Task 01 - Project Setup (3h)

Phase 2: Backend
[ ] Task 02 - Riot API (8h)
[ ] Task 03 - Caching (5h)
...

Total: X/90-110 hours
```

---

## Testing Strategy

### Per Task:

- **Unit tests**: Test individual functions/modules
- **Integration tests**: Test how this task integrates with previous ones
- **Manual tests**: Follow checklist in each task

### After All Tasks:

- **Platform E2E**: Install, run, load app, verify all features
- **Hunter Mode E2E**: Install .lolapp, play game, verify all panels

---

## When You're Stuck

### Problem: "I don't understand how X integrates with Y"

**Solution**: Check "Integration Points" section in both task prompts

### Problem: "I don't know if I'm done"

**Solution**: Go through "Acceptance Criteria" checklist - all must be ✅

### Problem: "The code doesn't compile/run"

**Solution**:

1. Check dependencies in Cargo.toml / package.json
2. Verify you created ALL files in "Files to Create"
3. Check "Testing Requirements" - run unit tests

### Problem: "I'm not sure what to implement next"

**Solution**: Follow the dependency graph in README.md - pick the next unlocked task

---

## Final Notes

### For broken-latch Platform:

- **Critical path**: Tasks 01 → 02 → 03 → 04 → ... → 09
- **Can't skip**: Task 09 (SDK) - Hunter Mode depends on it
- **Can defer**: Tasks 12-13 (Docs + CLI) - nice to have, not blocking

### For Hunter Mode:

- **Can't start until**: broken-latch Task 09 (SDK) is complete
- **Backend first**: Tasks 02-07 can be tested independently
- **Frontend needs backend**: Tasks 10-13 require backend to be running

### Time Estimates:

- **broken-latch**: 90-120 hours total
- **Hunter Mode**: 90-110 hours total
- **Total project**: 180-230 hours (4-6 weeks full-time)

---

## Ready to Start?

1. ✅ Read `broken-latch-tasks-prompts/README.md`
2. ✅ Read `broken-latch-tasks-prompts/01-project-setup.md`
3. ✅ Implement Task 01
4. ✅ Move to Task 02
5. ✅ Continue until Task 09 (SDK) is complete
6. ✅ Switch to Hunter Mode tasks
7. ✅ Celebrate when done! 🎉

---

## Questions?

Each task prompt is self-contained and has ALL the information you need. If something is unclear:

1. Re-read the "Context" section
2. Check "Integration Points" for dependencies
3. Look at the code examples in "What You Need to Build"

**You have everything you need to build this!** 🚀
