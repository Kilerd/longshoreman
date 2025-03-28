use anyhow::Result;
use gotcha::{async_trait, GotchaContext, State};
use gotcha::axum::http::StatusCode;
use gotcha::{api, Json, axum::extract::FromRequestParts, axum::http::request::Parts};
use crate::error::AppError;
use crate::service::{LoginRequest, LoginResponse, UserManager, ChangePasswordRequest, JwtManager, Claims, Token};
use crate::{App, AppState, Config};

pub async fn login(app: State<AppState>, payload: Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    let user_manager = app.user_manager.lock().await;
    if user_manager.verify_user(&payload.email, &payload.password)? {
        let jwt_manager = app.jwt_manager.lock().await;
        let token = jwt_manager.create_token(&payload.email)?;
        Ok(Json(LoginResponse { token }))
    } else {
        Err(AppError::Auth("Invalid credentials".to_string()))
    }
}

pub struct AuthUser {
    pub email: String,
}

#[async_trait]
impl FromRequestParts<GotchaContext<AppState, Config>> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &GotchaContext<AppState, Config>) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header format"));
        }

        let token = &auth_header[7..];
        let jwt_manager = state.state.jwt_manager.lock().await;
        let claims = jwt_manager.verify_token(token).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        Ok(AuthUser {
            email: claims.sub,
        })
    }
}

pub async fn change_password(
    app: State<AppState>  ,
    auth_user: AuthUser,
    payload: Json<ChangePasswordRequest>,
) -> Result<Json<String>, AppError> {
    let mut user_manager = app.user_manager.lock().await;
    user_manager.change_password(
        &auth_user.email,
        &payload.old_password,
        &payload.new_password,
    )?;

    // Generate new server ID to invalidate all existing tokens
    let mut jwt_manager = app.jwt_manager.lock().await;
    jwt_manager.regenerate_server_id();

    Ok(Json("Password changed successfully".to_string()))
} 