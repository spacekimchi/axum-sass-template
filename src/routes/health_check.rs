use axum::{Router, routing::get};
use crate::constants::route_paths;

pub fn routes() -> Router {
    Router::new().route(route_paths::ROOT, get(self::get::health_check))
}

mod get {
    pub async fn health_check() -> axum::response::Json<serde_json::Value> {
        axum::response::Json(serde_json::json!({ "status": "ok" }))
    }
}
