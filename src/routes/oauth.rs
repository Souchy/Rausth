use crate::auth::{JwtService, OAuthProvider, OAuthService};
use crate::config::Config;
use crate::db::Db;
use crate::models::{AuthResponse, OAuthCallbackRequest};
use crate::utils::{AuthError, AuthResult};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rocket_db_pools::Connection;

#[get("/google")]
pub async fn google_authorize(config: &State<Config>) -> AuthResult<Redirect> {
    let oauth_config = config.auth.google.as_ref()
        .ok_or_else(|| AuthError::ConfigError("Google OAuth is not configured".to_string()))?;

    if !oauth_config.enabled {
        return Err(AuthError::ConfigError("Google OAuth is disabled".to_string()));
    }

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    let (auth_url, _csrf_token) = oauth_service.get_authorization_url(
        OAuthProvider::Google,
        oauth_config,
    )?;

    Ok(Redirect::to(auth_url))
}

#[post("/google/callback", data = "<request>")]
pub async fn google_callback(
    mut db: Connection<Db>,
    config: &State<Config>,
    request: Json<OAuthCallbackRequest>,
) -> AuthResult<Json<AuthResponse>> {
    let oauth_config = config.auth.google.as_ref()
        .ok_or_else(|| AuthError::ConfigError("Google OAuth is not configured".to_string()))?;

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    let response = oauth_service.handle_callback(
        OAuthProvider::Google,
        oauth_config,
        request.code.clone(),
        &mut *db,
    ).await?;

    Ok(Json(response))
}

#[get("/microsoft")]
pub async fn microsoft_authorize(config: &State<Config>) -> AuthResult<Redirect> {
    let oauth_config = config.auth.microsoft.as_ref()
        .ok_or_else(|| AuthError::ConfigError("Microsoft OAuth is not configured".to_string()))?;

    if !oauth_config.enabled {
        return Err(AuthError::ConfigError("Microsoft OAuth is disabled".to_string()));
    }

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    let (auth_url, _csrf_token) = oauth_service.get_authorization_url(
        OAuthProvider::Microsoft,
        oauth_config,
    )?;

    Ok(Redirect::to(auth_url))
}

#[post("/microsoft/callback", data = "<request>")]
pub async fn microsoft_callback(
    mut db: Connection<Db>,
    config: &State<Config>,
    request: Json<OAuthCallbackRequest>,
) -> AuthResult<Json<AuthResponse>> {
    let oauth_config = config.auth.microsoft.as_ref()
        .ok_or_else(|| AuthError::ConfigError("Microsoft OAuth is not configured".to_string()))?;

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    let response = oauth_service.handle_callback(
        OAuthProvider::Microsoft,
        oauth_config,
        request.code.clone(),
        &mut *db,
    ).await?;

    Ok(Json(response))
}

#[get("/github")]
pub async fn github_authorize(config: &State<Config>) -> AuthResult<Redirect> {
    let oauth_config = config.auth.github.as_ref()
        .ok_or_else(|| AuthError::ConfigError("GitHub OAuth is not configured".to_string()))?;

    if !oauth_config.enabled {
        return Err(AuthError::ConfigError("GitHub OAuth is disabled".to_string()));
    }

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    let (auth_url, _csrf_token) = oauth_service.get_authorization_url(
        OAuthProvider::GitHub,
        oauth_config,
    )?;

    Ok(Redirect::to(auth_url))
}

#[post("/github/callback", data = "<request>")]
pub async fn github_callback(
    mut db: Connection<Db>,
    config: &State<Config>,
    request: Json<OAuthCallbackRequest>,
) -> AuthResult<Json<AuthResponse>> {
    let oauth_config = config.auth.github.as_ref()
        .ok_or_else(|| AuthError::ConfigError("GitHub OAuth is not configured".to_string()))?;

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    let response = oauth_service.handle_callback(
        OAuthProvider::GitHub,
        oauth_config,
        request.code.clone(),
        &mut *db,
    ).await?;

    Ok(Json(response))
}
