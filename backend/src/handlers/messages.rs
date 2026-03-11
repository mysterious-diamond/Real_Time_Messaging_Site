use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use crate::{AppState, errors::APIError, middleware::auth::Claims};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Message {
    pub id: i32,
    pub room_id: i32,
    pub user_id: i32,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted: i8,
}

#[derive(sqlx::FromRow)]
struct UserId {
    pub user_id: i32,
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path(room_id): Path<i32>
) -> Result<impl IntoResponse, APIError> {
    let pool: &MySqlPool = &state.pool;

    let result: Vec<Message> = sqlx::query_as!(
        Message,
        "SELECT messages.id, messages.room_id, messages.user_id, 
        users.username, messages.created_at, messages.deleted,
        CASE WHEN messages.deleted = 1 THEN 'message deleted' ELSE messages.content END as content
        FROM messages 
        JOIN users ON messages.user_id = users.id
        WHERE messages.room_id = ?
        ORDER BY messages.created_at ASC",
        room_id
    )
    .fetch_all(pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok(Json(result))
}

pub async fn delete_message(
    State(state): State<AppState>,
    claims: Claims,
    Path(message_id): Path<i32>,
) -> Result<impl IntoResponse, APIError> {
    let result: Option<UserId> = sqlx::query_as!(
        UserId,
        "SELECT user_id FROM messages WHERE id = ?",
        message_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    match result {
        Some(user_id) => {
            if user_id.user_id != claims.sub {
                return Err(APIError::Unauthorized);
            }
        }
        None => return Err(APIError::NotFound),
    }

    sqlx::query!(
        "UPDATE messages SET deleted = True WHERE id = ?",
        message_id,
    )
    .execute(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok((StatusCode::OK, "Deleted message").into_response())
}