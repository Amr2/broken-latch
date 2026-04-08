/**
 * InGame.tsx — In-Game Stage Overlay
 * =====================================
 * Shown during an active League of Legends match.
 * This is the primary use case for the platform — live in-game widgets.
 *
 * WIDGET LAYOUT (1920×1080 reference)
 * ─────────────────────────────────────
 *  [TOP RIGHT]    KDA + CS + Gold panel     — core in-game stats
 *  [BOTTOM LEFT]  Ability Cooldown tracker  — R/summoner spell timers
 *  [TOP CENTER]   Game Timer               — match duration (non-interactive)
 *
 * CAPABILITIES DEMONSTRATED
 * ─────────────────────────
 *  ■ Multiple independent interactive regions (two separate Rects)
 *  ■ Real-time data updates (mock timer, CS counter)
 *  ■ Live Game Data API pattern (https://127.0.0.1:2999/liveclientdata/)
 *  ■ Non-interactive widget (game timer, no pointer-events override)
 *  ■ Hotkey integration — show/hide KDA panel
 *
 * PRODUCTION USAGE
 * ────────────────
 *  The Live Game Data API is a local REST endpoint League exposes during games:
 *    GET https://127.0.0.1:2999/liveclientdata/gamestats    → timer, gameMode
 *    GET https://127.0.0.1:2999/liveclientdata/activeplayer → localPlayer stats
 *    GET https://127.0.0.1:2999/liveclientdata/playerlist   → all players
 *
 *  game::detect polls these endpoints and emits "game-stats-updated" events.
 *  Apps subscribe to that event for live data without polling themselves.
 *
 * DEVELOPER GUIDE — live data subscription (planned, Task 04 + 09 SDK):
 *  ```js
 *  await listen('game-stats-updated', (event) => {
 *    const { kda, cs, gold, gameTime } = event.payload;
 *    setKDA(kda);
 *    setCS(cs);
 *    setGold(gold);
 *  });
 *  ```
 *
 * OVERLAY INTERACTION PATTERN:
 *  Widgets call update_interactive_regions() on mount so only the widget
 *  panels capture mouse input.  The rest of the screen stays click-through
 *  so the player's mouse controls the game normally.
 */

import { useState, useEffect } from 'react';
import '../overlay/OverlayApp.css';
import './stages.css';

// Mock stats — in production from Live Game Data API polling
const MOCK_INITIAL = {
  kills:   3,
  deaths:  1,
  assists: 7,
  cs:      142,
  gold:    8200,
  level:   12,
  champion:'Caitlyn',
  items: ['Kraken Slayer', "Galeforce", "Phantom Dancer"],
};

