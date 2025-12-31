use Rausth::{REQWEST_CLIENT};
// use bluepoisonserver::models::google::{GoogleTokenResponse, RefreshTokenRequest};
// use bluepoisonserver::{CONFIG, REDIS_CLIENT, REQWEST_CLIENT};
use redis::AsyncCommands;
use serde_json;
use vercel_runtime::{Body, Error, Request, Response, StatusCode, run};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Parse the incoming JSON payload
    let payload: RefreshTokenRequest = match serde_json::from_slice(req.body()) {
        Ok(p) => p,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Invalid JSON"))
                .unwrap());
        }
    };

    // get redis connection
    let mut conn = match REDIS_CLIENT.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Database connection error: {}", e)))
                .unwrap());
        }
    };

    // get the refresh token from Redis
    let key = format!("google_refresh:{}", &payload.account_id);
    let refresh_token_opt: Option<String> = match conn.get(&key).await {
        Ok(token) => token,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Missing refresh token key error: {}",
                    e
                )))
                .unwrap());
        }
    };

    // ensure the refresh token exists
    let Some(refresh_token) = refresh_token_opt else {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("No refresh token found"))
            .unwrap());
    };

    // request new tokens from Google
    let params = [
        ("client_id", CONFIG.client_id.as_str()),
        ("client_secret", CONFIG.client_secret.as_str()),
        ("refresh_token", &refresh_token),
        ("grant_type", "refresh_token"),
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
                .body(Body::from(format!("Google API error: {}", e)))
                .unwrap());
        }
    };

    // if status failed
    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Body::from(format!("Google returned an error: {text}")))
            .unwrap());
    }

    // Parse the response into GoogleTokenResponse
    let token_response = match res.json::<GoogleTokenResponse>().await {
        Ok(resp) => resp,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Unable to parse google token response: {}",
                    e
                )))
                .unwrap());
        }
    };

    // send google token response back to client
    let body = serde_json::to_vec(&token_response).unwrap();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap())
}
