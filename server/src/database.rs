use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Database connection pool configuration
pub struct DatabaseConfig {
    /// Maximum number of connections in the pool (tuned for game server workload)
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: Duration,
    /// Idle connection timeout in seconds
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            // Game server typically needs fewer connections than web services
            // - Read match state: low frequency
            // - Write match results: end of match only
            // Most connections are short-lived queries
            max_connections: 10,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Initialize database connection pool
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
/// * `config` - Optional custom configuration (uses defaults if None)
///
/// # Returns
/// Configured PgPool ready for queries
///
/// # Example
/// ```
/// let pool = init_database("postgresql://user@localhost/thermite", None).await?;
/// ```
pub async fn init_database(
    database_url: &str,
    config: Option<DatabaseConfig>,
) -> Result<PgPool, sqlx::Error> {
    let config = config.unwrap_or_default();

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(database_url)
        .await?;

    // Verify connection with simple query
    sqlx::query!("SELECT 1 as test")
        .fetch_one(&pool)
        .await?;

    Ok(pool)
}

/// Test database connectivity and schema integrity
///
/// Verifies:
/// - Connection is alive
/// - Required tables exist
/// - Migration tracking table exists
///
/// # Returns
/// Ok(()) if all checks pass, Err otherwise
pub async fn verify_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Check that core tables exist
    let tables = sqlx::query!(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
        AND table_name IN ('players', 'matches', 'match_participants', 'stash_items')
        ORDER BY table_name
        "#
    )
    .fetch_all(pool)
    .await?;

    if tables.len() != 4 {
        return Err(sqlx::Error::Configuration(
            format!("Expected 4 core tables, found {}", tables.len()).into(),
        ));
    }

    // Verify migrations table exists
    let migrations = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM _sqlx_migrations
        WHERE success = true
        "#
    )
    .fetch_one(pool)
    .await?;

    if migrations.count.unwrap_or(0) == 0 {
        return Err(sqlx::Error::Configuration(
            "No successful migrations found".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://delorenj@localhost:5432/thermite".to_string());

        let pool = init_database(&database_url, None)
            .await
            .expect("Failed to connect to database");

        verify_schema(&pool)
            .await
            .expect("Schema verification failed");

        pool.close().await;
    }
}
