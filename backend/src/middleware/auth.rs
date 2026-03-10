use axum::extract::FromRequestParts;
use axum::http::{ HeaderMap, request::Parts };
use jsonwebtoken::{ decode, DecodingKey, Validation };
use async_trait::async_trait;
use crate::errors::APIError;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims where S: Send + Sync {
    // Runs automatically when we try to get Claims
    type Rejection = APIError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = std::env::var("JWT_SECRET").unwrap();
        let token = extract_token(&parts.headers).ok_or(APIError::Unauthorized)?;

        decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default()
        )
            .map(|data| data.claims)
            .map_err(|_| APIError::Unauthorized) // Returns the data if valid, else returns Unauthorized
    }
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.to_string())
}
