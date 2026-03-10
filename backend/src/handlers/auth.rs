use axum::{ Json, http::StatusCode, response::IntoResponse, extract::State };
use sqlx::MySqlPool;
use jsonwebtoken::{ encode, EncodingKey, Header };
use bcrypt::{ hash, verify };
use std::time::{ SystemTime, UNIX_EPOCH };

use crate::{ AppState, errors::APIError, middleware::auth::Claims };

#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    username: String,
    password_hash: String,
}

#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(data): Json<RegisterRequest>
) -> Result<impl IntoResponse, APIError> {
    let RegisterRequest { username, password } = data;
    let pool: &MySqlPool = &state.pool;

    let result: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = ?",
        username
    )
        .fetch_optional(pool).await
        .unwrap();

    if let Some(_) = result {
        return Err(APIError::Conflict);
    }

    let password_hash: String = hash(password, 12).map_err(|_| APIError::InternalServerError)?;

    sqlx::query!("INSERT INTO users(username, password_hash) VALUES(?, ?)", username, password_hash)
        .execute(pool).await
        .map_err(|_| APIError::InternalServerError)?;

    Ok((StatusCode::CREATED, "User created").into_response())
}

pub async fn login(
    State(state): State<AppState>, 
    Json(data): Json<LoginRequest>
) -> Result<impl IntoResponse, APIError> {
    let LoginRequest {username, password} = data;
    let secret: String = std::env::var("JWT_SECRET").unwrap();
    let pool: &MySqlPool = &state.pool;

    let result: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = ?",
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| APIError::InternalServerError)?;

    let user: User = match result {
        Some(user) => user,
        None => return Err(APIError::NotFound),
    };

    let is_valid_password: bool = verify(password, &user.password_hash).map_err(|_| APIError::InternalServerError)?;
    if !is_valid_password {
        return Err(APIError::Unauthorized);
    }

    let expiry: usize =
        (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize) +
        60 * 60 * 24 * 7;

    let token = encode(
        &Header::default(),
        &(Claims { sub: user.id, exp: expiry }),
        &EncodingKey::from_secret(secret.as_bytes())
    ).unwrap();
    Ok((StatusCode::OK, Json(token)).into_response())
}
