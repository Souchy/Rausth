pub mod jwt;
pub mod email;
pub mod oauth;  // Legacy - will be deprecated
pub mod refresh;
pub mod providers;

pub use jwt::JwtService;
pub use email::EmailPasswordAuth;
pub use oauth::{OAuthService, OAuthProvider};  // Legacy
pub use refresh::RefreshTokenService;

// New provider-specific services
pub use providers::{GoogleOAuthService, MicrosoftOAuthService, GitHubOAuthService};
