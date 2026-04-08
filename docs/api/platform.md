# Platform API

Query platform metadata and utilities.

## Methods

### `getVersion()`

Get the running platform version.

**Returns:** `Promise<string>`

```javascript
const version = await LOLOverlay.platform.getVersion();
console.log("Platform:", version); // "0.1.0"
```

---

### `getAppId()`

Get the current app's ID (as set in `init()`).

**Returns:** `string | null`

```javascript
const id = LOLOverlay.platform.getAppId();
```

---

## Platform Constants

Available as properties after `init()`:

```javascript
LOLOverlay.platform.SDK_VERSION  // "1.0.0"
LOLOverlay.platform.API_BASE     // "http://localhost:45678"
```
