# Initialization

## `LOLOverlay.init(config)`

Must be called once before using any other SDK method. Registers your app with the platform and sets up the authentication token.

**Parameters:**

| Name | Type | Required | Description |
|---|---|---|---|
| `config.appId` | `string` | Yes | Must match the `id` field in your manifest |
| `config.version` | `string` | Yes | Your app's current version |

**Returns:** `void`

**Example:**

```javascript
LOLOverlay.init({
  appId: "hunter-mode",
  version: "2.1.0",
});
```

## App ID

The `appId` must exactly match the `id` field in `manifest.json`. The platform uses this to:

- Scope storage keys to your app
- Route events to the correct windows
- Authenticate API requests

## Auth Token

After calling `init()`, the SDK automatically reads the `x-app-id` URL parameter injected by the platform into widget window URLs. You do not need to manage authentication manually.

## Best Practice

Call `init()` at the very top of your main script, before any event listeners or async operations:

```javascript
// ✅ Correct — init first
LOLOverlay.init({ appId: "my-app", version: "1.0.0" });

LOLOverlay.game.onPhaseChange((event) => {
  // ...
});

// ❌ Wrong — calling SDK before init
LOLOverlay.game.onPhaseChange(() => {}); // may fail
LOLOverlay.init({ appId: "my-app", version: "1.0.0" });
```
