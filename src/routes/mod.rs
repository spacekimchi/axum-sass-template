use axum::Router;

mod health_check;
mod homepage;
mod auth;
mod protected;

pub fn homepage_routes() -> Router {
    Router::new().nest("/", homepage::routes())
}

pub fn auth_routes() -> Router {
    Router::new().nest("/", auth::routes())
}

pub fn health_check_routes() -> Router {
    Router::new().nest("/health", health_check::routes())
}

pub fn protected_routes() -> Router {
    Router::new().nest("/protected", protected::routes())
}
