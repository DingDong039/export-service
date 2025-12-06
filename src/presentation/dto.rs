use serde::{Deserialize, Serialize};

/// Token response
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: i64,
    pub token_type: String,
}
