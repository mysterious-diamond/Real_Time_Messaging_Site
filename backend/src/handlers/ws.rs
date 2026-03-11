use axum::{extract::{Path, Query, State, WebSocketUpgrade, ws::{Message, WebSocket}}, http::StatusCode, response::IntoResponse};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{DecodingKey, Validation, decode};
use redis::aio::MultiplexedConnection;
use sqlx::MySqlPool;
use crate::{AppState, errors::APIError, middleware::auth::Claims};

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
}

pub fn validate_token(token: &str) -> Result<Option<Claims>, APIError> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| APIError::InternalServerError)?;
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .ok())
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(room_id): Path<i32>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, APIError> {
    let claims = match validate_token(&query.token)? {
        Some(claims) => claims,
        None => return Ok((StatusCode::UNAUTHORIZED, "Unauthorized").into_response()),
    };

    // Check if user has access to this room
    let room = sqlx::query!(
        "SELECT is_private FROM rooms WHERE id = ?",
        room_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?
    .ok_or(APIError::NotFound)?;

    if room.is_private != 0 {
        let member = sqlx::query!(
            "SELECT id FROM room_members WHERE room_id = ? AND user_id = ?",
            room_id,
            claims.sub,
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| APIError::InternalServerError)?;

        if member.is_none() {
            return Err(APIError::Unauthorized);
        }
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, room_id, claims.sub)))
}

async fn publish_message(
    pool: &MySqlPool,
    redis_conn: &mut MultiplexedConnection,
    message: String,
    room_id: i32,
    user_id: i32,
) -> Result<(), APIError> {
    // get username from DB
    let user = sqlx::query!(
        "SELECT username FROM users WHERE id = ?",
        user_id
    )
        .fetch_one(pool)
        .await
        .map_err(|_| APIError::InternalServerError)?;

    let msg: String = serde_json::json!({
        "username": user.username,
        "user_id": user_id,
        "content": message,
        "room_id": room_id,
        "deleted": 0,
    }).to_string();

    redis::cmd("PUBLISH")
    .arg(format!("room:{}", room_id))
    .arg(msg.clone())
    .query_async::<_, ()>(redis_conn)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    sqlx::query!(
        "INSERT INTO messages(room_id, user_id, content) VALUES(?, ?, ?)",
        room_id,
        user_id,
        message,
    )
    .execute(pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok(())
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    room_id: i32,
    id: i32,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut pubsub = state.redis_client.get_async_pubsub().await.unwrap();
    pubsub.subscribe(format!("room:{}", room_id)).await.unwrap();

    let mut redis_conn = state.redis_client.
        get_multiplexed_async_connection()
        .await
        .unwrap();
    
    let mut messages = pubsub.on_message();

    loop {
        tokio::select! {
            Some(Ok(Message::Text(message))) = receiver.next() => {
                if let Err(_) = publish_message(&state.pool, &mut redis_conn, message, room_id, id).await {
                    return;
                }
            },

            Some(redis_msg) = messages.next() => {
                let text: String = redis_msg.get_payload().unwrap();
                sender.send(Message::Text(text)).await.unwrap();
            }
        }
    }
}