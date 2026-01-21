use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::time::Duration;

pub mod queries;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Add SQLite connection options for proper file handling in containers
        let connection_url = if database_url.starts_with("sqlite:") {
            format!("{}?mode=rwc", database_url)
        } else {
            database_url.to_string()
        };

        tracing::info!("Connecting with URL: {}", connection_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&connection_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }
}
