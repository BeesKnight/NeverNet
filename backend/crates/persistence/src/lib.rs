use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn opens_postgres_pool(pool: PgPool) {
        let result = connect_pool(&std::env::var("DATABASE_URL").unwrap(), 2)
            .await
            .expect("pool should connect");
        let one = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&result)
            .await
            .unwrap();

        assert_eq!(one, 1);
        assert!(pool.acquire().await.is_ok());
    }
}
