import React from "react";
import { Play, Square, Trash2, Settings } from "lucide-react";

interface AppCardProps {
  app: {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    enabled: boolean;
    state: string;
  };
  onStart: () => void;
  onStop: () => void;
  onUninstall: () => void;
}

export function AppCard({ app, onStart, onStop, onUninstall }: AppCardProps) {
  const isRunning = app.state === "Running";

  return (
    <div style={{
      background: "#1f2937",
      borderRadius: "0.5rem",
      padding: "1.5rem",
      border: "1px solid #374151",
    }}>
      <div style={{ marginBottom: "1rem" }}>
        <h3 style={{ fontSize: "1.125rem", fontWeight: 600, marginBottom: "0.25rem" }}>
          {app.name}
        </h3>
        <p style={{ fontSize: "0.875rem", color: "#9ca3af", marginBottom: "0.5rem" }}>
          v{app.version} by {app.author}
        </p>
        <p style={{ fontSize: "0.875rem", color: "#d1d5db" }}>{app.description}</p>
      </div>

      <div style={{ marginBottom: "1rem" }}>
        <span style={{
          display: "inline-block",
          padding: "0.25rem 0.75rem",
          borderRadius: "9999px",
          fontSize: "0.75rem",
          fontWeight: 500,
          background: isRunning ? "#14532d" : "#374151",
          color: isRunning ? "#86efac" : "#d1d5db",
        }}>
          {app.state}
        </span>
      </div>

      <div style={{ display: "flex", gap: "0.5rem" }}>
        {isRunning ? (
          <button
            onClick={onStop}
            style={{
              flex: 1,
              padding: "0.5rem 1rem",
              background: "#dc2626",
              border: "none",
              borderRadius: "0.5rem",
              color: "white",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              gap: "0.5rem",
            }}
          >
            <Square size={16} /> Stop
          </button>
        ) : (
          <button
            onClick={onStart}
            style={{
              flex: 1,
              padding: "0.5rem 1rem",
              background: "#16a34a",
              border: "none",
              borderRadius: "0.5rem",
              color: "white",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              gap: "0.5rem",
            }}
          >
            <Play size={16} /> Start
          </button>
        )}

        <button
          title="App Settings (coming soon)"
          style={{
            padding: "0.5rem 0.75rem",
            background: "#374151",
            border: "none",
            borderRadius: "0.5rem",
            color: "white",
            cursor: "pointer",
          }}
        >
          <Settings size={16} />
        </button>

        <button
          onClick={onUninstall}
          title="Uninstall"
          style={{
            padding: "0.5rem 0.75rem",
            background: "#374151",
            border: "none",
            borderRadius: "0.5rem",
            color: "white",
            cursor: "pointer",
          }}
        >
          <Trash2 size={16} />
        </button>
      </div>
    </div>
  );
}
