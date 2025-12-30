use crate::auth::{EmailPasswordAuth, JwtService};
use crate::config::Config;
use crate::db::Db;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest};
use crate::utils::AuthResult;
use rocket::serde::json::Json;
use rocket::{State, post};
use rocket_db_pools::Connection;

#[post("/register", data = "<request>")]
pub async fn register(
    mut db: Connection<Db>,
    config: &State<Config>,
    request: Json<RegisterRequest>,
) -> AuthResult<Json<AuthResponse>> {
    if !config.auth.email_password {
        return Err(crate::utils::AuthError::ConfigError(
            "Email/password authentication is disabled".to_string(),
        ));
    }

    let jwt_service = JwtService::new(config.jwt.clone());
    let auth = EmailPasswordAuth::new(jwt_service);
    
    let response = auth.register(&mut *db, request.into_inner()).await?;
    Ok(Json(response))
}

#[post("/login", data = "<request>")]
pub async fn login(
    mut db: Connection<Db>,
    config: &State<Config>,
    request: Json<LoginRequest>,
) -> AuthResult<Json<AuthResponse>> {
    if !config.auth.email_password {
        return Err(crate::utils::AuthError::ConfigError(
            "Email/password authentication is disabled".to_string(),
        ));
    }

    let jwt_service = JwtService::new(config.jwt.clone());
    let auth = EmailPasswordAuth::new(jwt_service);
    
    let response = auth.login(&mut *db, request.into_inner()).await?;
    Ok(Json(response))
}
