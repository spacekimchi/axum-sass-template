use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum::Extension;
use axum::response::Html;
use axum_messages::{Message, Messages};
use serde::Deserialize;
use crate::startup::AppState;
use crate::template_helpers::{render_content, RenderTemplateParams};

use crate::user::{AuthSession, Credentials};

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn routes() -> Router<()> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/login", get(self::get::login))
        .route("/logout", get(self::get::logout))
}

mod post {
    use super::*;

    pub async fn login(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        println!("\n\nTRYING TO LOGIN\n\n");
        println!("\n\nTRYING TO LOGIN: {:?}\n\n", creds);
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                messages.error("Invalid credentials");

                let mut login_url = "/login".to_string();
                if let Some(next) = creds.next {
                    login_url = format!("{}?next={}", login_url, next);
                };

                return Redirect::to(&login_url).into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        messages.success(format!("Successfully logged in as {}", user.username));

        if let Some(ref next) = creds.next {
            Redirect::to(next)
        } else {
            Redirect::to("/")
        }
        .into_response()
    }
}

mod get {
    use super::*;

    pub async fn login(
        Extension(state): Extension<AppState>,
        messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> impl IntoResponse {
        let mut context = tera::Context::new();
        let boo = "FROM THE LOGIN ROUTE";
        context.insert("boo", &boo);
        match render_content(
            &RenderTemplateParams::new("login.html", &state.tera)
            .with_context(&context)
        ) {
            Ok(homepage_template) => Html(homepage_template).into_response(),
            Err(e) => e.into_response()
        }
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
