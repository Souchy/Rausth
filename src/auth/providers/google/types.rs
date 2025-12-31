use serde::{Deserialize, Serialize};

/// Google user information returned from the Google API.
///
/// This struct represents the user data we receive after successful
/// OAuth authentication with Google.
#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    /// The user's unique identifier from Google
    pub id: String,
    /// The user's email address
    pub email: String,
    /// Whether the email has been verified by Google
    pub verified_email: bool,
}
