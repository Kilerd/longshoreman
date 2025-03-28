use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("User error: {0}")]
    User(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
}

impl gotcha::Responder for AppError {
    fn into_response(self) -> gotcha::axum::response::Response {
        (gotcha::axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
