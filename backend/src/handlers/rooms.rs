use chrono::{DateTime, Utc};
use axum::{Json, extract::{State, Path}, http::StatusCode, response::IntoResponse};
use crate::{AppState, errors::APIError, middleware::auth::Claims};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Room {
    id: i32,
    name: String,
    created_by: i32,
    is_private: i8,
    created_at: DateTime<Utc>
}

#[derive(serde::Deserialize)]
pub struct CreateRoomRequest {
    name: String,
    is_private: bool,
}

#[derive(serde::Deserialize)]
pub struct InviteRequest {
    username: String,
}

pub async fn get_all_rooms(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, APIError> {
    let result: Vec<Room> = sqlx::query_as!(
        Room,
        "SELECT id, name, created_by, is_private, created_at FROM rooms 
        WHERE is_private = 0 
        OR (is_private = 1 AND id IN (
            SELECT room_id FROM room_members WHERE user_id = ?
        ))",
        claims.sub
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok(Json(result))
}

pub async fn get_room(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<i32>,
) -> Result<impl IntoResponse, APIError> {
    let room = sqlx::query_as!(
        Room,
        "SELECT id, name, created_by, is_private, created_at FROM rooms WHERE id = ?",
        room_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?
    .ok_or(APIError::NotFound)?;

    Ok(Json(room))
}

pub async fn create_room(
    State(state): State<AppState>,
    claims: Claims,
    Json(data): Json<CreateRoomRequest>,
) -> Result<impl IntoResponse, APIError> {
    let CreateRoomRequest { name, is_private } = data;

    sqlx::query!(
        "INSERT INTO rooms(name, created_by, is_private) VALUES (?, ?, ?)",
        name,
        claims.sub,
        is_private,
    )
        .execute(&state.pool)
        .await
        .map_err(|_| APIError::InternalServerError)?;

    let room = sqlx::query!(
        "SELECT id FROM rooms WHERE name = ?",
        name,
    )
        .fetch_one(&state.pool)
        .await
        .map_err(|_| APIError::InternalServerError)?;

    // Add creator as a member
    sqlx::query!(
        "INSERT INTO room_members(room_id, user_id) VALUES (?, ?)",
        room.id,
        claims.sub,
    )
        .execute(&state.pool)
        .await
        .map_err(|_| APIError::InternalServerError)?;

    Ok((StatusCode::CREATED, "Room created").into_response())
}

pub async fn invite_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<i32>,
    Json(data): Json<InviteRequest>,
) -> Result<impl IntoResponse, APIError> {
    // Check if room exists and requester is the creator
    let room = sqlx::query!(
        "SELECT created_by FROM rooms WHERE id = ?",
        room_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?
    .ok_or(APIError::NotFound)?;

    if room.created_by != claims.sub {
        return Err(APIError::Unauthorized);
    }

    // Find the user to invite
    let user = sqlx::query!(
        "SELECT id FROM users WHERE username = ?",
        data.username,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?
    .ok_or(APIError::NotFound)?;

    // Add user as member
    sqlx::query!(
        "INSERT IGNORE INTO room_members(room_id, user_id) VALUES (?, ?)",
        room_id,
        user.id,
    )
    .execute(&state.pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok((StatusCode::OK, "User invited").into_response())
}
