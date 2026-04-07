use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, State as AxumState, Json},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
};
use tower_http::cors::CorsLayer;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::detect::GameDetector;
use crate::game::session::SessionManager;
use crate::hotkey::HotkeyManager;
use crate::widgets::WidgetManager;

/// Shared state for the API server
#[derive(Clone)]
pub struct ApiState {
    pub detector: Arc<GameDetector>,
    pub session_mgr: Arc<SessionManager>,
    pub hotkey_manager: Arc<HotkeyManager>,
    pub widget_manager: Arc<WidgetManager>,
}

/// Start the HTTP API server on localhost:45678
pub async fn start_api_server(state: ApiState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        .route("/api/windows/state/{widget_id}", get(get_window_state))
        // Hotkeys
        .route("/api/hotkeys/register", post(register_hotkey))
        .route("/api/hotkeys/unregister", post(unregister_hotkey))
        // Storage
        .route("/api/storage/{app_id}/{key}", get(get_storage))
        .route("/api/storage/{app_id}/{key}", post(set_storage))
        .route("/api/storage/{app_id}/{key}", delete(delete_storage))
        // Webhooks
        .route("/api/webhooks/register", post(register_webhook))
        .route("/api/webhooks/unregister", post(unregister_webhook))
        // SDK file
        .route("/sdk/loloverlay.js", get(serve_sdk))
        // CORS
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:45678").await?;
    log::info!("HTTP API server listening on http://localhost:45678");
    axum::serve(listener, app).await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// ROUTE HANDLERS
// ────────────────────────────────────────────────────────────────────────────

async fn get_platform_info() -> Json<PlatformInfo> {
    Json(PlatformInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: "broken-latch".to_string(),
    })
}

async fn get_game_phase(
    AxumState(state): AxumState<ApiState>,
) -> Json<GamePhaseResponse> {
    let phase = state.detector.get_current_phase();
    Json(GamePhaseResponse {
        phase: format!("{:?}", phase),
        timestamp: now_unix(),
    })
}

async fn get_game_session(
    AxumState(state): AxumState<ApiState>,
) -> Json<serde_json::Value> {
    match state.session_mgr.get_current_session() {
        Some(session) => Json(serde_json::to_value(session).unwrap_or_default()),
        None => Json(serde_json::json!(null)),
    }
}

async fn create_window(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWindowRequest>,
) -> Result<Json<CreateWindowResponse>, ApiError> {
    validate_auth(&headers)?;

    let config = crate::widgets::WidgetConfig {
        id: payload.id.clone(),
        app_id: payload.app_id.clone(),
        url: payload.url,
        default_position: crate::widgets::Position { x: payload.x, y: payload.y },
        default_size: crate::widgets::Size { width: payload.width, height: payload.height },
        min_size: None,
        max_size: None,
        draggable: payload.draggable.unwrap_or(true),
        resizable: payload.resizable.unwrap_or(false),
        persist_position: false,
        click_through: payload.click_through.unwrap_or(false),
        opacity: payload.opacity.unwrap_or(1.0),
        show_in_phases: payload.show_in_phases.unwrap_or_default(),
    };

    let widget_id = state.widget_manager.create_widget(config)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(CreateWindowResponse { widget_id }))
}

