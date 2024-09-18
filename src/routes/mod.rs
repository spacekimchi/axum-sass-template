use axum::Router;
use axum::middleware;
use crate::startup::AppState;
use crate::constants::route_paths;
use crate::middleware::admin::require_admin;

mod health_check;
mod homepage;
mod auth;
mod protected;
mod admin;

pub fn homepage_routes() -> Router {
    Router::new().nest(route_paths::ROOT, homepage::routes())
}

pub fn auth_routes() -> Router {
    Router::new().nest(route_paths::ROOT, auth::routes())
}

pub fn health_check_routes() -> Router {
    Router::new().nest(route_paths::HEALTH, health_check::routes())
}

pub fn protected_routes() -> Router {
    Router::new().nest(route_paths::PROTECTED, protected::routes())
}

pub fn admin_routes(app_state: &AppState) -> Router {
    Router::new()
        .nest(route_paths::ADMIN, admin::handlers::routes())
        .route_layer(middleware::from_fn_with_state(app_state.clone(), require_admin))
}
