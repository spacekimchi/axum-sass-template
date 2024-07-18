use axum::Router;

mod health_check;
mod homepage;

pub fn homepage_routes() -> Router {
    Router::new().nest("/", homepage::routes())
}

pub fn health_check_routes() -> Router {
    Router::new().nest("/health", health_check::routes())
}
