import React from "react";

interface StartupConfig {
  launch_with_windows: boolean;
  auto_update_check: boolean;
}

interface Props {
  config: StartupConfig;
  onChange: (c: StartupConfig) => void;
}

export function GeneralSettings({ config, onChange }: Props) {
  const update = (key: keyof StartupConfig, value: boolean) =>
    onChange({ ...config, [key]: value });

  return (
    <section style={sectionStyle}>
      <h2 style={headingStyle}>General</h2>
      <div style={rowStyle}>
        <div>
          <p style={labelStyle}>Launch with Windows</p>
          <p style={descStyle}>Start broken-latch automatically on login</p>
        </div>
        <Toggle
          checked={config.launch_with_windows}
          onChange={(v) => update("launch_with_windows", v)}
        />
      </div>
      <div style={rowStyle}>
        <div>
          <p style={labelStyle}>Auto-check for updates</p>
          <p style={descStyle}>Notify when a new platform version is available</p>
        </div>
        <Toggle
          checked={config.auto_update_check}
          onChange={(v) => update("auto_update_check", v)}
        />
      </div>
    </section>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      style={{
        width: "3rem",
        height: "1.5rem",
        borderRadius: "9999px",
        border: "none",
        cursor: "pointer",
        background: checked ? "#2563eb" : "#4b5563",
        position: "relative",
        flexShrink: 0,
        transition: "background 0.2s",
      }}
    >
      <span style={{
        position: "absolute",
        top: "0.125rem",
        left: checked ? "1.625rem" : "0.125rem",
        width: "1.25rem",
        height: "1.25rem",
        borderRadius: "50%",
        background: "white",
        transition: "left 0.2s",
      }} />
    </button>
  );
}

const sectionStyle: React.CSSProperties = {
  background: "#1f2937",
  borderRadius: "0.75rem",
  padding: "1.5rem",
  border: "1px solid #374151",
};
const headingStyle: React.CSSProperties = {
  fontSize: "1.125rem",
  fontWeight: 600,
  marginBottom: "1rem",
  color: "#f9fafb",
};
const rowStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  padding: "0.75rem 0",
  borderTop: "1px solid #374151",
};
const labelStyle: React.CSSProperties = { fontWeight: 500, color: "#f3f4f6" };
const descStyle: React.CSSProperties = { fontSize: "0.875rem", color: "#9ca3af", marginTop: "0.125rem" };
