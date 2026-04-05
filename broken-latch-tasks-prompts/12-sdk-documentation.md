# Task 12: SDK Documentation

**Platform: broken-latch**  
**Dependencies:** Task 09 (JavaScript SDK)  
**Estimated Complexity:** Low-Medium  
**Priority:** P1 (Important for developer onboarding)

---

## Objective

Create comprehensive documentation for the broken-latch JavaScript SDK, including API reference, usage examples, tutorials, and a getting started guide. This documentation will be essential for third-party developers building apps on the platform.

---

## Context

The SDK documentation needs to cover:

1. **Getting Started Guide** - Quick setup and first app
2. **API Reference** - Complete method documentation with parameters/returns
3. **Tutorials** - Common use cases with full code examples
4. **App Manifest Guide** - How to configure app manifest.json
5. **Best Practices** - Performance, UX, and architecture recommendations
6. **Type Definitions** - TypeScript definitions for all SDK interfaces

Documentation will be:

- Hosted at `https://docs.broken-latch.gg`
- Built with VitePress or Docusaurus
- Includes interactive examples
- Auto-generated from JSDoc comments in SDK source

---

## What You Need to Build

### 1. Documentation Site Structure

```
docs/
├── .vitepress/
│   └── config.ts
├── index.md                          # Homepage
├── getting-started/
│   ├── installation.md
│   ├── your-first-app.md
│   ├── manifest.md
│   └── local-development.md
├── api/
│   ├── index.md                      # API Overview
│   ├── initialization.md
│   ├── game-lifecycle.md
│   ├── windows.md
│   ├── hotkeys.md
│   ├── storage.md
│   ├── notifications.md
│   ├── messaging.md
│   └── platform.md
├── guides/
│   ├── building-a-widget.md
│   ├── game-phase-tracking.md
│   ├── persistent-storage.md
│   ├── backend-integration.md
│   ├── permissions.md
│   └── debugging.md
├── examples/
│   ├── minimal-widget.md
│   ├── multi-panel-app.md
│   ├── hotkey-toggle.md
│   └── webhook-backend.md
├── reference/
│   ├── manifest-schema.md
│   ├── permissions.md
│   ├── game-phases.md
│   └── typescript-types.md
└── package.json
```

### 2. VitePress Configuration (`docs/.vitepress/config.ts`)

```typescript
import { defineConfig } from "vitepress";

export default defineConfig({
  title: "broken-latch",
  description: "Developer documentation for the broken-latch overlay platform",

  themeConfig: {
    nav: [
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started/installation" },
      { text: "API Reference", link: "/api/" },
      { text: "Guides", link: "/guides/building-a-widget" },
      { text: "Examples", link: "/examples/minimal-widget" },
    ],

    sidebar: {
      "/getting-started/": [
        {
          text: "Getting Started",
          items: [
            { text: "Installation", link: "/getting-started/installation" },
            { text: "Your First App", link: "/getting-started/your-first-app" },
            { text: "App Manifest", link: "/getting-started/manifest" },
            {
              text: "Local Development",
              link: "/getting-started/local-development",
            },
          ],
        },
      ],

      "/api/": [
        {
          text: "API Reference",
          items: [
            { text: "Overview", link: "/api/" },
            { text: "Initialization", link: "/api/initialization" },
            { text: "Game Lifecycle", link: "/api/game-lifecycle" },
            { text: "Windows", link: "/api/windows" },
            { text: "Hotkeys", link: "/api/hotkeys" },
            { text: "Storage", link: "/api/storage" },
            { text: "Notifications", link: "/api/notifications" },
            { text: "Messaging", link: "/api/messaging" },
            { text: "Platform", link: "/api/platform" },
          ],
        },
      ],

      "/guides/": [
        {
          text: "Guides",
          items: [
            { text: "Building a Widget", link: "/guides/building-a-widget" },
            {
              text: "Game Phase Tracking",
              link: "/guides/game-phase-tracking",
            },
            { text: "Persistent Storage", link: "/guides/persistent-storage" },
            {
              text: "Backend Integration",
              link: "/guides/backend-integration",
            },
            { text: "Permissions", link: "/guides/permissions" },
            { text: "Debugging", link: "/guides/debugging" },
          ],
        },
      ],

      "/examples/": [
        {
          text: "Examples",
          items: [
            { text: "Minimal Widget", link: "/examples/minimal-widget" },
            { text: "Multi-Panel App", link: "/examples/multi-panel-app" },
            { text: "Hotkey Toggle", link: "/examples/hotkey-toggle" },
            { text: "Webhook Backend", link: "/examples/webhook-backend" },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/broken-latch/platform" },
    ],

    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2026 broken-latch",
    },
  },
});
```

