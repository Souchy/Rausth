pub mod google;
pub mod microsoft;
pub mod github;

pub use google::GoogleOAuthService;
pub use microsoft::MicrosoftOAuthService;
pub use github::GitHubOAuthService;
