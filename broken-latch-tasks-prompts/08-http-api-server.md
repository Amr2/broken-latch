# Task 08: HTTP API Server

**Platform: broken-latch**  
**Dependencies:** Task 01, 04, 05, 06, 07  
**Estimated Complexity:** Medium-High  
**Priority:** P0 (Critical Path)

---

## Objective

Implement the local HTTP REST API server running on `http://localhost:45678` that exposes all platform functionality to apps. This allows apps with backend processes (Node.js, Python, etc.) to interact with the platform via HTTP calls instead of only through the JavaScript SDK.

---

## Context

The HTTP API provides:

- RESTful endpoints for all platform operations
- Webhook broadcasting for events (game phase changes, hotkey presses, etc.)
- Authentication via `X-broken-latch-App-Token` header
- JSON request/response format
- CORS enabled for localhost origins only
- Serves the JavaScript SDK at `/sdk/loloverlay.js`

Apps like Hunter Mode use this to:

- Receive game phase change webhooks at their backend server
- Store/retrieve data via the platform storage API
- Trigger notifications

---

## What You Need to Build

### 1. Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
axum = "0.7"                         # Web framework
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = "0.11"                     # For webhook calls
```

### 2. API Server Structure (`src-tauri/src/http_api.rs`)

```rust
use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, State, Json},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use std::collections::HashMap;

pub struct ApiServer {
    app_handle: tauri::AppHandle,
}

impl ApiServer {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    /// Start the HTTP API server
    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            // Platform info
            .route("/api/platform/info", get(get_platform_info))

            // Game lifecycle
            .route("/api/game/phase", get(get_game_phase))
            .route("/api/game/session", get(get_game_session))

            // Window management
            .route("/api/windows/create", post(create_window))
            .route("/api/windows/show", post(show_window))
            .route("/api/windows/hide", post(hide_window))
            .route("/api/windows/set-position", post(set_window_position))
            .route("/api/windows/set-opacity", post(set_window_opacity))
            .route("/api/windows/state/:widget_id", get(get_window_state))

            // Hotkeys
            .route("/api/hotkeys/register", post(register_hotkey))
            .route("/api/hotkeys/unregister", post(unregister_hotkey))
            .route("/api/hotkeys/is-registered", get(is_hotkey_registered))

            // Storage
            .route("/api/storage/:app_id/:key", get(get_storage))
            .route("/api/storage/:app_id/:key", post(set_storage))
            .route("/api/storage/:app_id/:key", delete(delete_storage))
            .route("/api/storage/:app_id", delete(clear_storage))

            // Notifications
            .route("/api/notify/toast", post(show_toast))

            // Webhooks
            .route("/api/webhooks/register", post(register_webhook))
            .route("/api/webhooks/unregister", post(unregister_webhook))

            // SDK serving
            .route("/sdk/loloverlay.js", get(serve_sdk))

            // Middleware
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            )
            .with_state(self.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:45678").await?;

        log::info!("HTTP API server listening on http://localhost:45678");

        axum::serve(listener, app).await?;

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ROUTE HANDLERS
// ────────────────────────────────────────────────────────────────────────────

async fn get_platform_info(
    State(server): State<Arc<ApiServer>>,
) -> Result<Json<PlatformInfo>, ApiError> {
    Ok(Json(PlatformInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        installed_apps: vec![],  // TODO: Query from app registry
    }))
}

async fn get_game_phase(
    State(server): State<Arc<ApiServer>>,
) -> Result<Json<GamePhaseResponse>, ApiError> {
    // TODO: Get from game detector
    Ok(Json(GamePhaseResponse {
        phase: "InGame".to_string(),
        since: chrono::Utc::now().timestamp(),
    }))
}

async fn get_game_session(
    State(server): State<Arc<ApiServer>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Get from session manager
    Ok(Json(serde_json::json!({})))
}

async fn create_window(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<CreateWindowRequest>,
) -> Result<Json<CreateWindowResponse>, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call widget manager
    Ok(Json(CreateWindowResponse {
        widget_id: format!("{}:{}", payload.app_id, payload.window_config.id),
    }))
}

