use serde::{Deserialize, Serialize};

/// Microsoft user information returned from the Microsoft Graph API.
///
/// This struct represents the user data we receive after successful
/// OAuth authentication with Microsoft.
#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftUserInfo {
    /// The user's unique identifier from Microsoft
    pub id: String,
    /// The user's primary email address (may not be set)
    pub mail: Option<String>,
    /// The user's User Principal Name (alternative to email)
    pub user_principal_name: Option<String>,
}
