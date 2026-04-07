import React, { useState, useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppManager } from "./components/AppManager/AppManager";
import { Settings } from "./components/Settings/Settings";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, staleTime: 5_000 },
  },
});

type View = "apps" | "settings" | "default";

function getViewFromHash(): View {
  const hash = window.location.hash.replace("#", "");
  if (hash === "apps") return "apps";
  if (hash === "settings") return "settings";
  return "default";
}

function AppContent() {
  const [view, setView] = useState<View>(getViewFromHash);

  useEffect(() => {
    const handler = () => setView(getViewFromHash());
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  if (view === "apps") return <AppManager />;
  if (view === "settings") return <Settings />;

  // Default: platform status screen (shown in the hidden 1x1 background window)
  return (
    <div style={{
      minHeight: "100vh",
      background: "#111827",
      color: "white",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      padding: "2rem",
    }}>
      <div style={{
        background: "#1f2937",
        borderRadius: "1rem",
        padding: "2rem",
        maxWidth: "480px",
        width: "100%",
        border: "1px solid #374151",
        textAlign: "center",
      }}>
        <h1 style={{ fontSize: "1.5rem", fontWeight: 700, marginBottom: "0.5rem" }}>
          broken-latch Platform
        </h1>
        <p style={{ color: "#9ca3af" }}>Running in the background</p>
        <div style={{ marginTop: "1.5rem", display: "flex", gap: "1rem", justifyContent: "center" }}>
          <button
            onClick={() => { window.location.hash = "apps"; }}
            style={{
              padding: "0.5rem 1.25rem",
              background: "#2563eb",
              border: "none",
              borderRadius: "0.5rem",
              color: "white",
              cursor: "pointer",
            }}
          >
            Manage Apps
          </button>
          <button
            onClick={() => { window.location.hash = "settings"; }}
            style={{
              padding: "0.5rem 1.25rem",
              background: "#374151",
              border: "none",
              borderRadius: "0.5rem",
              color: "white",
              cursor: "pointer",
            }}
          >
            Settings
          </button>
        </div>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  );
}
