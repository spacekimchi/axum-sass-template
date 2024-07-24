use axum_sass_template::configuration::get_configuration;
use axum_sass_template::startup::get_connection_pool;
use axum_sass_template::models::user::{User, CreateUserParams};
use fake::faker::internet::en::SafeEmail;
use fake::Fake;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = get_connection_pool(&configuration.database);
    let user = create_random_user(connection_pool).await?;
    match user {
        Some(user) => {
            println!("USER CREATED: {:#?}", user);
        },
        None => {
            println!("NOTHING FOUND");
        },
    }
    println!("Hello, Bin!");
    Ok(())
}

pub async fn create_random_user(db: sqlx::PgPool) -> Result<Option<User>, axum_sass_template::models::Error> {
    let create_user_params = CreateUserParams::new_with_default_password(fake_email());
    User::create_user(db, &create_user_params).await
}

pub fn fake_email() -> String {
    SafeEmail().fake::<String>()
}


