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

use crate::configuration::Settings;
use crate::configuration::DatabaseSettings;
use crate::routes::health_check_routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub hmac_secret: Secret<String>,
}

pub struct Application {
    port: u16,
    server: axum::serve::Serve<Router, Router>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            connection_pool,
            listener,
            configuration.application.base_url,
            configuration.redis_uri,
            configuration.application.hmac_secret,
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

pub async fn run(db_pool: PgPool, listener: TcpListener, _base_url: String, _redis_uri: Secret<String>, hmac_secret: Secret<String>) -> Result<axum::serve::Serve<Router, Router>, anyhow::Error> {
    let app = api_router()
        .layer(
            ServiceBuilder::new()
            .layer(Extension(AppState {db: db_pool, hmac_secret}))
            .layer(TraceLayer::new_for_http())
        );
    let serve = axum::serve(listener, app);

    Ok(serve)
}

fn api_router() -> Router {
    // This is the order that the modules were authored in.
    Router::new()
        .merge(health_check_routes())
}