async fn show_window(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<WindowActionRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    state.widget_manager.show_widget(&payload.widget_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn hide_window(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<WindowActionRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    state.widget_manager.hide_widget(&payload.widget_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn set_window_position(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<SetPositionRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    state.widget_manager.set_widget_position(
        &payload.widget_id,
        crate::widgets::Position { x: payload.x, y: payload.y },
    ).map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn set_window_opacity(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<SetOpacityRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    state.widget_manager.set_widget_opacity(&payload.widget_id, payload.opacity)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn get_window_state(
    AxumState(state): AxumState<ApiState>,
    Path(widget_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let widget_state = state.widget_manager.get_state(&widget_id)
        .map_err(|e| ApiError::NotFound(e.to_string()))?;
    Ok(Json(serde_json::to_value(widget_state).unwrap_or_default()))
}

async fn register_hotkey(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterHotkeyRequest>,
) -> Result<Json<RegisterHotkeyResponse>, ApiError> {
    validate_auth(&headers)?;
    let win_id = state.hotkey_manager.register(&payload.app_id, &payload.hotkey_id, &payload.keys)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok(Json(RegisterHotkeyResponse { win_hotkey_id: win_id }))
}

async fn unregister_hotkey(
    AxumState(state): AxumState<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<UnregisterHotkeyRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    state.hotkey_manager.unregister(payload.win_hotkey_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn get_storage(
    Path((_app_id, _key)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate_auth(&headers)?;
    // TODO: Query app_storage table
    Ok(Json(serde_json::json!(null)))
}

async fn set_storage(
    Path((_app_id, _key)): Path<(String, String)>,
    headers: HeaderMap,
    Json(_payload): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    // TODO: Upsert app_storage table
    Ok(StatusCode::OK)
}

async fn delete_storage(
    Path((_app_id, _key)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    // TODO: Delete from app_storage table
    Ok(StatusCode::OK)
}

async fn register_webhook(
    headers: HeaderMap,
    Json(_payload): Json<RegisterWebhookRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    // TODO: Insert into webhooks table
    Ok(StatusCode::OK)
}

async fn unregister_webhook(
    headers: HeaderMap,
    Json(_payload): Json<UnregisterWebhookRequest>,
) -> Result<StatusCode, ApiError> {
    validate_auth(&headers)?;
    // TODO: Delete from webhooks table
    Ok(StatusCode::OK)
}

// ────────────────────────────────────────────────────────────────────────────
// AUTH
// ────────────────────────────────────────────────────────────────────────────

fn validate_auth(headers: &HeaderMap) -> Result<(), ApiError> {
    let _token = headers
        .get("X-broken-latch-App-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    // TODO: Verify token against installed apps
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

    /// Broadcast event to all registered webhook URLs
    pub async fn broadcast_event(
        &self,
        event: &str,
        payload: serde_json::Value,
        webhook_urls: Vec<String>,
    ) {
        for url in webhook_urls {
            let event_payload = serde_json::json!({
                "event": event,
                "data": payload,
                "timestamp": now_unix(),
            });

            let client = self.client.clone();
            tokio::spawn(async move {
                match client.post(&url).json(&event_payload).send().await {
                    Ok(resp) => log::debug!("Webhook delivered to {}: {}", url, resp.status()),
                    Err(e) => log::warn!("Webhook delivery failed to {}: {}", url, e),
                }
            });
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// REQUEST/RESPONSE TYPES
// ────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PlatformInfo {
    version: String,
    platform: String,
}

#[derive(Serialize)]
struct GamePhaseResponse {
    phase: String,
    timestamp: i64,
}

#[derive(Deserialize)]
struct CreateWindowRequest {
    app_id: String,
    id: String,
    url: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    draggable: Option<bool>,
    resizable: Option<bool>,
    click_through: Option<bool>,
    opacity: Option<f32>,
    show_in_phases: Option<Vec<String>>,
}

#[derive(Serialize)]
struct CreateWindowResponse {
    widget_id: String,
}

#[derive(Deserialize)]
struct WindowActionRequest {
    widget_id: String,
}

#[derive(Deserialize)]
struct SetPositionRequest {
    widget_id: String,
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
struct SetOpacityRequest {
    widget_id: String,
    opacity: f32,
}

#[derive(Deserialize)]
struct RegisterHotkeyRequest {
    app_id: String,
    hotkey_id: String,
    keys: String,
}

#[derive(Serialize)]
struct RegisterHotkeyResponse {
    win_hotkey_id: i32,
}

#[derive(Deserialize)]
struct UnregisterHotkeyRequest {
    win_hotkey_id: i32,
}

#[derive(Deserialize)]
struct RegisterWebhookRequest {
    #[allow(dead_code)]
    app_id: String,
    #[allow(dead_code)]
    event: String,
    #[allow(dead_code)]
    url: String,
}

#[derive(Deserialize)]
struct UnregisterWebhookRequest {
    #[allow(dead_code)]
    app_id: String,
    #[allow(dead_code)]
    event: String,
}

// ────────────────────────────────────────────────────────────────────────────
// ERROR HANDLING
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum ApiError {
    Unauthorized,
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, serde_json::json!({"error": "Unauthorized"})),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, serde_json::json!({"error": msg})),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": msg})),
        };
        (status, Json(body)).into_response()
    }
}

/// Serve the pre-built JavaScript SDK bundle.
/// Apps load it via: <script src="http://localhost:45678/sdk/loloverlay.js"></script>
async fn serve_sdk() -> impl IntoResponse {
    const SDK_CONTENT: &str = include_str!("../../sdk/dist/loloverlay.js");

    (
        StatusCode::OK,
        [
            ("Content-Type", "application/javascript; charset=utf-8"),
            ("Cache-Control", "public, max-age=3600"),
        ],
        SDK_CONTENT,
    )
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info_response() {
        let info = PlatformInfo {
            version: "0.1.0".to_string(),
            platform: "broken-latch".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("broken-latch"));
    }

    #[test]
    fn test_auth_missing_header() {
        let headers = HeaderMap::new();
        assert!(validate_auth(&headers).is_err());
    }

    #[test]
    fn test_auth_present_header() {
        let mut headers = HeaderMap::new();
        headers.insert("X-broken-latch-App-Token", "test-token".parse().unwrap());
        assert!(validate_auth(&headers).is_ok());
    }
}
