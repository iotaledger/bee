pub(crate) struct Database {
    pub pool: sqlx::SqlitePool,
}

impl Database {
    pub(crate) async fn new() -> Result<Self,sqlx::Error> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let pool = sqlx::SqlitePool::connect(&database_url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self {pool})
    }

    pub(crate) async fn new_in_memory() -> Result<Self,sqlx::Error> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self {pool})
    }
}