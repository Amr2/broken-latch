/**
 * Launching.tsx — Game Launching Stage Overlay
 * =============================================
 * Shown when the platform detects LeagueOfLegends.exe starting up.
 *
 * WIDGET LAYOUT (1920×1080 reference)
 * ─────────────────────────────────────
 *  [TOP CENTER]  Notification banner — "League of Legends Detected"
 *
 * CAPABILITIES DEMONSTRATED
 * ─────────────────────────
 *  ■ Single centered notification widget
 *  ■ pointer-events: auto on the panel (user can dismiss it)
 *  ■ Win32 interactive region: { x:300, y:20, width:500, height:80 }
 *
 * PRODUCTION USAGE
 * ────────────────
 *  In production, this stage is entered automatically when game::detect
 *  finds a LeagueOfLegends.exe process starting.  The widget would show
 *  real platform initialization progress (connecting to LCU, loading DB, etc.)
 *
 * DEVELOPER GUIDE — how to build a "launch" widget:
 *  1. Register your widget with the platform (Task 07 app manifest)
 *  2. Subscribe to "LAUNCHING" in your widget lifecycle hooks
 *  3. Render an absolutely-positioned panel with pointer-events: auto
 *  4. Call update_interactive_regions() with your bounding box
 *  5. Dismiss (hide) when you receive the next phase-changed event
 */

import { useState } from 'react';
import '../overlay/OverlayApp.css';
import './stages.css';

export default function Launching() {
  const [dismissed, setDismissed] = useState(false);

  if (dismissed) return null;

  return (
    <div
      className="widget-panel launching-banner"
      style={{ top: 20, left: '50%', transform: 'translateX(-50%)', width: 520 }}
    >
      <span className="demo-badge">DEMO</span>

      {/* Widget: Launch notification */}
      <div className="widget-header">
        <span>🚀 Platform Notification</span>
        {/* Interactive close button — demonstrates pointer-events: auto */}
        <button className="dismiss-btn" onClick={() => setDismissed(true)} title="Dismiss">✕</button>
      </div>

      <div className="widget-body launch-body">
        <div className="launch-icon">🎮</div>
        <div className="launch-info">
          <div className="launch-title">League of Legends Detected</div>
          <div className="launch-subtitle">Process: LeagueOfLegends.exe  •  PID: 12340</div>
          <div className="launch-progress">
            <div className="progress-bar">
              <div className="progress-fill launching" />
            </div>
            <span className="progress-label">Platform initializing…</span>
          </div>
          <div className="launch-steps">
            <Step done label="Game process found" />
            <Step done label="LCU connection established" />
            <Step active label="Loading platform hooks" />
            <Step label="Overlay ready" />
          </div>
        </div>
      </div>

      {/* Documentation callout for developers */}
      <div className="dev-callout">
        <code>phase: LAUNCHING</code> — overlay shows, Rust emits <code>phase-changed</code> event.
        Interactive region registered at <code>&#123; x:300, y:20, w:520, h:~130 &#125;</code>.
      </div>
    </div>
  );
}

function Step({ done, active, label }: { done?: boolean; active?: boolean; label: string }) {
  const icon = done ? '✓' : active ? '◉' : '○';
  const cls  = done ? 'step done' : active ? 'step active' : 'step';
  return <div className={cls}><span>{icon}</span>{label}</div>;
}