### 3. Homepage (`docs/index.md`)

````markdown
---
layout: home

hero:
  name: broken-latch
  text: League of Legends Overlay Platform
  tagline: Build powerful in-game overlays with JavaScript
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started/installation
    - theme: alt
      text: API Reference
      link: /api/
    - theme: alt
      text: View on GitHub
      link: https://github.com/broken-latch/platform

features:
  - icon: 🎮
    title: Game Lifecycle Events
    details: Automatically detect game phases and respond to champion select, loading, and in-game events.

  - icon: 🪟
    title: Widget Windows
    details: Create draggable, resizable overlay windows that persist across games.

  - icon: ⌨️
    title: Global Hotkeys
    details: Register system-wide hotkeys that work even when the game has focus.

  - icon: 💾
    title: Persistent Storage
    details: Store app data that persists across sessions with a simple key-value API.

  - icon: 🔔
    title: Notifications
    details: Show toast notifications to users without interrupting gameplay.

  - icon: 🔌
    title: Webhook Support
    details: Integrate with backend services via webhooks for real-time data.
---

## Quick Start

```bash
# Install broken-latch platform
# Download from https://broken-latch.gg

# Create your first app
npm create @broken-latch/app my-overlay-app

# Start development server
cd my-overlay-app
npm run dev
```
````

## What is broken-latch?

broken-latch is a lightweight overlay platform for League of Legends that lets you build apps using HTML, CSS, and JavaScript. The platform handles all the complex Windows API, DirectX, and process injection — you just write web code.

## Example App

```javascript
// Load the SDK
import LOLOverlay from "http://localhost:45678/sdk/loloverlay.js";

// Initialize your app
LOLOverlay.init({
  appId: "my-overlay-app",
  version: "1.0.0",
});

// Listen for game phase changes
LOLOverlay.game.onPhaseChange((event) => {
  console.log(`Game phase: ${event.current}`);

  if (event.current === "InGame") {
    LOLOverlay.windows.show("main-panel");
  }
});

// Create a widget window
await LOLOverlay.windows.create({
  id: "main-panel",
  url: "panel.html",
  defaultPosition: { x: 20, y: 100 },
  defaultSize: { width: 320, height: 480 },
  draggable: true,
  showInPhases: ["InGame"],
});
```

````

### 4. API Reference Example (`docs/api/game-lifecycle.md`)

```markdown
# Game Lifecycle API

The Game Lifecycle API provides methods to detect and respond to League of Legends game phases.

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
- `callback: (event: PhaseChangeEvent) => void` - Function to call on phase change

**Returns:** `void`

**Example:**

```javascript
LOLOverlay.game.onPhaseChange((event) => {
  console.log(`Phase changed from ${event.previous} to ${event.current}`);

  if (event.current === 'InGame') {
    // Show your widgets
    LOLOverlay.windows.show('combat-tracker');
  } else if (event.current === 'NotRunning') {
    // Hide all widgets
    LOLOverlay.windows.hide('combat-tracker');
  }
});
````

---

### `getPhase()`

Get the current game phase.

**Parameters:** None

**Returns:** `Promise<GamePhase>`

**Example:**

```javascript
const phase = await LOLOverlay.game.getPhase();

if (phase === "InGame") {
  console.log("Game is active!");
}
```

---

### `getSession()`

Get the current game session data including all players.

**Parameters:** None

**Returns:** `Promise<GameSession | null>`

Returns `null` if no active game session.

**Example:**

```javascript
const session = await LOLOverlay.game.getSession();

if (session) {
  console.log("Local player:", session.localPlayer.summonerName);
  console.log("Enemy team:");
  session.enemyTeam.forEach((player) => {
    console.log(`- ${player.summonerName} (${player.championName})`);
  });
}
```

---

## Types

### `PhaseChangeEvent`

```typescript
interface PhaseChangeEvent {
  previous: GamePhase;
  current: GamePhase;
  timestamp: number; // Unix timestamp
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
  position?: string; // "TOP", "JUNGLE", etc.
}
```

````

### 5. Tutorial Example (`docs/guides/building-a-widget.md`)

```markdown
# Building a Widget

This guide walks you through creating a simple overlay widget that displays during games.

## Step 1: Create the HTML

Create `widget.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <title>My Widget</title>
  <style>
    body {
      margin: 0;
      padding: 20px;
      background: rgba(0, 0, 0, 0.8);
      color: white;
      font-family: Arial, sans-serif;
    }

    .header {
      font-size: 18px;
      font-weight: bold;
      margin-bottom: 10px;
      cursor: move;
    }

    .content {
      font-size: 14px;
    }
  </style>
