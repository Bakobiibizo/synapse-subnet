//! Database operations and migrations
//! 
//! Handles SQLite database connections and schema migrations

use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}, migrate::MigrateDatabase, Sqlite};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Create database if it doesn't exist
        if !Sqlite::database_exists(database_url).await? {
            Sqlite::create_database(database_url).await?;
        }

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn transaction<F, R>(&self, f: F) -> Result<R, Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, Sqlite>) -> Result<R, Box<dyn std::error::Error>>,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
