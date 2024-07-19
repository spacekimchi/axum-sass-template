use axum_sass_template::telemetry::{get_subscriber, init_subscriber};
use axum_sass_template::configuration::get_configuration;
use axum_sass_template::startup::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /* a way for application to ignore errors from loading .env instead of failing */
    dotenv::dotenv().ok();

    let subscriber = get_subscriber("axum_sass_template".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await
}
