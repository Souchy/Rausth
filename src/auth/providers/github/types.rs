use serde::{Deserialize, Serialize};

/// GitHub user information returned from the GitHub API.
///
/// This struct represents the user data we receive after successful
/// OAuth authentication with GitHub.
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUserInfo {
    /// The user's unique identifier from GitHub
    pub id: u64,
    /// The user's email address (may not be public)
    pub email: Option<String>,
}

/// GitHub email information for fetching verified emails.
///
/// Used when the user's email is not publicly available.
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEmail {
    /// The email address
    pub email: String,
    /// Whether this is the user's primary email
    pub primary: bool,
    /// Whether the email has been verified
    pub verified: bool,
}
