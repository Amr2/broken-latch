# Windows API

Create and control overlay widget windows.

## Methods

### `create(config)`

Create a new widget window.

**Parameters:** `WidgetConfig`

**Returns:** `Promise<string>` — the widget ID

```javascript
const widgetId = await LOLOverlay.windows.create({
  id: "score-tracker",
  url: "tracker.html",
  defaultPosition: { x: 20, y: 100 },
  defaultSize: { width: 300, height: 400 },
  draggable: true,
  resizable: false,
  persistPosition: true,
  opacity: 0.9,
  showInPhases: ["InGame"],
});
```

---

### `show(widgetId)`

Show a hidden widget window.

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.windows.show("score-tracker");
```

---

### `hide(widgetId)`

Hide a widget window (does not destroy it).

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.windows.hide("score-tracker");
```

---

### `toggle(widgetId)`

Toggle a widget window's visibility.

**Returns:** `Promise<void>`

```javascript
await LOLOverlay.windows.toggle("score-tracker");
```

---

### `setOpacity(widgetId, opacity)`

Set the window's opacity.

**Parameters:**
- `widgetId: string`
- `opacity: number` — 0.0 to 1.0

**Returns:** `Promise<void>`

---

### `setPosition(widgetId, position)`

Move the widget window.

**Parameters:**
- `widgetId: string`
- `position: { x: number; y: number }`

**Returns:** `Promise<void>`

---

### `setClickThrough(widgetId, enabled)`

Enable or disable click-through mode for the window.

**Parameters:**
- `widgetId: string`
- `enabled: boolean`

**Returns:** `Promise<void>`

---

### `destroy(widgetId)`

Permanently close and remove a widget window.

**Returns:** `Promise<void>`

---

## Types

### `WidgetConfig`

```typescript
interface WidgetConfig {
  id: string;
  url: string;
  defaultPosition: { x: number; y: number };
  defaultSize: { width: number; height: number };
  minSize?: { width: number; height: number };
  maxSize?: { width: number; height: number };
  draggable?: boolean;       // default: true
  resizable?: boolean;       // default: false
  persistPosition?: boolean; // default: false
  clickThrough?: boolean;    // default: false
  opacity?: number;          // default: 1.0
  showInPhases?: string[];   // default: [] (always visible)
}
```
