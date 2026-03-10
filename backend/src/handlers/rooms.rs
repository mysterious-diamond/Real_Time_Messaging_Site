use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use crate::{AppState, errors::APIError, middleware::auth::Claims};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Room {
    id: i32,
    name: String,
    created_by: i32,
    created_at: DateTime<Utc>
}

#[derive(serde::Deserialize)]
pub struct CreateRoomRequest {
    name: String
}

pub async fn get_all_rooms(
    State(state): State<AppState>
) -> Result<impl IntoResponse, APIError> {
    let pool: &MySqlPool = &state.pool;

    let result: Vec<Room> = sqlx::query_as!(
        Room,
        "SELECT id, name, created_by, created_at FROM rooms"
    )
    .fetch_all(pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok(Json(result))
}

pub async fn create_room(
    State(state): State<AppState>,
    claims: Claims,
    Json(data): Json<CreateRoomRequest>,
) -> Result<impl IntoResponse, APIError> {
    let CreateRoomRequest {name} = data;
    let pool: &MySqlPool = &state.pool;

    sqlx::query!(
        "INSERT INTO rooms(name, created_by) VALUES (?, ?)",
        name,
        claims.sub,
    )
    .execute(pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    Ok((StatusCode::CREATED, "Room created").into_response())
}

