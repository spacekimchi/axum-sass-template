use axum::{
    http::Request,
    middleware::Next,
    response::Response,
    routing::get,
    body::Body,
    Router,
};
use axum::extract::State;
use axum::response::Redirect;
use crate::startup::AppState;
use crate::user::AuthSession;
use crate::constants::route_paths;

pub async fn require_admin(
    State(state): State<AppState>,
    auth_session: AuthSession,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Redirect> {
    match auth_session.user {
        Some(user) => {
            // Check if the user has the admin role
            let is_admin = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1 AND r.name = 'admin')",
                user.id
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(None)
            .unwrap_or(false);

            if is_admin {
                println!("\n\n\nWELCOM ADMIN\n\n\n");
                Ok(next.run(request).await)
            } else {
                println!("\n\n\nREDIRECTING BECAUSE YOU ARE NOT AN ADMIN\n\n\n");
                Err(Redirect::to(route_paths::ROOT))
            }
        },
        None => {
            println!("\n\n\nREDIRECTING BECAUSE YOU ARE NOT LOGGED IN\n\n\n");
            Err(Redirect::to(route_paths::LOGIN))
        },
    }
}
