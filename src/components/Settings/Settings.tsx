import React from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { GeneralSettings } from "./GeneralSettings";
import { OverlaySettings } from "./OverlaySettings";
import { PerformanceSettings } from "./PerformanceSettings";
import { DeveloperSettings } from "./DeveloperSettings";

interface PlatformConfig {
  version: string;
  startup: {
    launch_with_windows: boolean;
    auto_update_check: boolean;
  };
  overlay: {
    default_opacity: number;
    screen_capture_visible: boolean;
  };
  performance: {
    performance_mode: boolean;
    cpu_usage_limit: number;
  };
  developer: {
    debug_mode: boolean;
    show_console_logs: boolean;
    simulated_game_phases: boolean;
  };
}

export function Settings() {
  const queryClient = useQueryClient();

  const { data: config, isLoading } = useQuery<PlatformConfig>({
    queryKey: ["platform-config"],
    queryFn: () => invoke<PlatformConfig>("get_platform_config"),
  });

  const updateMutation = useMutation({
    mutationFn: (updated: PlatformConfig) =>
      invoke<void>("update_platform_config", { config: updated }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["platform-config"] }),
  });

  if (isLoading || !config) {
    return (
      <div style={{ padding: "2rem", color: "#9ca3af" }}>Loading settings…</div>
    );
  }

  const update = (patch: Partial<PlatformConfig>) =>
    updateMutation.mutate({ ...config, ...patch });

  return (
    <div style={{ minHeight: "100vh", background: "#111827", color: "white", padding: "2rem" }}>
      <div style={{ maxWidth: "720px", margin: "0 auto" }}>
        <div style={{ marginBottom: "2rem" }}>
          <h1 style={{ fontSize: "1.875rem", fontWeight: 700 }}>Platform Settings</h1>
          <p style={{ color: "#9ca3af", marginTop: "0.25rem" }}>
            Platform v{config.version}
          </p>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
          <GeneralSettings
            config={config.startup}
            onChange={(startup) => update({ startup })}
          />
          <OverlaySettings
            config={config.overlay}
            onChange={(overlay) => update({ overlay })}
          />
          <PerformanceSettings
            config={config.performance}
            onChange={(performance) => update({ performance })}
          />
          <DeveloperSettings
            config={config.developer}
            onChange={(developer) => update({ developer })}
          />
        </div>

        {updateMutation.isPending && (
          <p style={{ color: "#6b7280", marginTop: "1rem", fontSize: "0.875rem" }}>
            Saving…
          </p>
        )}
      </div>
    </div>
  );
}
