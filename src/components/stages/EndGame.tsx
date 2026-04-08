/**
 * EndGame.tsx — Post-Game Results Stage Overlay
 * ===============================================
 * Shown after the match ends (Victory or Defeat screen).
 *
 * WIDGET LAYOUT (1920×1080 reference)
 * ─────────────────────────────────────
 *  [CENTER]  Full match results panel — large, dismissible
 *
 * CAPABILITIES DEMONSTRATED
 * ─────────────────────────
 *  ■ Large centered interactive widget
 *  ■ Webhook integration pattern — POST match data to an external service
 *  ■ App storage write — persist match history for next session
 *  ■ Platform event: app can emit "match-recorded" webhook
 *
 * PRODUCTION USAGE
 * ────────────────
 *  When the END_GAME phase fires, game::session still holds the full session.
 *  After the game ends, the Live Game Data API stops responding. Instead, the
 *  platform fetches the match result from the LCU:
 *    GET /lol-end-of-game/v1/eog-stats-block
 *
 *  Apps can also subscribe to the "match-ended" Tauri event which fires
 *  immediately when the phase transitions to END_GAME.
 *
 *  WEBHOOK PATTERN — notifying external services (planned, Task 08):
 *  ```js
 *  // Register a webhook for match end events
 *  await invoke('register_webhook', {
 *    event: 'match-ended',
 *    url:   'https://your-server.com/hook/match'
 *  });
 *  // The platform will POST the match payload to your URL after each game
 *  ```
 *
 *  APP STORAGE PATTERN — persisting match history:
 *  ```js
 *  await BrokenLatch.storage.set('last_match', JSON.stringify(matchResult));
 *  const history = JSON.parse(await BrokenLatch.storage.get('match_history') ?? '[]');
 *  history.unshift(matchResult);
 *  await BrokenLatch.storage.set('match_history', JSON.stringify(history.slice(0, 20)));
 *  ```
 */

import { useState } from 'react';
import '../overlay/OverlayApp.css';
import './stages.css';

// Mock match result — in production from LCU /lol-end-of-game/v1/eog-stats-block
const MOCK_RESULT = {
  victory:       true,
  champion:      'Caitlyn',
  duration:      '28:14',
  kills:         8,
  deaths:        2,
  assists:       12,
  cs:            218,
  gold:          14200,
  visionScore:   28,
  damage:        32041,
  kp:            71,    // kill participation %
  mvp:           true,
  items:         ['Kraken Slayer', 'Galeforce', 'Phantom Dancer', 'Infinity Edge', 'Lord Dominik'],
  topDamage:     true,
  allies: [
    { name: 'Renekton', kda: '4/3/8',  cs: 189, grade: 'A'  },
    { name: 'Vi',       kda: '6/4/14', cs: 112, grade: 'A+' },
    { name: 'Viktor',   kda: '7/5/11', cs: 263, grade: 'B+' },
    { name: 'Caitlyn', kda:  '8/2/12', cs: 218, grade: 'S'  },
    { name: 'Lulu',    kda:  '0/3/18', cs: 28,  grade: 'A'  },
  ],
};

