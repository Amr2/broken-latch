/**
 * CooldownWidget — Ability + summoner spell cooldown tracker.
 *
 * Runs in its own overlay window (ingame-cooldowns.html).
 * Window size: 240 × 180 px, positioned bottom-left by phases.json.
 * click_through: false → this overlay captures mouse input.
 *
 * Usage: click a spell button when you cast it to start the cooldown timer.
 * Developers can also call getCurrentWindow().startDragging() from any element
 * or use the platform-injected drag edge at the top of the window.
 */

import { useState, useEffect } from 'react';
import '../stages/stages.css';
import './widgets.css';

export default function CooldownWidget() {
  const [cdR,     setCdR]     = useState(0);
  const [cdFlash, setCdFlash] = useState(0);

  // Count down every second
  useEffect(() => {
    const id = setInterval(() => {
      setCdR(c     => Math.max(0, c - 1));
      setCdFlash(c => Math.max(0, c - 1));
    }, 1000);
    return () => clearInterval(id);
  }, []);

  const formatTime = (s: number) =>
    s >= 60
      ? `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`
      : `${s}s`;

  return (
    <div className="widget-panel interactive" style={{ width: '100%', height: '100%', borderRadius: 0 }}>
      <span className="demo-badge">DEMO</span>
      <div className="widget-header">
        <span>⏱ Cooldowns</span>
      </div>
      <div className="widget-body">
        {/* Ultimate */}
        <div className="cd-row">
          <button
            className="cd-btn"
            onClick={() => setCdR(90)}
            style={{ opacity: cdR > 0 ? 0.5 : 1 }}
          >
            <span className="cd-key">R</span>
            <span className="cd-spell">Ace in the Hole</span>
            {cdR > 0
              ? <span className="cd-time red">{formatTime(cdR)}</span>
              : <span className="cd-time green">Ready</span>
            }
          </button>
        </div>

        {/* Flash */}
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
          Click a button when you cast the spell
        </div>
      </div>
    </div>
  );
}
