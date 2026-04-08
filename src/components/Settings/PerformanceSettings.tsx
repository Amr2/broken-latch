import React from "react";

interface PerformanceConfig {
  performance_mode: boolean;
  cpu_usage_limit: number;
}

interface Props {
  config: PerformanceConfig;
  onChange: (c: PerformanceConfig) => void;
}

export function PerformanceSettings({ config, onChange }: Props) {
  return (
    <section style={sectionStyle}>
      <h2 style={headingStyle}>Performance</h2>

      <div style={rowStyle}>
        <div>
          <p style={labelStyle}>Performance mode</p>
          <p style={descStyle}>Reduce background polling to save CPU</p>
        </div>
        <Toggle
          checked={config.performance_mode}
          onChange={(v) => onChange({ ...config, performance_mode: v })}
        />
      </div>

      <div style={{ ...rowStyle, flexDirection: "column", alignItems: "flex-start", gap: "0.5rem" }}>
        <div style={{ display: "flex", justifyContent: "space-between", width: "100%" }}>
          <div>
            <p style={labelStyle}>CPU usage limit</p>
            <p style={descStyle}>Target maximum platform CPU percentage</p>
          </div>
          <span style={{ color: "#d1d5db" }}>{config.cpu_usage_limit.toFixed(1)}%</span>
        </div>
        <input
          type="range"
          min={0.5}
          max={10}
          step={0.5}
          value={config.cpu_usage_limit}
          onChange={(e) => onChange({ ...config, cpu_usage_limit: Number(e.target.value) })}
          style={{ width: "100%", accentColor: "#2563eb" }}
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
  fontSize: "1.125rem", fontWeight: 600, marginBottom: "1rem", color: "#f9fafb",
};
const rowStyle: React.CSSProperties = {
  display: "flex", justifyContent: "space-between", alignItems: "center",
  padding: "0.75rem 0", borderTop: "1px solid #374151",
};
const labelStyle: React.CSSProperties = { fontWeight: 500, color: "#f3f4f6" };
const descStyle: React.CSSProperties = { fontSize: "0.875rem", color: "#9ca3af", marginTop: "0.125rem" };
