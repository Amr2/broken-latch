/**
 * Loading.tsx — Game Loading Screen Stage Overlay
 * =================================================
 * Shown while League of Legends is loading into a match.
 *
 * WIDGET LAYOUT (1920×1080 reference)
 * ─────────────────────────────────────
 *  [BOTTOM CENTER]  Loading tip banner  — champion lore + match info
 *
 * CAPABILITIES DEMONSTRATED
 * ─────────────────────────
 *  ■ Bottom-anchored widget (non-intrusive during loading)
 *  ■ Minimal interactive region — most of overlay stays click-through
 *  ■ Game mode detection from session data (game::session)
 *
 * PRODUCTION USAGE
 * ────────────────
 *  game::session is populated during CHAMP_SELECT when the LCU sends session data.
 *  By the time LOADING fires, your app has access to:
 *    - Local player's champion_id and champion_name
 *    - game_mode, map_id
 *    - Full ally and enemy team compositions
 *  This widget uses that data to show a relevant loading tip.
 *
 * DEVELOPER GUIDE — reading session data (planned, Task 04 + 09 SDK):
 *  ```js
 *  const session = await BrokenLatch.game.getSession();
 *  // {
 *  //   localPlayer: { championName: 'Jinx', team_id: 100 },
 *  //   allyTeam: [...],
 *  //   enemyTeam: [...],
 *  //   gameMode: 'CLASSIC',
 *  //   mapId: 11
 *  // }
 *  ```
 */

import { useState, useEffect } from 'react';
import '../overlay/OverlayApp.css';
import './stages.css';

// Mock data — in production from game::session populated during CHAMP_SELECT
const MOCK_SESSION = {
  champion:  'Caitlyn',
  gameMode:  'Ranked Solo/Duo',
  mapName:   "Summoner's Rift",
  allies:    ['Renekton', 'Vi', 'Viktor', 'Caitlyn', 'Lulu'],
  enemies:   ['Darius', 'Hecarim', 'Zed', 'Jinx', 'Thresh'],
};

const LOADING_TIPS: string[] = [
  "Caitlyn's Headshot (passive) charges faster when standing in brush.",
  "Use Yordle Snap Trap to zone enemies off objectives during teamfights.",
  "Piltover Peacemaker pierces through units — use it to poke in lane.",
  "Caitlyn's ultimate is blocked by other champion bodies — position carefully.",
  "The 90 Caliber Net can be used as a short escape blink over terrain.",
];

export default function Loading() {
  const [tipIndex, setTipIndex]   = useState(0);
  const [progress, setProgress]   = useState(0);

  // Cycle tips every 8 seconds
  useEffect(() => {
    const id = setInterval(() => setTipIndex(i => (i + 1) % LOADING_TIPS.length), 8000);
    return () => clearInterval(id);
  }, []);

  // Mock loading progress
  useEffect(() => {
    if (progress >= 100) return;
    const id = setTimeout(() => setProgress(p => Math.min(100, p + Math.random() * 4 + 1)), 400);
    return () => clearTimeout(id);
  }, [progress]);

  return (
    <>
      {/* Widget: Loading tip banner (bottom center) */}
      <div
        className="widget-panel"
        style={{
          bottom: 80,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 900,
        }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>⏳ Loading — {MOCK_SESSION.champion}  •  {MOCK_SESSION.gameMode}  •  {MOCK_SESSION.mapName}</span>
          <span style={{ color: '#6366f1', fontFamily: 'monospace' }}>{Math.floor(progress)}%</span>
        </div>
        <div className="widget-body loading-body">
          {/* Match teams */}
          <div className="loading-teams">
            <div className="loading-team blue">
              {MOCK_SESSION.allies.map(c => (
                <span key={c} className={`loading-champ ${c === MOCK_SESSION.champion ? 'is-you' : ''}`}>{c}</span>
              ))}
            </div>
            <span className="loading-vs">VS</span>
            <div className="loading-team red">
              {MOCK_SESSION.enemies.map(c => (
                <span key={c} className="loading-champ">{c}</span>
              ))}
            </div>
          </div>

          {/* Progress bar */}
          <div className="loading-progress-bar">
            <div
              className="loading-progress-fill"
              style={{ width: `${progress}%` }}
            />
          </div>

          {/* Tip */}
          <div className="loading-tip">
            <span className="tip-icon">💡</span>
            <span className="tip-text">{LOADING_TIPS[tipIndex]}</span>
          </div>

          <div className="dev-note" style={{ marginTop: 8 }}>
            📖 Session data from <code>game::session</code> (Task 04)  •
            Tips pulled from <code>app_storage</code> champion knowledge base
          </div>
        </div>
      </div>
    </>
  );
}
