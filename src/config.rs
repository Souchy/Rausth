use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;

// pub struct Settings {
//     pub client_id: String,
//     pub client_secret: String,
//     pub redirect_uri: String,
//     pub db_url: String,
// }

// impl Settings {
//     pub fn from_env() -> Result<Self, (StatusCode, String)> {
//         Ok(Self {
//             client_id: std::env::var("GOOGLE_CLIENT_ID").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing client ID".into()))?,
//             client_secret: std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing client secret".into()))?,
//             redirect_uri: std::env::var("GOOGLE_REDIRECT_URI").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing redirect URI".into()))?,
//             db_url: std::env::var("DB_URL").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing database URL".into()))?,
//         })
//     }
// }

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| {
    read_settings().expect("Failed to read configuration")
});

fn read_settings() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        // Add configuration from a file named "config.json"
        .add_source(File::with_name("config.json"))
        // You can also override with environment variables if needed
        .add_source(Environment::with_prefix("APP"))
        .build()?;

    // Deserialize the configuration into the Settings struct
    let final_settings: Settings = settings.try_deserialize()?;

    // Use the settings in your application
    println!("Database: {:?}, at {}", final_settings.database.kind, final_settings.database.url);

    Ok(final_settings)
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub auth_providers: Vec<AuthProviderConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthProviderConfig {
    pub active: bool,
    pub name: AuthProviderType,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: String,
    // maybe urls to get user_info, exchange code for tokens, refresh tokens?
    pub auth_url: String, // exchange code?
    pub token_url: String, // refresh token?
    pub user_info_url: String, // get user info?
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub kind: DatabaseType,
}

#[derive(Debug, Deserialize)]
pub enum DatabaseType {
    InMemory,
    Redis,
    Postgres,
    MySQL,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AuthProviderType {
    Microsoft,
    Google,
    GitHub,
    Facebook,
    Twitter,
}

