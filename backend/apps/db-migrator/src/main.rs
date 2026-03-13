use persistence::connect_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("db-migrator", "db_migrator=info");
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| std::io::Error::other("DATABASE_URL is required for db-migrator"))?;
    let pool = connect_pool(&database_url, 2).await?;

    tracing::info!("applying SQLx migrations");
    sqlx::migrate!("../../migrations").run(&pool).await?;
    tracing::info!("database migrations applied successfully");

    Ok(())
}
