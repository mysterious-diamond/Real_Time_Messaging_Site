use dotenv::dotenv;
use tokio::net::TcpListener;
use axum::{ Router, routing::{ post, get, delete } };
use sqlx::MySqlPool;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

mod db;
mod redis_client;
mod errors;
mod middleware;
mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub redis_client: redis::Client
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);

    let pool: MySqlPool = db::create_mysql_pool().await;
    let redis_client: redis::Client = redis_client::create_redis_connection();

    let state: AppState = AppState { pool, redis_client };

    let app = Router::new()
        .route("/register", post(handlers::auth::register_user))
        .route("/login", post(handlers::auth::login))
        .route("/rooms", get(handlers::rooms::get_all_rooms))
        .route("/rooms", post(handlers::rooms::create_room))
        .route("/rooms/:id/messages", get(handlers::messages::get_messages))
        .route("/ws/:room_id", get(handlers::ws::ws_handler))
        .route("/messages/:id", delete(handlers::messages::delete_message))
        .route("/rooms/:id/invite", post(handlers::rooms::invite_user))
        .route("/rooms/:id", get(handlers::rooms::get_room))
        .layer(cors)
        .with_state(state);
    let listener: TcpListener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