export default function EndGame() {
  const [dismissed, setDismissed] = useState(false);
  const [webhookSent, setWebhookSent] = useState(false);

  const handleWebhook = () => {
    // In production: invoke('trigger_webhook', { event: 'match-ended', payload: matchResult })
    setWebhookSent(true);
  };

  if (dismissed) return null;

  return (
    <div
      className="widget-panel"
      style={{
        top:       '50%',
        left:      '50%',
        transform: 'translate(-50%, -50%)',
        width:     700,
      }}
    >
      <span className="demo-badge">DEMO</span>

      {/* Header */}
      <div
        className="widget-header"
        style={{
          background: MOCK_RESULT.victory
            ? 'rgba(34,197,94,0.15)'
            : 'rgba(239,68,68,0.15)',
          borderColor: MOCK_RESULT.victory
            ? 'rgba(34,197,94,0.3)'
            : 'rgba(239,68,68,0.3)',
        }}
      >
        <span style={{ fontSize: 14, fontWeight: 800, letterSpacing: 2, textTransform: 'uppercase',
          color: MOCK_RESULT.victory ? '#22c55e' : '#ef4444' }}>
          {MOCK_RESULT.victory ? '🏆 VICTORY' : '💀 DEFEAT'}
        </span>
        <div style={{ display: 'flex', gap: 8 }}>
          <span style={{ color: '#888', fontSize: 11 }}>{MOCK_RESULT.duration}</span>
          <button className="dismiss-btn" onClick={() => setDismissed(true)}>✕</button>
        </div>
      </div>

      <div className="widget-body">
        {/* Hero stats */}
        <div className="endgame-hero">
          <div className="hero-champ">{MOCK_RESULT.champion}</div>
          <div className="hero-kda">
            <span className="kda-num green">{MOCK_RESULT.kills}</span>
            <span className="kda-sep">/</span>
            <span className="kda-num red">{MOCK_RESULT.deaths}</span>
            <span className="kda-sep">/</span>
            <span className="kda-num blue">{MOCK_RESULT.assists}</span>
          </div>
          {MOCK_RESULT.mvp && <div className="mvp-badge">⭐ MVP</div>}
        </div>

        {/* Stat row */}
        <div className="endgame-stats">
          <StatCell val={String(MOCK_RESULT.cs)}          label="CS" />
          <StatCell val={`${(MOCK_RESULT.gold/1000).toFixed(1)}k`} label="Gold" />
          <StatCell val={String(MOCK_RESULT.visionScore)} label="Vision" />
          <StatCell val={`${(MOCK_RESULT.damage/1000).toFixed(1)}k`} label="Damage" />
          <StatCell val={`${MOCK_RESULT.kp}%`}           label="KP" />
        </div>

        {/* Badges */}
        <div className="badge-row">
          {MOCK_RESULT.topDamage && <span className="badge orange">🔥 Top Damage</span>}
          {MOCK_RESULT.mvp       && <span className="badge gold">⭐ MVP</span>}
          <span className="badge blue">🎯 {MOCK_RESULT.kp}% Kill Participation</span>
        </div>

        {/* Team scoreboard */}
        <div className="section-label" style={{ marginTop: 10 }}>Team Scoreboard</div>
        <table className="scoreboard">
          <thead>
            <tr>
              <th>Champion</th>
              <th>KDA</th>
              <th>CS</th>
              <th>Grade</th>
            </tr>
          </thead>
          <tbody>
            {MOCK_RESULT.allies.map(p => (
              <tr key={p.name} className={p.name === MOCK_RESULT.champion ? 'highlight-row' : ''}>
                <td>{p.name === MOCK_RESULT.champion ? `→ ${p.name}` : p.name}</td>
                <td style={{ fontFamily: 'monospace', fontSize: 11 }}>{p.kda}</td>
                <td>{p.cs}</td>
                <td style={{ color: gradeColor(p.grade), fontWeight: 700 }}>{p.grade}</td>
              </tr>
            ))}
          </tbody>
        </table>

        {/* Platform capability: Webhook */}
        <div className="endgame-actions">
          <button
            className={`ctrl-btn ${webhookSent ? 'green' : ''}`}
            onClick={handleWebhook}
          >
            {webhookSent ? '✓ Webhook sent' : '📤 Send match data (webhook demo)'}
          </button>
          <button className="ctrl-btn red" onClick={() => setDismissed(true)}>
            Close Overlay
          </button>
        </div>

        <div className="dev-note">
          💾 Match data persisted to <code>app_storage</code> table  •
          Webhook registered via <code>register_webhook()</code> (Task 08)
        </div>
      </div>
    </div>
  );
}

function StatCell({ val, label }: { val: string; label: string }) {
  return (
    <div className="stat-cell">
      <div className="stat-cell-val">{val}</div>
      <div className="stat-cell-label">{label}</div>
    </div>
  );
}

function gradeColor(grade: string) {
  if (grade.startsWith('S')) return '#f59e0b';
  if (grade.startsWith('A')) return '#22c55e';
  return '#888';
}
