use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub email_password: bool,
    pub google: Option<OAuthProviderConfig>,
    pub microsoft: Option<OAuthProviderConfig>,
    pub github: Option<OAuthProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_seconds: i64,
    pub refresh_token_expiry_seconds: i64,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        
        let config = Config {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8000".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")?,
            },
            auth: AuthConfig {
                email_password: std::env::var("AUTH_EMAIL_PASSWORD")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                google: Self::parse_oauth_provider("GOOGLE"),
                microsoft: Self::parse_oauth_provider("MICROSOFT"),
                github: Self::parse_oauth_provider("GITHUB"),
            },
            jwt: JwtConfig {
                secret: std::env::var("JWT_SECRET")?,
                access_token_expiry_seconds: std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()?,
                refresh_token_expiry_seconds: std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
                    .unwrap_or_else(|_| "604800".to_string())
                    .parse()?,
            },
        };
        
        Ok(config)
    }

    fn parse_oauth_provider(prefix: &str) -> Option<OAuthProviderConfig> {
        let enabled = std::env::var(format!("AUTH_{}_ENABLED", prefix))
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        
        if !enabled {
            return None;
        }

        let client_id = std::env::var(format!("AUTH_{}_CLIENT_ID", prefix)).ok()?;
        let client_secret = std::env::var(format!("AUTH_{}_CLIENT_SECRET", prefix)).ok()?;
        let redirect_uri = std::env::var(format!("AUTH_{}_REDIRECT_URI", prefix)).ok()?;

        Some(OAuthProviderConfig {
            enabled,
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}
