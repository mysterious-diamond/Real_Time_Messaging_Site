use axum::{extract::{State, Path}, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use crate::{AppState, errors::APIError};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Message {
    pub id: i32,
    pub room_id: i32,
    pub user_id: i32,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path(room_id): Path<i32>
) -> Result<impl IntoResponse, APIError> {
    let pool: &MySqlPool = &state.pool;

    let result: Vec<Message> = sqlx::query_as!(
        Message,
        "SELECT messages.id, messages.room_id, messages.user_id, 
        users.username, messages.content, messages.created_at 
        FROM messages 
        JOIN users ON messages.user_id = users.id 
        WHERE messages.room_id = ?",
        room_id
    )
    .fetch_all(pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok(Json(result))
}