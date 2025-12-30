use crate::auth::{JwtService, RefreshTokenService};
use crate::config::Config;
use crate::db::Db;
use crate::models::{AuthResponse, RefreshTokenRequest};
use crate::utils::AuthResult;
use rocket::serde::json::Json;
use rocket::{post, State};
use rocket_db_pools::Connection;

#[post("/refresh", data = "<request>")]
pub async fn refresh_token(
    mut db: Connection<Db>,
    config: &State<Config>,
    request: Json<RefreshTokenRequest>,
) -> AuthResult<Json<AuthResponse>> {
    let jwt_service = JwtService::new(config.jwt.clone());
    let refresh_service = RefreshTokenService::new(jwt_service);
    
    let response = refresh_service.refresh(&mut *db, request.into_inner()).await?;
    Ok(Json(response))
}