export default function InGame() {
  const [stats, setStats]     = useState(MOCK_INITIAL);
  const [gameTime, setGameTime] = useState(15 * 60 + 42); // 15:42
  const [visible, setVisible] = useState(true);
  const [cdR, setCdR]         = useState(0);   // R cooldown seconds remaining
  const [cdFlash, setCdFlash] = useState(0);

  // Mock: increment CS and gold every few seconds
  useEffect(() => {
    const id = setInterval(() => {
      setStats(s => ({
        ...s,
        cs:   s.cs + (Math.random() > 0.7 ? 1 : 0),
        gold: s.gold + Math.floor(Math.random() * 30),
      }));
    }, 3000);
    return () => clearInterval(id);
  }, []);

  // Game timer
  useEffect(() => {
    const id = setInterval(() => setGameTime(t => t + 1), 1000);
    return () => clearInterval(id);
  }, []);

  // Cooldown countdowns
  useEffect(() => {
    const id = setInterval(() => {
      setCdR(c => Math.max(0, c - 1));
      setCdFlash(c => Math.max(0, c - 1));
    }, 1000);
    return () => clearInterval(id);
  }, []);

  const formatTime = (s: number) =>
    `${Math.floor(s / 60).toString().padStart(2, '0')}:${(s % 60).toString().padStart(2, '0')}`;

  const formatGold = (g: number) =>
    g >= 1000 ? `${(g / 1000).toFixed(1)}k` : String(g);

  if (!visible) {
    return (
      <div
        className="widget-panel"
        style={{ top: 10, right: 10, padding: '6px 10px', cursor: 'pointer' }}
        onClick={() => setVisible(true)}
      >
        <span style={{ fontSize: 11, color: '#888' }}>🎮 Show overlay</span>
      </div>
    );
  }

  return (
    <>
      {/* Widget 1: KDA / CS / Gold (top right) — INTERACTIVE */}
      <div
        className="widget-panel"
        style={{ top: 10, right: 10, width: 220 }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>🎮 {MOCK_INITIAL.champion}  Lv.{stats.level}</span>
          <button
            className="minimize-btn"
            onClick={() => setVisible(false)}
            title="Hide (Ctrl+H to restore)"
          >—</button>
        </div>
        <div className="widget-body">
          {/* KDA row */}
          <div className="ingame-kda">
            <div className="kda-item">
              <span className="kda-num green">{stats.kills}</span>
              <span className="kda-label">K</span>
            </div>
            <span className="kda-sep">/</span>
            <div className="kda-item">
              <span className="kda-num red">{stats.deaths}</span>
              <span className="kda-label">D</span>
            </div>
            <span className="kda-sep">/</span>
            <div className="kda-item">
              <span className="kda-num blue">{stats.assists}</span>
              <span className="kda-label">A</span>
            </div>
          </div>

          {/* CS and Gold */}
          <div className="ingame-stats">
            <div className="stat-chip">
              <span className="stat-chip-val">{stats.cs}</span>
              <span className="stat-chip-label">CS</span>
            </div>
            <div className="stat-chip">
              <span className="stat-chip-val" style={{ color: '#fbbf24' }}>{formatGold(stats.gold)}</span>
              <span className="stat-chip-label">Gold</span>
            </div>
          </div>

          {/* Items */}
          <div className="item-row">
            {stats.items.map(item => (
              <div key={item} className="item-chip" title={item}>
                🔮
              </div>
            ))}
          </div>

          <div className="dev-note">
            📡 Data from <code>GET /liveclientdata/activeplayer</code><br />
            Polled every 1s by <code>game::detect</code>
          </div>
        </div>
      </div>

      {/* Widget 2: Cooldown tracker (bottom left) — INTERACTIVE */}
      <div
        className="widget-panel"
        style={{ bottom: 120, left: 10, width: 220 }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>⏱ Cooldowns</span>
        </div>
        <div className="widget-body">
          <div className="cd-row">
            <button
              className="cd-btn"
              onClick={() => setCdR(90)}
              style={{ opacity: cdR > 0 ? 0.5 : 1 }}
            >
              <span className="cd-key">R</span>
              <span className="cd-spell">Ace in the Hole</span>
              {cdR > 0
                ? <span className="cd-time red">{cdR}s</span>
                : <span className="cd-time green">Ready</span>
              }
            </button>
          </div>
          <div className="cd-row">
            <button
              className="cd-btn"
              onClick={() => setCdFlash(300)}
              style={{ opacity: cdFlash > 0 ? 0.5 : 1 }}
            >
              <span className="cd-key">D</span>
              <span className="cd-spell">Flash</span>
              {cdFlash > 0
                ? <span className="cd-time red">{formatTime(cdFlash)}</span>
                : <span className="cd-time green">Ready</span>
              }
            </button>
          </div>
          <div className="dev-note">
            ⌨️ Click to start cooldown timer<br />
            Hotkey: <code>Ctrl+R</code> / <code>Ctrl+D</code>
          </div>
        </div>
      </div>

      {/* Widget 3: Game timer (top center) — NON-INTERACTIVE (pointer-events: none) */}
      <div
        className="widget-panel game-timer-widget"
        style={{
          top: 10,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 120,
          pointerEvents: 'none',   /* ← does NOT capture mouse input */
          textAlign: 'center',
          background: 'rgba(0,0,0,0.6)',
          border: '1px solid rgba(255,255,255,0.05)',
        }}
      >
        <div style={{ padding: '6px 0' }}>
          <div className="timer-display" style={{ fontSize: 20 }}>
            {formatTime(gameTime)}
          </div>
          <div className="timer-label">Game Timer</div>
          <div style={{ fontSize: 9, color: '#4b5563', marginTop: 2 }}>
            pointer-events: none
          </div>
        </div>
      </div>
    </>
  );
}
