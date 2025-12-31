use reqwest::StatusCode;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub db_url: String,
}


#[derive(Clone)]
pub struct ProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scopes: String,
    pub redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Result<Self, (StatusCode, String)> {
        Ok(Self {
            client_id: std::env::var("GOOGLE_CLIENT_ID").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing client ID".into()))?,
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing client secret".into()))?,
            redirect_uri: std::env::var("GOOGLE_REDIRECT_URI").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing redirect URI".into()))?,
            db_url: std::env::var("DB_URL").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing database URL".into()))?,
        })
    }
}
