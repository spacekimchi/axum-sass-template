// use sqlx::{Pool, Postgres};
// Pool<Postgres> is similar to PgPool
// PgPool is sqlx's version
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use secrecy::Secret;
use axum::{Extension,Router};
use tower::ServiceBuilder;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use tera::Tera;
use tower_http::services::{ServeDir, ServeFile};
use std::fs;
use std::path::Path;

use crate::configuration::Settings;
use crate::configuration::DatabaseSettings;
use crate::routes::health_check_routes;
use crate::routes::homepage_routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub hmac_secret: Secret<String>,
    pub tera: Arc<Tera>,
}

pub struct Application {
    port: u16,
    server: axum::serve::Serve<Router, Router>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        // Compile SCSS files to CSS at runtime
        compile_scss_to_css("scss", "public/css");
        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr().unwrap().port();
        let tera = Tera::new("templates/**/*html")?;
        let tera = Arc::new(tera);
        let server = run(
            connection_pool,
            listener,
            configuration.application.base_url,
            configuration.redis_uri,
            configuration.application.hmac_secret,
            tera,
        ).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(
    configuration: &DatabaseSettings
) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub String);

pub async fn run(db_pool: PgPool, listener: TcpListener, _base_url: String, _redis_uri: Secret<String>, hmac_secret: Secret<String>, tera: Arc<Tera>) -> Result<axum::serve::Serve<Router, Router>, anyhow::Error> {

    let app = api_router()
        .layer(
            ServiceBuilder::new()
            .layer(Extension(AppState {db: db_pool, hmac_secret, tera}))
            .layer(TraceLayer::new_for_http())
        );
    let serve = axum::serve(listener, app);

    Ok(serve)
}

fn api_router() -> Router {
    // The ServeDir directory will allow the application to access these files and its
    // subdirectories
    let service = ServeDir::new("public")
        .fallback(ServeFile::new("assets/file_not_found.html"));

    Router::new()
        .nest_service("/public", service)
        .merge(health_check_routes())
        .merge(homepage_routes())
}

fn compile_scss_to_css(scss_dir: &str, css_dir: &str) {
    // Create the CSS directory if it doesn't exist
    fs::create_dir_all(css_dir).unwrap();

    // Compile SCSS files to CSS
    for entry in fs::read_dir(scss_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("scss") {
            let css = grass::from_path(&path, &grass::Options::default()).expect("Failed to compile SCSS");

            let css_path = Path::new(css_dir).join(path.with_extension("css").file_name().unwrap());
            fs::write(css_path, css).expect("Failed to write CSS");
        }
    }
}
