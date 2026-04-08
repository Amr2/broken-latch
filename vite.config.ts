import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

/**
 * Vite configuration for broken-latch Platform
 *
 * Multi-page setup:
 *  - index.html     → src/main.tsx     → Dev Simulator dashboard (main window)
 *  - overlay.html   → src/overlay.tsx  → Transparent overlay window
 *
 * In dev mode (`tauri dev`):
 *   Vite serves both HTML files from http://localhost:5173.
 *   Tauri's main window loads "/"  (→ index.html).
 *   Tauri's overlay window loads "/overlay.html" (created dynamically in Rust).
 *
 * In production (`tauri build`):
 *   Both files are bundled into dist/ and served from the app bundle.
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
    // Multi-page: both HTML entry points are built into dist/
    rollupOptions: {
      input: {
        main:    "./index.html",
        overlay: "./overlay.html",
      },
    },
  },
});
