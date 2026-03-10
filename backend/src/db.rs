pub use sqlx;
use std::time::Duration;
use sqlx::{ MySqlPool, mysql::MySqlPoolOptions };

pub async fn create_mysql_pool() -> MySqlPool {
    let url: String = std::env::var("DATABASE_URL").unwrap();
    println!("Connecting to: {}", url);

    MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&url).await
        .unwrap()
}
