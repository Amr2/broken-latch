/**
 * GameTimerWidget — Live match duration display.
 *
 * Runs in its own overlay window (ingame-timer.html).
 * Window size: 140 × 50 px, centred at top by phases.json.
 * click_through: true → fully transparent to mouse input (Win32 WS_EX_TRANSPARENT).
 *
 * The CSS also sets pointer-events: none as a belt-and-suspenders for the
 * browser rendering layer.
 *
 * Production data source:
 *   GET https://127.0.0.1:2999/liveclientdata/gamestats → { gameTime: number }
 *   Polled by game::detect, emitted as "game-stats-updated".
 */

import { useState, useEffect } from 'react';
import '../overlay/OverlayApp.css';   // box-sizing reset + transparent root
import './widgets.css';

export default function GameTimerWidget() {
  const [gameTime, setGameTime] = useState(15 * 60 + 42); // mock: 15:42

  useEffect(() => {
    const id = setInterval(() => setGameTime(t => t + 1), 1000);
    return () => clearInterval(id);
  }, []);

  const formatTime = (s: number) =>
    `${Math.floor(s / 60).toString().padStart(2, '0')}:${(s % 60).toString().padStart(2, '0')}`;

  return (
    <div className="timer-widget">
      <div className="timer-display">{formatTime(gameTime)}</div>
      <div className="timer-label">Game Timer</div>
    </div>
  );
}
