# Installation

## Prerequisites

- Windows 10 or later
- League of Legends installed
- A modern browser (for dev tools)

## Install broken-latch

1. Download the latest installer from [broken-latch.gg](https://broken-latch.gg)
2. Run `broken-latch-setup.exe`
3. Launch broken-latch — it appears in the system tray

The platform starts automatically with Windows. It runs a local HTTP server on `localhost:45678` that serves the SDK and platform APIs.

## Verify the Platform is Running

Open your browser and navigate to:

```
http://localhost:45678/health
```

You should see:

```json
{ "status": "ok", "version": "0.1.0" }
```

## SDK Access

The JavaScript SDK is served at:

```
http://localhost:45678/sdk/loloverlay.js
```

Include it in any HTML file:

```html
<script src="http://localhost:45678/sdk/loloverlay.js"></script>
```

## Next Steps

- [Build your first app](/getting-started/your-first-app)
- [Understand the manifest format](/getting-started/manifest)
