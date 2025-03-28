use std::fmt;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // email
    pub exp: i64,          // expiration time
    pub server_id: String, // server UUID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token: String,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    server_id: String,
}
impl fmt::Debug for JwtManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JwtManager {{ server_id: {} }}", self.server_id)
    }
}

impl JwtManager {
    pub fn new(secret: &[u8]) -> Self {
        let server_id = Uuid::new_v4().to_string();
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            server_id,
        }
    }

    pub fn create_token(&self, email: &str) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: email.to_string(),
            exp: expiration,
            server_id: self.server_id.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &self.encoding_key,
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        
        // Verify server_id matches
        if token_data.claims.server_id != self.server_id {
            return Err(AppError::Auth("Token is invalid: server ID mismatch".to_string()));
        }

        Ok(token_data.claims)
    }

    pub fn get_server_id(&self) -> &str {
        &self.server_id
    }

    pub fn regenerate_server_id(&mut self) {
        self.server_id = Uuid::new_v4().to_string();
    }
} 