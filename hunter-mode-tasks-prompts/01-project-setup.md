# Task 01: Project Setup & Manifest

**App: Hunter Mode**  
**Dependencies:** None  
**Estimated Complexity:** Medium  
**Priority:** P0 (Foundation)

---

## Objective

Bootstrap the complete Hunter Mode project structure with all necessary configuration files, dependencies, and the LOLOverlay app manifest. This task creates the foundation for both frontend (React + TypeScript + Vite) and backend (Node.js + Express) components of the app.

---

## Context

Hunter Mode is a `.lolapp` package that runs on the LOLOverlay platform. It does NOT create its own overlay window, handle DirectX, or manage game detection — all of that is provided by LOLOverlay.

This task sets up:

1. The manifest.json file that declares all windows, hotkeys, webhooks, and permissions
2. The frontend project (React 18 + TypeScript + Vite + TailwindCSS)
3. The backend project (Node.js + Express)
4. All folder structure following the spec
5. Initial package.json files with all dependencies
6. Environment configuration

---

## What You Need to Build

### 1. Root Folder Structure

```
hunter-mode/
├── manifest.json
├── frontend/
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   ├── postcss.config.js
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── panels/
│       │   ├── LoadingPanel/
│       │   ├── WinCondition/
│       │   ├── HunterFocus/
│       │   └── PostGame/
│       ├── shared/
│       ├── hooks/
│       ├── store/
│       │   └── appStore.ts
│       └── lib/
│           ├── platform.ts
│           └── constants.ts
├── backend/
│   ├── server.js
│   ├── package.json
│   ├── .env.example
│   ├── routes/
│   ├── services/
│   └── data/
│       └── .gitkeep
└── icons/
    ├── icon-32.png
    └── icon-256.png
```

### 2. Manifest File (`manifest.json`)

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
    {
      "event": "game_phase_changed",
      "url": "http://localhost:45679/webhook"
    },
    {
      "event": "game_session_ready",
      "url": "http://localhost:45679/webhook"
    }
  ],
  "backend": {
    "process": "backend/server.js",
    "runtime": "node",
    "port": 45679
  }
}
```

### 3. Frontend Package Configuration (`frontend/package.json`)

```json
{
  "name": "hunter-mode-frontend",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext ts,tsx",
    "test": "vitest"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "zustand": "^4.4.1",
    "axios": "^1.5.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.22",
    "@types/react-dom": "^18.2.7",
    "@typescript-eslint/eslint-plugin": "^6.7.2",
    "@typescript-eslint/parser": "^6.7.2",
    "@vitejs/plugin-react": "^4.0.4",
    "autoprefixer": "^10.4.15",
    "eslint": "^8.50.0",
    "eslint-plugin-react-hooks": "^4.6.0",
    "postcss": "^8.4.29",
    "tailwindcss": "^3.3.3",
    "typescript": "^5.2.2",
    "vite": "^4.4.9",
    "vitest": "^0.34.4"
  }
}
```

### 4. Vite Configuration (`frontend/vite.config.ts`)

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: "../dist/frontend",
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    strictPort: false,
  },
});
```

### 5. TypeScript Configuration (`frontend/tsconfig.json`)

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### 6. TailwindCSS Configuration (`frontend/tailwind.config.js`)

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        hm: {
          bg: "rgba(8, 8, 14, 0.88)",
          border: "rgba(255, 255, 255, 0.08)",
          primary: "rgba(255, 255, 255, 0.92)",
          secondary: "rgba(255, 255, 255, 0.55)",
          muted: "rgba(255, 255, 255, 0.30)",
          threat1: "#4CAF50",
          threat2: "#8BC34A",
          threat3: "#FF9800",
          threat4: "#FF5722",
          threat5: "#F44336",
          favorable: "#4FC3F7",
          even: "#90A4AE",
          unfavorable: "#FF7043",
          win: "#4CAF50",
          loss: "#EF5350",
        },
      },
      fontFamily: {
        sans: ["Inter", "Segoe UI", "sans-serif"],
      },
    },
  },
  plugins: [],
};
```

### 7. PostCSS Configuration (`frontend/postcss.config.js`)

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

### 8. Frontend Entry HTML (`frontend/index.html`)

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Hunter Mode</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

### 9. Backend Package Configuration (`backend/package.json`)

```json
{
  "name": "hunter-mode-backend",
  "version": "1.0.0",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.2",
    "axios": "^1.5.0",
    "lowdb": "^6.0.1",
    "dotenv": "^16.3.1",
    "cors": "^2.8.5"
  },
  "devDependencies": {
    "nodemon": "^3.0.1",
    "jest": "^29.7.0",
    "supertest": "^6.3.3"
  }
}
```

### 10. Backend Environment Template (`backend/.env.example`)

```bash
# Riot Games API Key
# Get yours at: https://developer.riotgames.com/
HUNTER_MODE_RIOT_API_KEY=RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# Server configuration
PORT=45679
NODE_ENV=development

# Cache settings
CACHE_TTL_SUMMONER=10
CACHE_TTL_MATCH=60
CACHE_TTL_MASTERY=10
```

### 11. Basic Backend Server (`backend/server.js`)

```javascript
require("dotenv").config();
const express = require("express");
const cors = require("cors");
const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// Health check
app.get("/health", (req, res) => {
  res.json({ status: "ok", service: "hunter-mode-backend" });
});

// Routes will be added in subsequent tasks
// app.use("/enemies", require("./routes/enemies"));
// app.use("/win-condition", require("./routes/wincondition"));
// app.use("/hunter", require("./routes/hunter"));
// app.use("/post-game", require("./routes/postgame"));
// app.use("/webhook", require("./routes/webhook"));

