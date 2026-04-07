# API Overview

The broken-latch JavaScript SDK (`LOLOverlay`) exposes a set of namespaced modules, all available after including the SDK script.

## Loading the SDK

```html
<script src="http://localhost:45678/sdk/loloverlay.js"></script>
```

The SDK attaches itself to `window.LOLOverlay`.

## Initialization

Always call `init()` before using any other API:

```javascript
LOLOverlay.init({
  appId: "my-app",     // must match manifest id
  version: "1.0.0",
});
```

## Module Overview

| Module | Description |
|---|---|
| [`LOLOverlay.game`](/api/game-lifecycle) | Game phase and session data |
| [`LOLOverlay.windows`](/api/windows) | Create and control widget windows |
| [`LOLOverlay.hotkeys`](/api/hotkeys) | Register global keyboard shortcuts |
| [`LOLOverlay.storage`](/api/storage) | Persistent key-value storage |
| [`LOLOverlay.notifications`](/api/notifications) | Toast notifications |
| [`LOLOverlay.messaging`](/api/messaging) | Inter-app messaging |
| [`LOLOverlay.platform`](/api/platform) | Platform info and utilities |

## HTTP API

The platform also exposes a REST API at `http://localhost:45678`:

| Endpoint | Description |
|---|---|
| `GET /health` | Platform health check |
| `GET /game/phase` | Current game phase |
| `GET /game/session` | Current game session |
| `GET /apps` | Installed apps list |
| `GET /sdk/loloverlay.js` | SDK bundle |

## Error Handling

All async SDK methods return Promises and reject with descriptive error messages:

```javascript
try {
  await LOLOverlay.windows.create({ ... });
} catch (err) {
  console.error("Failed to create window:", err.message);
}
```
