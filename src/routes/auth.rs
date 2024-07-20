use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum::Extension;
use axum::response::Html;
use axum_messages::Messages;
use serde::Deserialize;
use crate::startup::AppState;
use crate::template_helpers::{render_content, RenderTemplateParams};
use secrecy::Secret;
use crate::utils::e500;
use crate::telemetry;
use password_auth::generate_hash;

use crate::user::{AuthSession, Credentials};
use crate::domain::{NewUser, UserEmail, UserPassword};

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationForm {
    pub email: String,
    pub password: Secret<String>,
}

/// This runs validations on RegistrationForm. It tries to create the NewUser
/// struct with the values passed in from RegistrationForm.
/// Validations are inside of the NewUser file
impl TryFrom<RegistrationForm> for NewUser {
    type Error = String;

    fn try_from(value: RegistrationForm) -> Result<Self, Self::Error> {
        let email = UserEmail::parse(value.email)?;
        let password = UserPassword::parse(value.password)?;
        Ok(Self { email, password })
    }
}

pub fn routes() -> Router<()> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/register", get(self::get::register))
        .route("/register", post(self::post::register))
        .route("/login", get(self::get::login))
        .route("/logout", get(self::get::logout))
}

mod post {
    use super::*;

    pub async fn register(
        Extension(state): Extension<AppState>,
        messages: Messages,
        Form(creds): Form<RegistrationForm>,
    ) -> impl IntoResponse {
        let new_user = match NewUser::try_from(creds) {
            Ok(new_user) => new_user,
            Err(err) => {
                messages.error(err.to_string());
                return Redirect::to("/register").into_response();
            },
        };
        let user_id = uuid::Uuid::new_v4();
        let password_hash = match telemetry::spawn_blocking_with_tracing(move || generate_hash(new_user.password)).await {
         Ok(hash) => hash,
         Err(err) => {
             messages.error(err.to_string());
             return Redirect::to("/register").into_response();
         },
        };

        match sqlx::query(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) RETURNING id, email, password_hash, created_at, updated_at"
        )
            .bind(&user_id)
            .bind(&new_user.email.email)
            .bind(&password_hash)
            .fetch_one(&state.db)
            .await
            .map_err(e500) {
                Ok(user) => user,
                Err(err) => return err.into_response()
            };
        messages.success(format!("Successfully registered account!"));
        Redirect::to("/").into_response()
    }

    pub async fn login(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
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

        messages.success(format!("Successfully logged in as {}", user.email));

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

    pub async fn register(
        Extension(state): Extension<AppState>,
        _messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> impl IntoResponse {
        let mut context = tera::Context::new();
        context.insert("next", &next);
        match render_content(
            &RenderTemplateParams::new("register.html", &state.tera)
            .with_context(&context)
        ) {
            Ok(register_template) => Html(register_template).into_response(),
            Err(e) => e.into_response()
        }
    }

    pub async fn login(
        Extension(state): Extension<AppState>,
        _messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> impl IntoResponse {
        let mut context = tera::Context::new();
        let boo = "FROM THE LOGIN ROUTE";
        context.insert("boo", &boo);
        context.insert("next", &next);
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
