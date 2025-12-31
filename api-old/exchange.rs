use std::collections::HashMap;

// use bluepoisonserver::{
//     CONFIG, REDIS_CLIENT, REQWEST_CLIENT,
//     models::google::{GoogleTokenResponse, GoogleUserInfo},
// };
use url::Url;
use vercel_runtime::{Body, Error, Request, Response, StatusCode, run};
use redis::AsyncCommands;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let parsed_url = Url::parse(&req.uri().to_string())?;
    let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    let query_parameter = hash_query.get("code");

    // --- 1. Exchange code for tokens ---
    let Some(code) = query_parameter else {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Missing authorization code"))?);
    };

    let params = [
        ("code", code.as_str()),
        ("client_id", &CONFIG.client_id),
        ("client_secret", &CONFIG.client_secret),
        ("redirect_uri", &CONFIG.redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let res = match REQWEST_CLIENT
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Google request error: {}", e)))
                .unwrap());
        }
    };

    // if status failed
    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Body::from(format!("Google response error: {text}")))
            .unwrap());
    }

    // Parse the response into GoogleTokenResponse
    let token_res = match res.json::<GoogleTokenResponse>().await {
        Ok(resp) => resp,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Token parse error: {}", e)))
                .unwrap());
        }
    };

    // Check if refresh_token is present
    if token_res.refresh_token.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Missing refresh token"))
            .unwrap());
    }

    // --- 2. Fetch Google user info to get account_id ---
    let user_info = match request_user_info(&token_res.access_token).await {
        Ok(info) => info,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("User info request failed: {}", e)))
                .unwrap());
        }
    };

    // --- 3. Persist into DB ---
    match save_refresh_token(&user_info.sub, &token_res.refresh_token.unwrap()).await {
        Ok(_) => {}
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to save refresh token: {}", e)))
                .unwrap());
        }
    }

    // Optionally redirect to a “success” page or return JSON
    let body = serde_json::to_vec(&user_info).unwrap();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap())
}

pub async fn request_user_info(access_token: &str) -> Result<GoogleUserInfo, String> {
    // let client = reqwest::Client::new();
    let res = REQWEST_CLIENT
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Google API error: {}", res.status()));
    }

    res.json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))
}

pub async fn save_refresh_token(account_id: &String, refresh_token: &String) -> Result<(), String> {
    let mut conn = REDIS_CLIENT
        .get_multiplexed_async_connection() //.get_async_connection()
        .await
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

    let key = format!("google_refresh:{}", &account_id);

    // Store the refresh token under the account_id key
    let _: () = conn.set(key, refresh_token.clone())
        .await
        .map_err(|e| format!("Failed to save token: {}", e))?;

    Ok(())
}
