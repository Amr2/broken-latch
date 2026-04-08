// SDK is served via the HTTP API server at /sdk/loloverlay.js
// See http_api.rs → serve_sdk() handler
// The built SDK file is embedded at compile time from sdk/dist/loloverlay.js

pub fn log_sdk_info() {
    log::info!("SDK available at http://localhost:45678/sdk/loloverlay.js");
}
