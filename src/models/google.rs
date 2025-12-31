use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub account_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct GoogleTokenDoc {
//     pub provider_account_id: String,
//     pub refresh_token: String,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub locale: Option<String>,
}
