use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,      // Issuer
    pub sub: String,      // Subject
    pub exp: i64,         // Expiration
    pub iat: i64,         // Issued at
}

/// JWT Handler
pub struct JwtHandler {
    secret: String,
    expiration: i64,
}

impl JwtHandler {
    pub fn new(secret: String, expiration: i64) -> Self {
        Self { secret, expiration }
    }

    /// Get token expiration time in seconds
    pub fn expiration(&self) -> i64 {
        self.expiration
    }

    /// Generate new JWT token
    pub fn generate_token(&self) -> String {
        let now = Utc::now().timestamp();
        let claims = Claims {
            iss: "export-service".to_string(),
            sub: "web-client".to_string(),
            exp: now + self.expiration,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap_or_default()
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.to_string())
    }
}
