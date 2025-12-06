use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

/// Database connection pool for PostgreSQL
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Optimized connection pool settings
        let pool = PgPoolOptions::new()
            .max_connections(20) // Increased for better concurrency
            .min_connections(5) // Keep minimum connections alive
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600)) // 10 minutes
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            .test_before_acquire(true) // Test connections before use
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        // Refresh materialized views after migration
        let _ = sqlx::query("SELECT refresh_materialized_views()")
            .execute(&pool)
            .await;

        // Analyze tables for query planner
        let _ = sqlx::query("SELECT analyze_tables()")
            .execute(&pool)
            .await;

        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Health check - verify database connection
    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Refresh materialized views (call periodically)
    #[allow(dead_code)]
    pub async fn refresh_views(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT refresh_materialized_views()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Analyze tables for query planner optimization
    #[allow(dead_code)]
    pub async fn analyze_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT analyze_tables()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

