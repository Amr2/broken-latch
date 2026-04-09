/**
 * OverlayApp.tsx — Transparent Overlay Root Component
 * =====================================================
 * This is the top-level React component running inside the transparent
 * fullscreen overlay window.
 *
 * It does one thing: listens for the `phase-changed` Tauri event emitted by
 * `simulate_phase()` (or, in production, by the game lifecycle detector),
 * and renders the appropriate stage component for the current phase.
 *
 * ┌─────────────────────────────────────────────────────────────────────┐
 * │  HOW THE EVENT FLOW WORKS                                           │
 * │                                                                     │
 * │  Rust simulate_phase()                                              │
 * │    └→ app_handle.emit("phase-changed", "IN_GAME")                  │
 * │         └→ [this window] listen("phase-changed") fires             │
 * │              └→ setPhase("IN_GAME")                                 │
 * │                   └→ <InGame /> renders                             │
 * │                                                                     │
 * │  The event is also received by the DevDashboard window for logging. │
 * └─────────────────────────────────────────────────────────────────────┘
 *
 * DEVELOPER NOTE (building an app on this platform):
 *   You do NOT modify this file.  Your app widgets are injected into the
 *   overlay by the widget system (Task 06) as absolutely-positioned panels.
 *   This file is the platform's internal overlay frame.
 *
 * Each stage component:
 *   - Uses `position: absolute` to place widgets at specific screen coordinates
 *   - Sets `pointer-events: auto` only on its interactive widget panels
 *   - Everything else remains `pointer-events: none` (click-through to game)
 */

import { useState, useEffect } from 'react';
import { listen }              from '@tauri-apps/api/event';
import { invoke }              from '@tauri-apps/api/core';

import Launching   from '../stages/Launching';
import InLobby     from '../stages/InLobby';
import ChampSelect from '../stages/ChampSelect';
import Loading     from '../stages/Loading';
import InGame      from '../stages/InGame';
import EndGame     from '../stages/EndGame';

import './OverlayApp.css';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type GamePhase =
  | 'NOT_RUNNING'
  | 'LAUNCHING'
  | 'IN_LOBBY'
  | 'CHAMP_SELECT'
  | 'LOADING'
  | 'IN_GAME'
  | 'END_GAME';

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function OverlayApp() {
  const [phase, setPhase] = useState<GamePhase>('NOT_RUNNING');

  useEffect(() => {
    // Sync current phase on mount — handles the case where simulate_phase()
    // was called before this window finished loading and missed the event.
    invoke<string>('get_current_phase')
      .then(p => setPhase(p as GamePhase))
      .catch(() => {}); // ignore if backend not ready yet

    /**
     * Subscribe to the `phase-changed` event.
     *
     * The Rust backend emits this via:
     *   app_handle.emit("phase-changed", phase_string)
     *
     * In production, this event fires whenever the game lifecycle detector
     * (game::detect + game::lcu) senses a phase transition.
     *
     * In demo mode, the DevDashboard calls simulate_phase() which triggers
     * the same event path.
     */
    const unlistenPromise = listen<string>('phase-changed', (event) => {
      setPhase(event.payload as GamePhase);
    });

    // Cleanup: unsubscribe when the component unmounts
    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

  return (
    /**
     * overlay-root:
     *   - 100vw × 100vh, position:relative
     *   - pointer-events: none (entire window is click-through by default)
     *   - Individual widgets inside each stage component override with
     *     pointer-events: auto on their own containers
     *   - The Win32 side also calls SetWindowRgn() to define interactive
     *     areas at the OS level (belt-and-suspenders approach)
     */
    <div className="overlay-root" data-phase={phase}>
      {phase === 'LAUNCHING'    && <Launching />}
      {phase === 'IN_LOBBY'     && <InLobby />}
      {phase === 'CHAMP_SELECT' && <ChampSelect />}
      {phase === 'LOADING'      && <Loading />}
      {phase === 'IN_GAME'      && <InGame />}
      {phase === 'END_GAME'     && <EndGame />}
      {/* NOT_RUNNING renders nothing — overlay is hidden by Rust anyway */}
    </div>
  );
}
