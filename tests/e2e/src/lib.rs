use zangief::prelude::*;
use std::sync::Arc;

mod tests {
    pub mod registrar;
    pub mod validator;
    pub mod docker;
}

// Test fixtures
pub struct TestContext {
    pub registrar_client: Arc<reqwest::Client>,
    pub validator_client: Arc<reqwest::Client>,
    pub db: Arc<sqlx::SqlitePool>,
}

#[zangief::suite]
pub async fn e2e_test_suite() {
    // Setup test environment
    before_all(async {
        // Initialize test database
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        
        // Run migrations
        sqlx::migrate!("../../crates/registrar/migrations")
            .run(&db)
            .await
            .unwrap();

        // Create test context
        let ctx = TestContext {
            registrar_client: Arc::new(reqwest::Client::new()),
            validator_client: Arc::new(reqwest::Client::new()),
            db: Arc::new(db),
        };

        ctx
    });

    // Cleanup after all tests
    after_all(async {
        // Cleanup resources
    });

    // Include test modules
    include_test_module!(tests::registrar);
    include_test_module!(tests::validator);
    include_test_module!(tests::docker);
}
