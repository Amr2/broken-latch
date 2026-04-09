import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

/**
 * Vite configuration for broken-latch Platform
 *
 * Multi-page setup — one entry per window/overlay:
 *  - index.html          → src/main.tsx                       Dev Simulator dashboard
 *  - overlay.html        → src/overlay.tsx                    Legacy overlay (kept for reference)
 *  - launching.html      → src/phase-windows/launching.tsx    LAUNCHING phase window
 *  - in-lobby.html       → src/phase-windows/in-lobby.tsx     IN_LOBBY phase window
 *  - champ-select.html   → src/phase-windows/champ-select.tsx CHAMP_SELECT phase window
 *  - loading.html        → src/phase-windows/loading.tsx      LOADING phase window
 *  - ingame-overlay.html → src/phase-windows/ingame-overlay.tsx IN_GAME overlay
 *  - end-game.html       → src/phase-windows/end-game.tsx     END_GAME phase window
 *
 * To add a new phase window:
 *   1. Create the HTML file at the project root (e.g. my-phase.html)
 *   2. Create the TSX entry at src/phase-windows/my-phase.tsx
 *   3. Add the entry to rollupOptions.input below
 *   4. Reference the HTML filename in src-tauri/resources/phases.json
 */
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        main:           "./index.html",
        overlay:        "./overlay.html",
        launching:      "./launching.html",
        inLobby:        "./in-lobby.html",
        champSelect:    "./champ-select.html",
        loading:        "./loading.html",
        ingameOverlay:  "./ingame-overlay.html",
        endGame:        "./end-game.html",
      },
    },
  },
});
