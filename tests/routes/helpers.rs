use sqlx::{PgConnection, Executor, Connection};
use axum_sass_template::configuration::{get_configuration, DatabaseSettings};
use axum_sass_template::telemetry::{get_subscriber, init_subscriber};
use axum_sass_template::startup::Application;
use sqlx::PgPool;
use once_cell::sync::Lazy;
use uuid::Uuid;
// use argon2::password_hash::SaltString;
// use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});


pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
    pub db_settings: DatabaseSettings,
}

impl TestApp {
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize
    {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .form(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_health_check(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/health", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_users(&self, body: &serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/api/users", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn _post_users_form<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize
    {
        self.api_client
            .post(&format!("{}/users", &self.address))
            .form(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_login_html(&self) -> String {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
            .text()
            .await
            .unwrap()
    }

    // pub async fn _login_test_user(&self) -> reqwest::Response {
    //     let login_body = serde_json::json!({
    //         "username": &self.test_user.username,
    //         "password": &self.test_user.password
    //     });
    //     self.post_login(&login_body).await
    // }
}

pub async fn spawn_app() -> TestApp {
    /*
     * The first time 'initialize is invoked the code in 'TRACING' is executed.
     * All other invocations will instead skip execution (so init_subscriber() is only called once)
     */
    Lazy::force(&TRACING);
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    /* Session */
    let db_pool = configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);

    let _ = tokio::spawn(application.run_until_stopped());
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let test_app = TestApp {
        address,
        db_pool,
        port: application_port,
        api_client: client,
        db_settings: configuration.database
    };
    // test_app.test_user.store(&mut test_app.db_pool).await;
    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    /* Create database to use for testing */
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}

