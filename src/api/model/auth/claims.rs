use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    // Subject the user's UUID
    pub sub: String,
    // Issued at (Unix timestamp)
    pub iat: i64,
    // Expiration (Unix timestamp)
    pub exp: i64,
    // Token type discriminant
    pub token_type: String,
    // OAuth provider
    pub provider: String,
    // User email (optional)
    pub email: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    // Subject the user's UUID
    pub sub: String,
    // Unique token ID (for revocation)
    pub jti: String,
    // Issued at (Unix timestamp)
    pub iat: i64,
    // Expiration (Unix timestamp)
    pub exp: i64,
    // Token type discriminant
    pub token_type: String,
}