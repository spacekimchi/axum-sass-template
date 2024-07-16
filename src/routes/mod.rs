use axum::Router;

mod health_check;

pub fn health_check_routes() -> Router {
    Router::new().nest("/health", health_check::routes())
}
