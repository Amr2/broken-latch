import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { AppTile } from "./AppTile";
import { AppDetails } from "./AppDetails";

export interface RegistryApp {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: string;
  download_url: string;
  icon_url: string;
  screenshots: string[];
  min_platform_version: string;
  file_size: number;
  checksum: string;
  rating: number;
  download_count: number;
}

const CATEGORIES = ["all", "coaching", "stats", "utility", "entertainment"];

export function AppBrowser() {
  const queryClient = useQueryClient();
  const [selectedApp, setSelectedApp] = useState<RegistryApp | null>(null);
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState("all");

  const { data: registry, isLoading, isError } = useQuery({
    queryKey: ["app-registry"],
    queryFn: () => invoke<{ version: string; apps: RegistryApp[] }>("fetch_app_registry"),
    staleTime: 60_000,
  });

  const installMutation = useMutation({
    mutationFn: async (app: RegistryApp) => {
      const tempPath = await invoke<string>("download_registry_app", { appId: app.id });
      return invoke("install_app", { lolappPath: tempPath });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] });
    },
  });

  const filtered = (registry?.apps ?? []).filter((app) => {
    const matchSearch =
      app.name.toLowerCase().includes(search.toLowerCase()) ||
      app.description.toLowerCase().includes(search.toLowerCase());
    const matchCategory = category === "all" || app.category === category;
    return matchSearch && matchCategory;
  });

  return (
    <div style={{ minHeight: "100vh", background: "#111827", color: "white", padding: "2rem" }}>
      <div style={{ maxWidth: "1100px", margin: "0 auto" }}>
        <div style={{ marginBottom: "2rem" }}>
          <h1 style={{ fontSize: "1.875rem", fontWeight: 700, marginBottom: "0.25rem" }}>
            App Browser
          </h1>
          <p style={{ color: "#9ca3af" }}>Discover and install apps for broken-latch</p>
        </div>

        {/* Search + Filter */}
        <div style={{ display: "flex", gap: "0.75rem", marginBottom: "1.5rem" }}>
          <input
            type="text"
            placeholder="Search apps..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              flex: 1, padding: "0.5rem 1rem",
              background: "#1f2937", border: "1px solid #374151",
              borderRadius: "0.5rem", color: "white", fontSize: "0.875rem",
            }}
          />
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            style={{
              padding: "0.5rem 1rem",
              background: "#1f2937", border: "1px solid #374151",
              borderRadius: "0.5rem", color: "white", fontSize: "0.875rem",
            }}
          >
            {CATEGORIES.map((c) => (
              <option key={c} value={c}>
                {c === "all" ? "All Categories" : c.charAt(0).toUpperCase() + c.slice(1)}
              </option>
            ))}
          </select>
        </div>

        {/* Content */}
        {isLoading && (
          <div style={{ textAlign: "center", padding: "3rem", color: "#6b7280" }}>
            Loading apps…
          </div>
        )}
        {isError && (
          <div style={{ textAlign: "center", padding: "3rem", color: "#ef4444" }}>
            Failed to load registry. Check your connection.
          </div>
        )}
        {!isLoading && !isError && (
          <>
            {filtered.length === 0 ? (
              <div style={{ textAlign: "center", padding: "3rem", color: "#6b7280" }}>
                No apps match your search.
              </div>
            ) : (
              <div style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))",
                gap: "1.25rem",
              }}>
                {filtered.map((app) => (
                  <AppTile
                    key={app.id}
                    app={app}
                    isInstalling={installMutation.isPending && installMutation.variables?.id === app.id}
                    onClick={() => setSelectedApp(app)}
                    onInstall={() => installMutation.mutate(app)}
                  />
                ))}
              </div>
            )}
          </>
        )}
      </div>

      {selectedApp && (
        <AppDetails
          app={selectedApp}
          isInstalling={installMutation.isPending && installMutation.variables?.id === selectedApp.id}
          onClose={() => setSelectedApp(null)}
          onInstall={() => installMutation.mutate(selectedApp)}
        />
      )}
    </div>
  );
}
