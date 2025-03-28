use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::AppError;


type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct User {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug)]
pub struct UserManager {
    users: Vec<User>,
    file_path: String,
}

impl UserManager {
    pub fn new(file_path: &str) -> Result<Self> {
        let users = if Path::new(file_path).exists() {
            let contents = fs::read_to_string(file_path)?;
            serde_json::from_str(&contents)?
        } else {
            Vec::new()
        };

        Ok(Self {
            users,
            file_path: file_path.to_string(),
        })
    }

    pub fn save(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.users)?;
        fs::write(&self.file_path, contents)?;
        Ok(())
    }

    pub fn create_user(&mut self, email: &str, password: &str) -> Result<()> {
        if self.users.iter().any(|u| u.email == email) {
            return Err(AppError::User("User already exists".to_string()));
        }

        let password_hash = hash(password.as_bytes(), DEFAULT_COST)?;
        self.users.push(User {
            email: email.to_string(),
            password: password_hash.to_string(),
        });
        self.save()?;
        Ok(())
    }

    pub fn verify_user(&self, email: &str, password: &str) -> Result<bool> {
        if let Some(user) = self.users.iter().find(|u| u.email == email) {
            Ok(verify(password.as_bytes(), &user.password)?)
        } else {
            Ok(false)
        }
    }

    pub fn change_password(&mut self, email: &str, old_password: &str, new_password: &str) -> Result<()> {
        // Find user and verify old password
        let user = self.users.iter_mut()
            .find(|u| u.email == email)
            .ok_or_else(|| AppError::User("User not found".to_string()))?;

        if !verify(old_password.as_bytes(), &user.password)? {
            return Err(AppError::User("Invalid old password".to_string()));
        }

        // Hash new password and update
        let new_password_hash = hash(new_password.as_bytes(), DEFAULT_COST)?;
        user.password = new_password_hash;
        self.save()?;
        Ok(())
    }
} 