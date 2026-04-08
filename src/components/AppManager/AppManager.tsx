import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { AppCard } from "./AppCard";
import { Plus, RefreshCw } from "lucide-react";

interface InstalledApp {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  state: string;
}

export function AppManager() {
  const [installing, setInstalling] = useState(false);
  const queryClient = useQueryClient();

  const { data: apps = [], isLoading } = useQuery<InstalledApp[]>({
    queryKey: ["installed-apps"],
    queryFn: () => invoke<InstalledApp[]>("list_installed_apps"),
  });

  const installMutation = useMutation({
    mutationFn: (lolappPath: string) =>
      invoke<void>("install_app", { lolappPath }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] }),
  });

  const startMutation = useMutation({
    mutationFn: (appId: string) => invoke<void>("start_app", { appId }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] }),
  });

  const stopMutation = useMutation({
    mutationFn: (appId: string) => invoke<void>("stop_app", { appId }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] }),
  });

  const uninstallMutation = useMutation({
    mutationFn: (appId: string) => invoke<void>("uninstall_app", { appId }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] }),
  });

  const handleInstall = async () => {
    setInstalling(true);
    try {
      const path = await invoke<string | null>("pick_lolapp_file");
      if (path) {
        await installMutation.mutateAsync(path);
      }
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div style={{ minHeight: "100vh", background: "#111827", color: "white", padding: "2rem" }}>
      <div style={{ maxWidth: "960px", margin: "0 auto" }}>
        {/* Header */}
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "2rem" }}>
          <div>
            <h1 style={{ fontSize: "1.875rem", fontWeight: 700 }}>App Manager</h1>
            <p style={{ color: "#9ca3af", marginTop: "0.25rem" }}>
              Manage your installed broken-latch apps
            </p>
          </div>
          <div style={{ display: "flex", gap: "0.75rem" }}>
            <button
              onClick={() => queryClient.invalidateQueries({ queryKey: ["installed-apps"] })}
              style={{
                padding: "0.5rem 1rem",
                background: "#374151",
                border: "none",
                borderRadius: "0.5rem",
                color: "white",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: "0.5rem",
              }}
            >
              <RefreshCw size={18} /> Refresh
            </button>
            <button
              onClick={handleInstall}
              disabled={installing || installMutation.isPending}
              style={{
                padding: "0.5rem 1rem",
                background: "#2563eb",
                border: "none",
                borderRadius: "0.5rem",
                color: "white",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: "0.5rem",
                opacity: installing ? 0.7 : 1,
              }}
            >
              <Plus size={18} />
              {installing ? "Installing…" : "Install App"}
            </button>
          </div>
        </div>

        {/* App Grid */}
        {isLoading ? (
          <div style={{ textAlign: "center", padding: "3rem", color: "#9ca3af" }}>
            Loading apps…
          </div>
        ) : apps.length > 0 ? (
          <div style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
            gap: "1.5rem",
          }}>
            {apps.map((app) => (
              <AppCard
                key={app.id}
                app={app}
                onStart={() => startMutation.mutate(app.id)}
                onStop={() => stopMutation.mutate(app.id)}
                onUninstall={() => {
                  if (window.confirm(`Uninstall ${app.name}?`)) {
                    uninstallMutation.mutate(app.id);
                  }
                }}
              />
            ))}
          </div>
        ) : (
          <div style={{ textAlign: "center", padding: "3rem" }}>
            <p style={{ color: "#9ca3af", marginBottom: "1rem" }}>No apps installed yet</p>
            <button
              onClick={handleInstall}
              style={{
                padding: "0.75rem 1.5rem",
                background: "#2563eb",
                border: "none",
                borderRadius: "0.5rem",
                color: "white",
                cursor: "pointer",
              }}
            >
              Install Your First App
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
