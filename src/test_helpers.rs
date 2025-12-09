// Test helpers for integration tests
#[cfg(test)]
use crate::api_state::AppState;
#[cfg(test)]
use sqlx::{PgPool, postgres::PgPoolOptions};
#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
/// Create a test AppState with test PostgreSQL database
pub async fn create_test_app_state() -> AppState {
    std::env::set_var("VERIDION_MASTER_KEY", "test_master_key_for_testing_only_32_bytes_long");
    
    // Use test database URL from environment (required for tests)
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| std::env::var("DATABASE_URL")
            .expect("Either TEST_DATABASE_URL or DATABASE_URL must be set for tests"));
    
    AppState::new(&database_url).await
        .expect("Failed to create test AppState. Make sure PostgreSQL is running and TEST_DATABASE_URL is set correctly.")
}

#[cfg(test)]
/// Create a test database pool (for direct database access in tests)
pub async fn create_test_db_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| std::env::var("DATABASE_URL")
            .expect("Either TEST_DATABASE_URL or DATABASE_URL must be set for tests"));
    
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}

