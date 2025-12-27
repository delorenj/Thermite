mod database;
pub mod map_system;

use database::{init_database, verify_schema};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Thermite Server - Database Connection Test");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://delorenj@localhost:5432/thermite".to_string());

    println!("Connecting to database...");
    let pool = init_database(&database_url, None).await?;
    println!("✓ Database connection established");

    println!("Verifying schema...");
    verify_schema(&pool).await?;
    println!("✓ Schema verification passed");

    // Query item definitions count
    let count = sqlx::query!("SELECT COUNT(*) as count FROM item_definitions")
        .fetch_one(&pool)
        .await?;
    println!("✓ Item definitions: {} items", count.count.unwrap_or(0));

    pool.close().await;
    println!("✓ Database connection closed");

    Ok(())
}
