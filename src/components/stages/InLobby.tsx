/**
 * InLobby.tsx — In Lobby Stage Overlay
 * ======================================
 * Shown when the player is in the League of Legends main client lobby.
 *
 * WIDGET LAYOUT (1920×1080 reference)
 * ─────────────────────────────────────
 *  [RIGHT SIDE]  Quick Stats panel    — rank, win rate, recent matches
 *  [RIGHT SIDE]  Queue Timer panel    — live countdown mock
 *
 * CAPABILITIES DEMONSTRATED
 * ─────────────────────────
 *  ■ Multiple positioned widgets on one stage
 *  ■ Persistent widget positions (in production, loaded from widget_positions DB table)
 *  ■ App storage pattern — per-app key-value data (app_storage DB table)
 *  ■ Interactive regions: two Rect areas on the right edge
 *
 * PRODUCTION USAGE
 * ────────────────
 *  Real lobby apps would:
 *  - Pull rank/stats from Riot API via the platform's HTTP API (Task 08)
 *  - Store user preferences in app_storage: { app_id, key:"preferred_role", value:"ADC" }
 *  - Load saved widget position from widget_positions table on mount
 *  - Register drag handles to let the user reposition panels
 *
 * DEVELOPER GUIDE — app_storage API (planned, Task 09 SDK):
 *  ```js
 *  // Save a value
 *  await BrokenLatch.storage.set('preferred_role', 'ADC');
 *
 *  // Read it back
 *  const role = await BrokenLatch.storage.get('preferred_role');
 *  ```
 *  Under the hood: INSERT OR REPLACE INTO app_storage (app_id, key, value, ...)
 */

import { useState, useEffect } from 'react';
import '../overlay/OverlayApp.css';
import './stages.css';

// Mock data — in production this comes from Riot API via platform HTTP API
const MOCK_STATS = {
  summonerName: 'BrokenLatch Dev',
  rank:         'Gold II',
  lp:           67,
  winRate:      54,
  wins:         312,
  losses:       264,
  recentMatches: [
    { result: 'WIN',  champion: 'Jinx',   kda: '8/2/14',  cs: 218, duration: '28:14' },
    { result: 'LOSS', champion: 'Caitlyn',kda: '4/7/6',   cs: 183, duration: '31:02' },
    { result: 'WIN',  champion: 'Jhin',   kda: '12/3/8',  cs: 241, duration: '25:47' },
    { result: 'WIN',  champion: 'Jinx',   kda: '7/4/17',  cs: 197, duration: '33:21' },
    { result: 'LOSS', champion: 'Ezreal', kda: '3/6/9',   cs: 156, duration: '26:38' },
  ],
};

export default function InLobby() {
  const [queueTime, setQueueTime] = useState(0);
  const [inQueue, setInQueue]     = useState(false);

  // Mock queue timer
  useEffect(() => {
    if (!inQueue) return;
    const id = setInterval(() => setQueueTime(t => t + 1), 1000);
    return () => clearInterval(id);
  }, [inQueue]);

  const formatTime = (s: number) =>
    `${Math.floor(s / 60).toString().padStart(2, '0')}:${(s % 60).toString().padStart(2, '0')}`;

  return (
    <>
      {/* Widget 1: Quick Stats (right side) */}
      <div
        className="widget-panel"
        style={{ top: 200, right: 20, width: 260 }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>📊 Quick Stats</span>
          <span style={{ color: '#22c55e', fontSize: 10 }}>● Live</span>
        </div>
        <div className="widget-body">
          {/* Summoner */}
          <div className="stat-name">{MOCK_STATS.summonerName}</div>

          {/* Rank badge */}
          <div className="rank-row">
            <div className="rank-badge">
              <span className="rank-tier">GOLD</span>
              <span className="rank-div">II</span>
            </div>
            <div className="rank-detail">
              <div>{MOCK_STATS.lp} LP</div>
              <div className="stat-dim">{MOCK_STATS.wins}W / {MOCK_STATS.losses}L  •  {MOCK_STATS.winRate}% WR</div>
            </div>
          </div>

          {/* Recent matches */}
          <div className="section-label">Recent Matches</div>
          {MOCK_STATS.recentMatches.map((m, i) => (
            <div key={i} className={`match-row ${m.result === 'WIN' ? 'win' : 'loss'}`}>
              <span className={`match-result ${m.result === 'WIN' ? 'win' : 'loss'}`}>
                {m.result === 'WIN' ? 'W' : 'L'}
              </span>
              <span className="match-champ">{m.champion}</span>
              <span className="match-kda">{m.kda}</span>
              <span className="match-cs">{m.cs} CS</span>
              <span className="match-dur">{m.duration}</span>
            </div>
          ))}

          {/* App storage callout */}
          <div className="dev-note">
            💾 Stats cached in <code>app_storage</code> DB table per app_id
          </div>
        </div>
      </div>

      {/* Widget 2: Queue Timer */}
      <div
        className="widget-panel"
        style={{ top: 680, right: 20, width: 260 }}
      >
        <span className="demo-badge">DEMO</span>
        <div className="widget-header">
          <span>⏱ Queue Status</span>
        </div>
        <div className="widget-body queue-body">
          {inQueue ? (
            <>
              <div className="queue-searching">Searching for match…</div>
              <div className="queue-timer">{formatTime(queueTime)}</div>
              <div className="queue-est">Est. wait: ~3:20</div>
              <button className="queue-btn cancel" onClick={() => { setInQueue(false); setQueueTime(0); }}>
                Cancel Queue
              </button>
            </>
          ) : (
            <>
              <div className="queue-idle">Not in queue</div>
              <button className="queue-btn start" onClick={() => setInQueue(true)}>
                ▶  Start Queue (demo)
              </button>
            </>
          )}
          <div className="dev-note">
            🖱️ These buttons have <code>pointer-events: auto</code> —<br />
            clicks do NOT pass through to the game
          </div>
        </div>
      </div>
    </>
  );
}
