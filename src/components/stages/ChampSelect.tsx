/**
 * ChampSelect.tsx — Champion Select Stage Overlay
 * =================================================
 * Shown during the champion pick/ban phase.
 *
 * WIDGET LAYOUT (1920×1080 reference)
 * ─────────────────────────────────────
 *  [LEFT SIDE]   Team Composition panel  — your team's picks and roles
 *  [RIGHT SIDE]  Enemy Bans panel        — banned champions list
 *  [TOP CENTER]  Phase Timer             — pick/ban countdown
 *
 * CAPABILITIES DEMONSTRATED
 * ─────────────────────────
 *  ■ Multiple non-overlapping interactive regions (two Rects registered)
 *  ■ Hotkey integration pattern — champion swap suggestion shortcut
 *  ■ LCU WebSocket event pattern (champ-select updates in real-time)
 *  ■ Data flows: LCU → game::lcu → Tauri event → widget update
 *
 * PRODUCTION USAGE
 * ────────────────
 *  game::lcu connects to the LCU WebSocket at ws://127.0.0.1:2999.
 *  It subscribes to "OnJsonApiEvent_lol-champ-select_v1_session" and emits
 *  a Tauri event "champ-select-updated" with the session payload.
 *  This widget listens for that event and re-renders the pick/ban state.
 *
 * DEVELOPER GUIDE — Champ Select event subscription (planned):
 *  ```js
 *  await listen('champ-select-updated', (event) => {
 *    const session = event.payload; // ChampSelectSession type
 *    updateMyTeam(session.myTeam);
 *    updateTheirTeam(session.theirTeam);
 *  });
 *  ```
 *
 * HOTKEY EXAMPLE (planned, Task 05):
 *  ```js
 *  // Register a hotkey — stored in hotkeys DB table
 *  await invoke('register_hotkey', {
 *    hotkey_id: 'suggest_swap',
 *    keys: 'Ctrl+Shift+S'
 *  });
 *  // Listen for the hotkey event
 *  await listen('hotkey-suggest_swap', () => showSwapSuggestion());
 *  ```
 */

import { useState, useEffect } from 'react';
import '../overlay/OverlayApp.css';
import './stages.css';

// Mock champ select state — in production from LCU WebSocket events
const MOCK_SESSION = {
  phase: 'PICK',
  timer: 27,
  myTeam: [
    { slot: 0, champion: 'Renekton',  role: 'TOP',    locked: true,  isYou: false },
    { slot: 1, champion: 'Vi',        role: 'JG',     locked: true,  isYou: false },
    { slot: 2, champion: null,         role: 'MID',    locked: false, isYou: false },
    { slot: 3, champion: null,         role: 'BOT',    locked: false, isYou: true  },
    { slot: 4, champion: null,         role: 'SUP',    locked: false, isYou: false },
  ],
  theirBans: ['Zed', 'Yone', 'Yasuo', 'Katarina', 'Leblanc'],
  suggestedPick: 'Caitlyn',
};

const ROLE_ICONS: Record<string, string> = {
  TOP: '🗡️', JG: '🌲', MID: '⚡', BOT: '🏹', SUP: '🛡️',
};

export default function ChampSelect() {
  const [timer, setTimer] = useState(MOCK_SESSION.timer);

  useEffect(() => {
    const id = setInterval(() => setTimer(t => Math.max(0, t - 1)), 1000);
    return () => clearInterval(id);
  }, []);

  const timerColor = timer > 10 ? '#22c55e' : timer > 5 ? '#f59e0b' : '#ef4444';

  return (
    <>
      {/* Widget 1: Team composition (left side) */}
      <div
        className="widget-panel"
        style={{ top: 150, left: 20, width: 260 }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>⚔️ Your Team</span>
          <span style={{ color: '#6366f1', fontSize: 10 }}>CHAMP SELECT</span>
        </div>
        <div className="widget-body">
          {MOCK_SESSION.myTeam.map(player => (
            <div
              key={player.slot}
              className={`cs-player ${player.isYou ? 'is-you' : ''} ${player.locked ? 'locked' : ''}`}
            >
              <span className="cs-role">{ROLE_ICONS[player.role]}</span>
              <div className="cs-champ-info">
                <span className="cs-champ-name">
                  {player.champion
                    ? player.champion
                    : player.isYou
                    ? `→ You — Picking…`
                    : 'Picking…'
                  }
                </span>
                {player.isYou && !player.champion && (
                  <span className="cs-suggestion">💡 Suggested: {MOCK_SESSION.suggestedPick}</span>
                )}
              </div>
              {player.locked && <span className="cs-locked">✓</span>}
            </div>
          ))}

          <div className="dev-note">
            📡 Updates via <code>champ-select-updated</code> Tauri event<br />
            (LCU WebSocket → game::lcu → emit)
          </div>
        </div>
      </div>

      {/* Widget 2: Enemy bans (right side) */}
      <div
        className="widget-panel"
        style={{ top: 150, right: 20, width: 220 }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>🚫 Enemy Bans</span>
        </div>
        <div className="widget-body">
          <div className="section-label">Banned Champions</div>
          <div className="bans-grid">
            {MOCK_SESSION.theirBans.map(champ => (
              <div key={champ} className="ban-chip">
                <span>🚫</span> {champ}
              </div>
            ))}
          </div>

          <div className="section-label" style={{ marginTop: 10 }}>Hotkey</div>
          <div className="hotkey-demo">
            <span className="key">Ctrl</span><span className="key-sep">+</span>
            <span className="key">Shift</span><span className="key-sep">+</span>
            <span className="key">S</span>
            <span className="key-label"> = Suggest Swap</span>
          </div>
          <div className="dev-note">
            ⌨️ Registered via <code>register_hotkey()</code><br />
            Stored in <code>hotkeys</code> DB table
          </div>
        </div>
      </div>

      {/* Widget 3: Timer (top center) */}
      <div
        className="widget-panel phase-timer"
        style={{
          top: 20,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 180,
          textAlign: 'center',
          pointer_events: 'auto',
        } as React.CSSProperties}
      >
        <div className="widget-header" style={{ justifyContent: 'center' }}>
          <span>⚔️ {MOCK_SESSION.phase} PHASE</span>
        </div>
        <div className="widget-body" style={{ padding: '8px 0' }}>
          <div className="timer-display" style={{ color: timerColor }}>
            {String(timer).padStart(2, '0')}
          </div>
          <div className="timer-label">seconds remaining</div>
        </div>
      </div>
    </>
  );
}
