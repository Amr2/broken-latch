# Debugging

## Enable Debug Mode

In broken-latch → **Settings → Developer**:

- **Debug mode** — enables diagnostic overlays and verbose platform logging
- **Show console logs** — forwards SDK log output to the widget's DevTools console
- **Simulate game phases** — cycles game phases without a live League client

## Opening DevTools

With debug mode enabled, right-click any widget window and choose **Inspect Element** to open Chrome DevTools.

## Console Logging

```javascript
// All LOLOverlay calls are logged in debug mode
LOLOverlay.init({ appId: "my-app", version: "1.0.0" });

// Your own logs appear in DevTools console
console.log("Phase changed:", event.current);
console.warn("Session data missing");
console.error("Failed to load config:", err);
```

## Network Tab

Use DevTools **Network** tab to inspect:
- SDK calls to `http://localhost:45678/api/*`
- Webhook POSTs to your backend
- Any `fetch()` calls in your app

## Platform Logs

The platform writes logs to:

```
%APPDATA%\broken-latch\logs\platform.log
```

Useful for diagnosing app install/start failures.

## Common Issues

| Symptom | Likely cause |
|---|---|
| Widget doesn't appear | Check `show_in_phases` in manifest |
| `PermissionError` | Add missing permission to manifest |
| SDK not found | Platform not running; check system tray |
| `invoke` returns null | Game not running; check phase |
| Window flickers | Reduce opacity animation frequency |

## Testing Phase Transitions

With **Simulate game phases** enabled, the platform cycles through phases automatically. You can also trigger specific phases via the debug panel (broken-latch → right-click tray → Debug).