const PORT = process.env.PORT || 45679;
app.listen(PORT, "127.0.0.1", () => {
  console.log(`[Hunter Mode Backend] Running on port ${PORT}`);
});

module.exports = app;
```

### 12. Initial Constants File (`frontend/src/lib/constants.ts`)

```typescript
export const API_BASE_URL = "http://localhost:45679";
export const DATA_DRAGON_BASE = "https://ddragon.leagueoflegends.com/cdn";
export const DATA_DRAGON_VERSION = "13.20.1"; // Update to latest

export const REGIONS = {
  na1: { name: "North America", regional: "americas" },
  euw1: { name: "Europe West", regional: "europe" },
  eun1: { name: "Europe Nordic & East", regional: "europe" },
  kr: { name: "Korea", regional: "asia" },
  br1: { name: "Brazil", regional: "americas" },
  la1: { name: "Latin America North", regional: "americas" },
  la2: { name: "Latin America South", regional: "americas" },
  oc1: { name: "Oceania", regional: "sea" },
  tr1: { name: "Turkey", regional: "europe" },
  ru: { name: "Russia", regional: "europe" },
} as const;

export type RegionCode = keyof typeof REGIONS;
```

### 13. README Documentation

Create `README.md` in root:

````markdown
# Hunter Mode

A League of Legends coaching overlay app built on the LOLOverlay platform.

## Features

- **Loading Screen Panel**: Enemy player analysis with win rates, KDA, build paths
- **Win Condition Advisor**: Draft composition analysis and power timelines
- **Hunter Focus**: Deep-dive enemy player insights during the game
- **Post-Game Review**: Performance comparison vs historical averages

## Development Setup

### Prerequisites

- Node.js 20+
- LOLOverlay platform installed
- Riot Games API key ([Get one here](https://developer.riotgames.com/))

### Installation

1. Clone this repository
2. Install frontend dependencies:
   ```bash
   cd frontend
   npm install
   ```
````

3. Install backend dependencies:
   ```bash
   cd backend
   npm install
   ```
4. Copy backend/.env.example to backend/.env and add your Riot API key

### Running in Development

1. Start the backend server:
   ```bash
   cd backend
   npm run dev
   ```
2. Start the frontend dev server:
   ```bash
   cd frontend
   npm run dev
   ```
3. Load the app in LOLOverlay:
   ```bash
   loloverlay dev /path/to/hunter-mode
   ```

### Building for Production

1. Build frontend:
   ```bash
   cd frontend
   npm run build
   ```
2. Package as .lolapp:
   ```bash
   loloverlay package /path/to/hunter-mode
   ```

## Project Structure

```
hunter-mode/
├── manifest.json          # LOLOverlay app manifest
├── frontend/              # React + TypeScript UI
├── backend/               # Node.js + Express API server
└── icons/                 # App icons
```

## License

MIT

````

---

## Integration Points

### For Task 02 (Riot API Client):
- Backend server is running on port 45679
- .env file configured with API key
- Folder structure ready for services/riotApi.js

### For Task 08 (Frontend Platform Integration):
- Frontend structure ready for lib/platform.ts
- Manifest declares all required permissions
- Window IDs defined for all panels

---

## Testing Requirements

### Verification Steps

1. **Folder structure exists**:
   ```bash
   tree hunter-mode -L 3
````

2. **Frontend dependencies install**:
   ```bash
   cd frontend && npm install
   ```
3. **Backend dependencies install**:
   ```bash
   cd backend && npm install
   ```
4. **Backend server runs**:
   ```bash
   cd backend && npm start
   # Should log: [Hunter Mode Backend] Running on port 45679
   ```
5. **Frontend dev server runs**:
   ```bash
   cd frontend && npm run dev
   # Should log: VITE v4.x.x ready at http://localhost:3000
   ```
6. **Health check endpoint works**:
   ```bash
   curl http://localhost:45679/health
   # Should return: {"status":"ok","service":"hunter-mode-backend"}
   ```
7. **Manifest is valid JSON**:
   ```bash
   cat manifest.json | jq .
   # Should parse without errors
   ```

---

## Acceptance Criteria

✅ **Complete when:**

1. All folder structure is created
2. manifest.json exists with all 4 windows, 8 hotkeys, 2 webhooks defined
3. frontend/package.json has all dependencies
4. backend/package.json has all dependencies
5. Vite config is set up for React + TypeScript
6. TailwindCSS is configured with Hunter Mode color tokens
7. Backend server starts successfully
8. Frontend dev server starts successfully
9. .env.example exists with all required variables
10. README.md has setup instructions
11. All folders are created (even empty ones with .gitkeep)
12. TypeScript compiles without errors
13. No runtime errors on initial load

---

## Performance Requirements

- Frontend dev server starts in <5 seconds
- Backend server starts in <1 second
- npm install completes in <60 seconds (with fast internet)

---

## Files to Create

### New Files:

- `manifest.json`
- `README.md`
- `frontend/package.json`
- `frontend/vite.config.ts`
- `frontend/tsconfig.json`
- `frontend/tsconfig.node.json`
- `frontend/tailwind.config.js`
- `frontend/postcss.config.js`
- `frontend/index.html`
- `frontend/src/lib/constants.ts`
- `backend/package.json`
- `backend/server.js`
- `backend/.env.example`
- `backend/data/.gitkeep`
- All empty folder structure

---

## Expected Time: 3-4 hours

## Difficulty: Medium (Configuration-heavy but straightforward)
