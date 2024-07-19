use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum::Extension;
use axum::response::Html;
use crate::startup::AppState;
use axum_messages::{Message, Messages};
use crate::template_helpers::{render_content, RenderTemplateParams};

use crate::user::AuthSession;

pub fn routes() -> Router<()> {
    Router::new().route("/", get(self::get::protected))
}

mod get {
    use super::*;

    pub async fn protected(auth_session: AuthSession, _messages: Messages, Extension(state): Extension<AppState>) -> impl IntoResponse {
        match auth_session.user {
            Some(_user) => {
                let mut context = tera::Context::new();
                let boo = "FROM PROTECTED ROUTE";
                context.insert("boo", &boo);
                match render_content(
                    &RenderTemplateParams::new("homepage.html", &state.tera)
                    .with_context(&context)
                ) {
                    Ok(homepage_template) => Html(homepage_template).into_response(),
                    Err(e) => e.into_response()
                }
            },
            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
        
    }
}
