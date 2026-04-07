import React from "react";
import { X, Download, Star, Package } from "lucide-react";
import type { RegistryApp } from "./AppBrowser";

interface Props {
  app: RegistryApp;
  isInstalling: boolean;
  onClose: () => void;
  onInstall: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function AppDetails({ app, isInstalling, onClose, onInstall }: Props) {
  return (
    <div
      style={{
        position: "fixed", inset: 0,
        background: "rgba(0,0,0,0.7)",
        display: "flex", alignItems: "center", justifyContent: "center",
        zIndex: 100,
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: "#1f2937", borderRadius: "1rem",
          border: "1px solid #374151", padding: "2rem",
          maxWidth: "560px", width: "90%", maxHeight: "80vh",
          overflowY: "auto", position: "relative",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={onClose}
          style={{
            position: "absolute", top: "1rem", right: "1rem",
            background: "none", border: "none", color: "#9ca3af",
            cursor: "pointer", padding: "0.25rem",
          }}
        >
          <X size={20} />
        </button>

        <div style={{ display: "flex", gap: "1rem", marginBottom: "1.5rem" }}>
          {app.icon_url ? (
            <img
              src={app.icon_url}
              alt={app.name}
              style={{ width: "4rem", height: "4rem", borderRadius: "0.75rem", objectFit: "cover" }}
            />
          ) : (
            <div style={{
              width: "4rem", height: "4rem", borderRadius: "0.75rem",
              background: "#374151", display: "flex", alignItems: "center", justifyContent: "center",
              fontSize: "1.75rem",
            }}>
              {app.name[0]}
            </div>
          )}
          <div>
            <h2 style={{ fontSize: "1.25rem", fontWeight: 700, color: "#f9fafb" }}>{app.name}</h2>
            <p style={{ color: "#9ca3af", fontSize: "0.875rem" }}>by {app.author} · v{app.version}</p>
            <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginTop: "0.25rem" }}>
              <Star size={14} fill="#fbbf24" color="#fbbf24" />
              <span style={{ color: "#fbbf24", fontSize: "0.875rem" }}>{app.rating.toFixed(1)}</span>
              <span style={{ color: "#4b5563", fontSize: "0.75rem" }}>·</span>
              <span style={{ color: "#6b7280", fontSize: "0.75rem" }}>
                {app.download_count.toLocaleString()} installs
              </span>
            </div>
          </div>
        </div>

        <p style={{ color: "#d1d5db", lineHeight: 1.6, marginBottom: "1.5rem" }}>
          {app.description}
        </p>

        <div style={{
          display: "flex", gap: "1rem", flexWrap: "wrap",
          marginBottom: "1.5rem", fontSize: "0.8rem",
        }}>
          {[
            ["Category", app.category],
            ["Size", formatBytes(app.file_size)],
            ["Requires platform", `≥ v${app.min_platform_version}`],
          ].map(([label, val]) => (
            <div key={label} style={{
              background: "#374151", borderRadius: "0.5rem", padding: "0.5rem 0.75rem",
            }}>
              <p style={{ color: "#9ca3af", marginBottom: "0.125rem" }}>{label}</p>
              <p style={{ color: "#f3f4f6", fontWeight: 500 }}>{val}</p>
            </div>
          ))}
        </div>

        <button
          onClick={onInstall}
          disabled={isInstalling}
          style={{
            width: "100%", display: "flex", alignItems: "center", justifyContent: "center",
            gap: "0.5rem", padding: "0.75rem",
            background: isInstalling ? "#1d4ed8" : "#2563eb",
            border: "none", borderRadius: "0.5rem",
            color: "white", fontSize: "1rem", fontWeight: 600,
            cursor: isInstalling ? "not-allowed" : "pointer",
            opacity: isInstalling ? 0.7 : 1,
          }}
        >
          {isInstalling ? (
            <><Package size={18} /> Installing…</>
          ) : (
            <><Download size={18} /> Install</>
          )}
        </button>
      </div>
    </div>
  );
}
