pub use redis;

pub fn create_redis_connection() -> redis::Client {
    let url: String = std::env::var("REDIS_URL").unwrap();
    redis::Client::open(url).unwrap()
}
