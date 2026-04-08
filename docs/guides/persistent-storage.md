# Persistent Storage

Storage data persists across game sessions, app restarts, and platform updates.

## Basic Usage

```javascript
// Save
await LOLOverlay.storage.set("config", { theme: "dark", opacity: 0.85 });

// Load
const config = await LOLOverlay.storage.get("config");

// Delete
await LOLOverlay.storage.delete("config");
```

## Pattern: Load with Defaults

```javascript
const DEFAULTS = {
  opacity: 0.9,
  position: { x: 20, y: 100 },
  showOnStartup: true,
};

async function loadConfig() {
  const saved = await LOLOverlay.storage.get("config");
  return { ...DEFAULTS, ...(saved ?? {}) };
}

async function saveConfig(config) {
  await LOLOverlay.storage.set("config", config);
  await LOLOverlay.notifications.show("Settings saved", { type: "success", duration: 1200 });
}
```

## Pattern: Per-Champion Settings

Store settings scoped to individual champions:

```javascript
async function getChampSettings(championId) {
  const all = (await LOLOverlay.storage.get("champ-settings")) ?? {};
  return all[championId] ?? { notes: "", priority: "normal" };
}

async function setChampSettings(championId, settings) {
  const all = (await LOLOverlay.storage.get("champ-settings")) ?? {};
  all[championId] = settings;
  await LOLOverlay.storage.set("champ-settings", all);
}
```

## Pattern: Settings UI

```javascript
// Load on page ready
const config = await loadConfig();
document.getElementById("opacity").value = config.opacity;

// Save on change
document.getElementById("opacity").addEventListener("change", async (e) => {
  config.opacity = parseFloat(e.target.value);
  await saveConfig(config);
  await LOLOverlay.windows.setOpacity("main", config.opacity);
});
```

## Limits

- Max 1 MB per key
- Max 50 MB total per app
- Values must be JSON-serializable (no functions, Dates, Maps, etc.)
