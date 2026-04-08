import React from "react";

interface DeveloperConfig {
  debug_mode: boolean;
  show_console_logs: boolean;
  simulated_game_phases: boolean;
}

interface Props {
  config: DeveloperConfig;
  onChange: (c: DeveloperConfig) => void;
}

export function DeveloperSettings({ config, onChange }: Props) {
  const update = (key: keyof DeveloperConfig, value: boolean) =>
    onChange({ ...config, [key]: value });

  return (
    <section style={sectionStyle}>
      <h2 style={headingStyle}>Developer</h2>
      <p style={{ ...descStyle, marginBottom: "0.5rem" }}>
        These settings are intended for app developers.
      </p>

      {([
        ["debug_mode", "Debug mode", "Extra logging and diagnostic overlays"],
        ["show_console_logs", "Show console logs", "Print SDK logs to the developer console"],
        ["simulated_game_phases", "Simulate game phases", "Cycle through phases without League running"],
      ] as const).map(([key, label, desc]) => (
        <div key={key} style={rowStyle}>
          <div>
            <p style={labelStyle}>{label}</p>
            <p style={descStyle}>{desc}</p>
          </div>
          <Toggle
            checked={config[key]}
            onChange={(v) => update(key, v)}
          />
        </div>
      ))}
    </section>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      style={{
        width: "3rem", height: "1.5rem", borderRadius: "9999px",
        border: "none", cursor: "pointer",
        background: checked ? "#2563eb" : "#4b5563",
        position: "relative", flexShrink: 0,
      }}
    >
      <span style={{
        position: "absolute", top: "0.125rem",
        left: checked ? "1.625rem" : "0.125rem",
        width: "1.25rem", height: "1.25rem",
        borderRadius: "50%", background: "white",
        transition: "left 0.2s",
      }} />
    </button>
  );
}

const sectionStyle: React.CSSProperties = {
  background: "#1f2937", borderRadius: "0.75rem",
  padding: "1.5rem", border: "1px solid #374151",
};
const headingStyle: React.CSSProperties = {
  fontSize: "1.125rem", fontWeight: 600, marginBottom: "0.5rem", color: "#f9fafb",
};
const rowStyle: React.CSSProperties = {
  display: "flex", justifyContent: "space-between", alignItems: "center",
  padding: "0.75rem 0", borderTop: "1px solid #374151",
};
const labelStyle: React.CSSProperties = { fontWeight: 500, color: "#f3f4f6" };
const descStyle: React.CSSProperties = { fontSize: "0.875rem", color: "#9ca3af", marginTop: "0.125rem" };
