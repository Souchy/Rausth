use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::slice::Iter;

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

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::from_env().expect("Failed to read configuration"));

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub auth_providers: Vec<AuthProviderConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthProviderConfig {
    pub enabled: bool,
    pub name: AuthProviderType,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: String, //Vec<String>,
    // maybe urls to get user_info, exchange code for tokens, refresh tokens?
    pub auth_url: String,      // exchange code?
    pub token_url: String,     // refresh token?
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

impl From<&str> for DatabaseType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "inmemory" | "memory" => DatabaseType::InMemory,
            "redis" => DatabaseType::Redis,
            "postgres" | "postgresql" => DatabaseType::Postgres,
            "mysql" => DatabaseType::MySQL,
            other => {
                // just throw an error
                panic!("Unknown database type: {}", other);
            }
        }
    }
}

// #[derive(EnumString)] // crate "strum" if you want automatic string conversion
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AuthProviderType {
    Unknown(String),
    EmailPassword,
    Microsoft,
    Google,
    Github,
    Facebook,
    Twitter,
}

impl From<&str> for AuthProviderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "emailpassword" | "email" => AuthProviderType::EmailPassword,
            "microsoft" | "ms" => AuthProviderType::Microsoft,
            "google" => AuthProviderType::Google,
            "github" | "git" => AuthProviderType::Github,
            "facebook" | "fb" => AuthProviderType::Facebook,
            "twitter" | "tw" => AuthProviderType::Twitter,
            // "okta" => AuthProviderType::Okta,
            other => AuthProviderType::Unknown(other.to_string()),
        }
    }
}


impl AuthProviderType {
    // https://stackoverflow.com/questions/21371534/in-rust-is-there-a-way-to-iterate-through-the-values-of-an-enum
    // there's another way with Copy as well that doesn't specify the length of array.
    pub fn iter() -> Iter<'static, AuthProviderType> {
        static PROVIDERS: [AuthProviderType; 6] = [
            AuthProviderType::EmailPassword,
            AuthProviderType::Microsoft,
            AuthProviderType::Google,
            AuthProviderType::Github,
            AuthProviderType::Facebook,
            AuthProviderType::Twitter,
        ];
        PROVIDERS.iter()
    }
}

impl Settings {
    
    pub fn from_json() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            // Add configuration from a file named "config.json"
            .add_source(File::with_name("config.json"))
            // You can also override with environment variables if needed
            .add_source(Environment::with_prefix("APP"))
            .build()?;

        // Deserialize the configuration into the Settings struct
        let final_settings: Settings = settings.try_deserialize()?;

        // Use the settings in your application
        println!(
            "Database: {:?}, at {}",
            final_settings.database.kind, final_settings.database.url
        );

        Ok(final_settings)
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Settings {
            database: DatabaseSettings {
                url: std::env::var("DATABASE_URL")?,
                kind: DatabaseType::from(std::env::var("DATABASE_TYPE")?.as_str()),
            },
            auth_providers: Self::parse_oauth_providers(),
        };

        Ok(config)
    }

    fn parse_oauth_providers() -> Vec<AuthProviderConfig> {
        let mut providers = Vec::new();

        for provider in AuthProviderType::iter() {
            if let Some(config) = Self::parse_oauth_provider(provider) {
                providers.push(config);
            }
        }

        providers
    }

    fn parse_oauth_provider(kind: &AuthProviderType) -> Option<AuthProviderConfig> {
        let prefix = format!("{:?}", kind).to_uppercase();

        let enabled = std::env::var(format!("AUTH_{}_ENABLED", prefix))
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        if !enabled {
            return None;
        }

        let client_id_key = format!("AUTH_{}_CLIENT_ID", prefix);
        let client_secret_key = format!("AUTH_{}_CLIENT_SECRET", prefix);
        let redirect_uri_key = format!("AUTH_{}_REDIRECT_URI", prefix);
        let scopes_key = format!("AUTH_{}_SCOPES", prefix);
        let auth_url_key = format!("AUTH_{}_AUTH_URL", prefix);
        let token_url_key = format!("AUTH_{}_TOKEN_URL", prefix);
        let user_info_url_key = format!("AUTH_{}_USER_INFO_URL", prefix);

        //  even those aren't strictly necessary, EmailPassword doesn't need them
        // let client_id = std::env::var(&client_id_key).ok()?;
        // let client_secret = std::env::var(&client_secret_key).ok()?;
        // let redirect_uri = std::env::var(&redirect_uri_key).ok()?;

        Some(AuthProviderConfig {
            enabled,
            name: kind.clone(),
            client_id: std::env::var(&client_id_key).unwrap_or_default(),
            client_secret: std::env::var(&client_secret_key).unwrap_or_default(),
            redirect_uri: std::env::var(&redirect_uri_key).unwrap_or_default(),
            scopes: std::env::var(&scopes_key).unwrap_or_default(), //.split(',').map(|s| s.to_string()).collect(),
            auth_url: std::env::var(&auth_url_key).unwrap_or_default(),
            token_url: std::env::var(&token_url_key).unwrap_or_default(),
            user_info_url: std::env::var(&user_info_url_key).unwrap_or_default(),
        })
    }
}
