import React from "react";
import { Download, Star } from "lucide-react";
import type { RegistryApp } from "./AppBrowser";

interface Props {
  app: RegistryApp;
  isInstalling: boolean;
  onClick: () => void;
  onInstall: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function AppTile({ app, isInstalling, onClick, onInstall }: Props) {
  return (
    <div
      style={{
        background: "#1f2937", borderRadius: "0.75rem",
        border: "1px solid #374151", padding: "1.25rem",
        cursor: "pointer", transition: "border-color 0.15s",
        display: "flex", flexDirection: "column", gap: "0.75rem",
      }}
      onClick={onClick}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
        {app.icon_url ? (
          <img
            src={app.icon_url}
            alt={app.name}
            style={{ width: "2.5rem", height: "2.5rem", borderRadius: "0.5rem", objectFit: "cover" }}
          />
        ) : (
          <div style={{
            width: "2.5rem", height: "2.5rem", borderRadius: "0.5rem",
            background: "#374151", display: "flex", alignItems: "center", justifyContent: "center",
            fontSize: "1.25rem",
          }}>
            {app.name[0]}
          </div>
        )}
        <div style={{ flex: 1, minWidth: 0 }}>
          <p style={{ fontWeight: 600, color: "#f9fafb", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {app.name}
          </p>
          <p style={{ fontSize: "0.75rem", color: "#6b7280" }}>by {app.author}</p>
        </div>
      </div>

      <p style={{ fontSize: "0.875rem", color: "#9ca3af", lineHeight: 1.4, flex: 1 }}>
        {app.description.length > 80 ? app.description.slice(0, 80) + "…" : app.description}
      </p>

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "auto" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "0.25rem", color: "#fbbf24", fontSize: "0.8rem" }}>
          <Star size={12} fill="#fbbf24" />
          <span>{app.rating.toFixed(1)}</span>
          <span style={{ color: "#4b5563", marginLeft: "0.25rem" }}>{formatBytes(app.file_size)}</span>
        </div>

        <button
          onClick={(e) => { e.stopPropagation(); onInstall(); }}
          disabled={isInstalling}
          style={{
            display: "flex", alignItems: "center", gap: "0.25rem",
            padding: "0.375rem 0.75rem",
            background: isInstalling ? "#1d4ed8" : "#2563eb",
            border: "none", borderRadius: "0.375rem",
            color: "white", fontSize: "0.8rem", cursor: isInstalling ? "not-allowed" : "pointer",
            opacity: isInstalling ? 0.7 : 1,
          }}
        >
          <Download size={12} />
          {isInstalling ? "Installing…" : "Install"}
        </button>
      </div>
    </div>
  );
}
