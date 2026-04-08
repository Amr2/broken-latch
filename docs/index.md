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
# Download and install broken-latch from https://broken-latch.gg

# Create your first app
npm create @broken-latch/app my-overlay-app

# Start development server
cd my-overlay-app
npm run dev
```

## What is broken-latch?

broken-latch is a lightweight overlay platform for League of Legends that lets you build apps using HTML, CSS, and JavaScript. The platform handles all the complex Windows API and process detection — you just write web code.

Apps are served from the platform's local HTTP server and displayed as transparent, click-through Tauri windows layered over the game. The SDK provides a unified API for game events, window management, hotkeys, storage, and more.

## Example App

```javascript
// Load the SDK (served by the platform)
// <script src="http://localhost:45678/sdk/loloverlay.js"></script>

LOLOverlay.init({ appId: "my-overlay-app", version: "1.0.0" });

// Listen for game phase changes
LOLOverlay.game.onPhaseChange((event) => {
  if (event.current === "InGame") {
    LOLOverlay.windows.show("main-panel");
  }
});

// Register a hotkey
LOLOverlay.hotkeys.register("toggle", "Ctrl+Shift+O", () => {
  LOLOverlay.windows.toggle("main-panel");
});
```
