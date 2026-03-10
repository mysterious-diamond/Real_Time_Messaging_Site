use axum::{ http::StatusCode, response::{ IntoResponse, Response } };

pub enum APIError {
    Unauthorized,
    NotFound,
    InternalServerError,
    Conflict,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        match self {
            APIError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            APIError::NotFound => (StatusCode::NOT_FOUND, "Page not found").into_response(),
            APIError::InternalServerError =>
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
            APIError::Conflict => (StatusCode::CONFLICT, "Already exists").into_response(),
        }
    }
}
