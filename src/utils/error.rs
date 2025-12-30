use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde_json::json;
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    
    #[error("OAuth error: {0}")]
    OAuthError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal server error")]
    InternalError,
}

impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            AuthError::InvalidCredentials => Status::Unauthorized,
            AuthError::UserAlreadyExists => Status::Conflict,
            AuthError::UserNotFound => Status::NotFound,
            AuthError::InvalidToken | AuthError::TokenExpired => Status::Unauthorized,
            AuthError::ConfigError(_) => Status::BadRequest,
            _ => Status::InternalServerError,
        };

        let error_message = json!({
            "error": self.to_string()
        });

        Response::build()
            .status(status)
            .sized_body(None, Cursor::new(error_message.to_string()))
            .header(rocket::http::ContentType::JSON)
            .ok()
    }
}

pub type AuthResult<T> = Result<T, AuthError>;
