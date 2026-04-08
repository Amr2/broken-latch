/**
 * overlay.tsx — Overlay Window React Entry Point
 * ================================================
 * This is the React entry for the transparent fullscreen overlay window.
 * It is loaded by overlay.html which Tauri opens as a second WebviewWindow.
 *
 * The overlay has no background — it is a transparent canvas over the game.
 * The OverlayApp component listens for `phase-changed` events from Rust and
 * swaps in the appropriate stage component.
 *
 * DEVELOPER NOTE:
 *   When building your own app on this platform, you do NOT modify this file.
 *   Your app widgets are rendered as absolutely-positioned panels inside the
 *   overlay by the widget system (Task 06). This file is the platform's own
 *   internal overlay renderer.
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import OverlayApp from './components/overlay/OverlayApp';

ReactDOM.createRoot(document.getElementById('overlay-root')!).render(
  <React.StrictMode>
    <OverlayApp />
  </React.StrictMode>,
);