async fn show_window(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<WindowActionRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call widget manager
    Ok(StatusCode::OK)
}

async fn hide_window(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<WindowActionRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call widget manager
    Ok(StatusCode::OK)
}

async fn set_window_position(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<SetPositionRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call widget manager
    Ok(StatusCode::OK)
}

async fn set_window_opacity(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<SetOpacityRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call widget manager
    Ok(StatusCode::OK)
}

async fn get_window_state(
    State(server): State<Arc<ApiServer>>,
    Path(widget_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Call widget manager
    Ok(Json(serde_json::json!({})))
}

async fn register_hotkey(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<RegisterHotkeyRequest>,
) -> Result<Json<RegisterHotkeyResponse>, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call hotkey manager
    Ok(Json(RegisterHotkeyResponse {
        win_hotkey_id: 1,
    }))
}

async fn unregister_hotkey(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<UnregisterHotkeyRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Call hotkey manager
    Ok(StatusCode::OK)
}

async fn is_hotkey_registered(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
) -> Result<Json<IsHotkeyRegisteredResponse>, ApiError> {
    // TODO: Call hotkey manager
    Ok(Json(IsHotkeyRegisteredResponse {
        registered: false,
    }))
}

async fn get_storage(
    State(server): State<Arc<ApiServer>>,
    Path((app_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<StorageValue>, ApiError> {
    validate_auth(&headers, &app_id)?;

    // TODO: Query database
    Ok(Json(StorageValue {
        value: serde_json::Value::Null,
    }))
}

async fn set_storage(
    State(server): State<Arc<ApiServer>>,
    Path((app_id, key)): Path<(String, String)>,
    headers: HeaderMap,
    Json(payload): Json<SetStorageRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &app_id)?;

    // TODO: Save to database
    Ok(StatusCode::OK)
}

async fn delete_storage(
    State(server): State<Arc<ApiServer>>,
    Path((app_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &app_id)?;

    // TODO: Delete from database
    Ok(StatusCode::OK)
}

async fn clear_storage(
    State(server): State<Arc<ApiServer>>,
    Path(app_id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &app_id)?;

    // TODO: Delete all from database for app
    Ok(StatusCode::OK)
}

async fn show_toast(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<ShowToastRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Show toast notification
    Ok(StatusCode::OK)
}

async fn register_webhook(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<RegisterWebhookRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Save webhook to database
    Ok(StatusCode::OK)
}

async fn unregister_webhook(
    State(server): State<Arc<ApiServer>>,
    headers: HeaderMap,
    Json(payload): Json<UnregisterWebhookRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers, &payload.app_id)?;

    // TODO: Delete webhook from database
    Ok(StatusCode::OK)
}

async fn serve_sdk() -> Result<impl IntoResponse, ApiError> {
    // Serve the SDK JavaScript file
    let sdk_content = include_str!("../../sdk/dist/loloverlay.js");

    Ok((
        StatusCode::OK,
        [("Content-Type", "application/javascript")],
        sdk_content,
    ))
}

// ────────────────────────────────────────────────────────────────────────────
// REQUEST/RESPONSE TYPES
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct PlatformInfo {
    version: String,
    installed_apps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GamePhaseResponse {
    phase: String,
    since: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateWindowRequest {
    app_id: String,
    window_config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateWindowResponse {
    widget_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindowActionRequest {
    app_id: String,
    widget_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetPositionRequest {
    app_id: String,
    widget_id: String,
    x: i32,
    y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetOpacityRequest {
    app_id: String,
    widget_id: String,
    opacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterHotkeyRequest {
    app_id: String,
    hotkey_id: String,
    keys: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterHotkeyResponse {
    win_hotkey_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnregisterHotkeyRequest {
    app_id: String,
    win_hotkey_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct IsHotkeyRegisteredResponse {
    registered: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageValue {
    value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetStorageRequest {
    value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShowToastRequest {
    app_id: String,
    message: String,
    #[serde(rename = "type")]
    toast_type: String,
    duration: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterWebhookRequest {
    app_id: String,
    event: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnregisterWebhookRequest {
    app_id: String,
    event: String,
}

// ────────────────────────────────────────────────────────────────────────────
// AUTHENTICATION
// ────────────────────────────────────────────────────────────────────────────

fn validate_auth(headers: &HeaderMap, app_id: &str) -> Result<(), ApiError> {
    let token = headers
        .get("X-broken-latch-App-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    // TODO: Verify token against database
    // For now, just check header exists

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// WEBHOOK BROADCASTER
// ────────────────────────────────────────────────────────────────────────────

pub struct WebhookBroadcaster {
    client: reqwest::Client,
}

impl WebhookBroadcaster {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Broadcast event to all registered webhooks
    pub async fn broadcast_event(
        &self,
        event: &str,
        payload: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Query webhooks table for this event
        let webhooks = vec![];  // Placeholder

        for webhook_url in webhooks {
            let event_payload = serde_json::json!({
                "event": event,
                "data": payload,
                "timestamp": chrono::Utc::now().timestamp(),
            });

            // Fire and forget (don't block on webhook response)
            let client = self.client.clone();
            tokio::spawn(async move {
                match client.post(webhook_url)
                    .json(&event_payload)
                    .send()
                    .await
                {
                    Ok(response) => {
                        log::debug!("Webhook delivered: status {}", response.status());
                    }
                    Err(e) => {
                        log::warn!("Webhook delivery failed: {}", e);
                    }
                }
            });
        }

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ERROR HANDLING
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum ApiError {
    Unauthorized,
    NotFound,
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ApiError::BadRequest(msg) => {
                return (StatusCode::BAD_REQUEST, msg).into_response();
            }
            ApiError::InternalError(msg) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
            }
        };

        (status, message).into_response()
    }
}

use serde::{Serialize, Deserialize};
```

### 3. Integration in Main (`src-tauri/src/main.rs`)

```rust
use crate::http_api::ApiServer;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();

            // Start HTTP API server
            let api_server = Arc::new(ApiServer::new(app_handle.clone()));
            tauri::async_runtime::spawn(async move {
                if let Err(e) = api_server.start().await {
                    log::error!("HTTP API server failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Integration Points

### Uses Task 04 (Game Lifecycle):

- Exposes game phase and session endpoints
- Broadcasts phase change events to webhooks

### Uses Task 05 (Hotkey Manager):

- Exposes hotkey registration endpoints

### Uses Task 06 (Widget Manager):

- Exposes window management endpoints

### Uses Task 07 (App Lifecycle):

- Validates app authentication tokens
- Queries app storage from database

### Used by Task 09 (SDK):

- SDK makes HTTP calls to these endpoints as fallback

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_info_endpoint() {
        // Test GET /api/platform/info
    }

    #[test]
    fn test_auth_validation() {
        // Test auth header parsing
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_api_flow() {
    // Start server
    // Make authenticated request
    // Verify response
}
```

### Manual Testing Checklist

- [ ] Server starts on port 45678
- [ ] GET /api/platform/info returns correct data
- [ ] Auth header is required for protected endpoints
- [ ] Invalid auth token returns 401
- [ ] Window creation endpoint works
- [ ] Storage endpoints persist data
- [ ] Webhook broadcasting works
- [ ] SDK is served at /sdk/loloverlay.js
- [ ] CORS headers are present
- [ ] JSON errors are properly formatted

---

## Acceptance Criteria

✅ **Complete when:**

1. HTTP server starts successfully on localhost:45678
2. All endpoints are implemented and respond correctly
3. Authentication validates app tokens from database
4. Webhook broadcasting delivers events to registered URLs
5. SDK JavaScript file is served correctly
6. CORS is configured for localhost only
7. All unit tests pass
8. Integration tests pass
9. Manual testing checklist is 100% complete
10. API documentation is complete

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/http_api.rs`

### Modified Files:

- `src-tauri/src/main.rs` (start server)
- `src-tauri/Cargo.toml` (add dependencies)

---

## Expected Time: 8-10 hours

## Difficulty: Medium-High (Web server + auth + webhooks)