</head>
<body>
  <div class="header" data-broken-latch-drag="true">
    My Widget
  </div>
  <div class="content" id="content">
    Waiting for game...
  </div>

  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script src="widget.js"></script>
</body>
</html>
````

## Step 2: Add JavaScript Logic

Create `widget.js`:

```javascript
// Initialize SDK
LOLOverlay.init({
  appId: "my-widget-app",
  version: "1.0.0",
});

const contentEl = document.getElementById("content");

// Listen for game phase changes
LOLOverlay.game.onPhaseChange(async (event) => {
  if (event.current === "InGame") {
    const session = await LOLOverlay.game.getSession();

    if (session) {
      const enemyNames = session.enemyTeam
        .map((p) => p.summonerName)
        .join("<br>");

      contentEl.innerHTML = `
        <strong>Enemy Team:</strong><br>
        ${enemyNames}
      `;
    }
  } else {
    contentEl.textContent = "Waiting for game...";
  }
});
```

## Step 3: Create Manifest

Create `manifest.json`:

```json
{
  "id": "my-widget-app",
  "name": "My Widget",
  "version": "1.0.0",
  "description": "A simple overlay widget",
  "author": "Your Name",
  "entryPoint": "widget.html",
  "permissions": ["game.session", "windows.create"],
  "windows": [
    {
      "id": "main",
      "url": "widget.html",
      "defaultPosition": { "x": 20, "y": 100 },
      "defaultSize": { "width": 250, "height": 300 },
      "draggable": true,
      "resizable": false,
      "persistPosition": true,
      "opacity": 0.9,
      "showInPhases": ["InGame"]
    }
  ]
}
```

## Step 4: Package and Install

```bash
# Package as .lolapp
zip -r my-widget-app.lolapp manifest.json widget.html widget.js

# Install via broken-latch UI
# File → Install App → Select my-widget-app.lolapp
```

## Next Steps

- [Add hotkeys](./hotkey-toggle.md)
- [Use persistent storage](./persistent-storage.md)
- [Integrate with a backend](./backend-integration.md)

````

### 6. TypeScript Definitions (`docs/reference/typescript-types.md`)

```markdown
# TypeScript Type Definitions

Full TypeScript type definitions for the broken-latch SDK.

## Installation

```bash
npm install --save-dev @broken-latch/types
````

## Usage

```typescript
/// <reference types="@broken-latch/sdk" />

import type { GamePhase, GameSession, WidgetConfig } from "@broken-latch/sdk";

const phase: GamePhase = await LOLOverlay.game.getPhase();
```

## Type Exports

See the [full type definitions file](https://github.com/broken-latch/sdk/blob/main/types/index.d.ts).

```typescript
// Core types
export type GamePhase =
  | "NotRunning"
  | "Launching"
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
  gameMode: string;
  mapId: number;
}

export interface SessionPlayer {
  summonerName: string;
  puuid: string;
  championId: number;
  championName: string;
  teamId: number;
  position?: string;
}

export interface WidgetConfig {
  id: string;
  url: string;
  defaultPosition: { x: number; y: number };
  defaultSize: { width: number; height: number };
  minSize?: { width: number; height: number };
  maxSize?: { width: number; height: number };
  draggable: boolean;
  resizable: boolean;
  persistPosition: boolean;
  clickThrough: boolean;
  opacity: number;
  showInPhases: string[];
}

// ... more types
```

````

---

## Documentation Deployment

Deploy to GitHub Pages or Vercel:

```json
// package.json
{
  "scripts": {
    "docs:dev": "vitepress dev docs",
    "docs:build": "vitepress build docs",
    "docs:preview": "vitepress preview docs"
  }
}
````

---

## Testing Requirements

### Manual Testing Checklist

- [ ] Documentation site builds successfully
- [ ] All internal links work
- [ ] Code examples are syntax-highlighted
- [ ] API reference is complete and accurate
- [ ] Tutorials are clear and follow best practices
- [ ] Type definitions match SDK implementation
- [ ] Search functionality works
- [ ] Mobile responsive design
- [ ] Dark mode works correctly

---

## Acceptance Criteria

✅ **Complete when:**

1. Documentation site is built and deployed
2. All API methods are documented
3. Getting started guide is complete
4. At least 4 tutorial guides written
5. At least 3 complete code examples provided
6. TypeScript definitions are published
7. All code examples are tested and work
8. Documentation is searchable
9. All manual tests pass

---

## Files to Create

### New Files:

- `docs/` directory with full structure
- All markdown documentation files
- VitePress configuration
- TypeScript type definition files

---

## Expected Time: 8-10 hours

## Difficulty: Low-Medium (Technical writing + documentation tooling)
