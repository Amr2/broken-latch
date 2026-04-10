/**
 * KdaWidget — Live KDA / CS / Gold panel.
 *
 * Runs in its own overlay window (ingame-kda.html).
 * Window size: 240 × 200 px, positioned top-right by phases.json.
 * click_through: false → this overlay captures mouse input.
 *
 * Production data source:
 *   GET https://127.0.0.1:2999/liveclientdata/activeplayer
 *   Polled by game::detect, emitted as "game-stats-updated".
 */

import { useState, useEffect } from 'react';
import '../overlay/OverlayApp.css';   // provides .widget-panel background/border/font
import '../stages/stages.css';
import './widgets.css';

const MOCK_INITIAL = {
  kills:    3,
  deaths:   1,
  assists:  7,
  cs:       142,
  gold:     8200,
  level:    12,
  champion: 'Caitlyn',
  items:    ['Kraken Slayer', 'Galeforce', 'Phantom Dancer'],
};

export default function KdaWidget() {
  const [stats, setStats] = useState(MOCK_INITIAL);
  const [visible, setVisible] = useState(true);

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

  const formatGold = (g: number) =>
    g >= 1000 ? `${(g / 1000).toFixed(1)}k` : String(g);

  if (!visible) {
    return (
      <div
        className="widget-panel interactive"
        style={{ top: 0, left: 0, width: '100%', padding: '5px 10px', cursor: 'pointer' }}
        onClick={() => setVisible(true)}
      >
        <span style={{ fontSize: 11, color: '#888' }}>🎮 Show KDA</span>
      </div>
    );
  }

  return (
    <div className="widget-panel interactive" style={{ top: 0, left: 0, width: '100%', height: '100%', borderRadius: 0 }}>
      <span className="demo-badge">DEMO</span>
      <div className="widget-header">
        <span>🎮 {MOCK_INITIAL.champion} Lv.{stats.level}</span>
        <button
          className="minimize-btn"
          onClick={() => setVisible(false)}
          title="Minimize"
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
            <div key={item} className="item-chip" title={item}>🔮</div>
          ))}
        </div>

        <div className="dev-note">
          📡 <code>GET /liveclientdata/activeplayer</code>
        </div>
      </div>
    </div>
  );
}
