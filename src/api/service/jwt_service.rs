use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::{api::model::{auth::claims::{AccessClaims, RefreshClaims}, user::User}, errors::app_error::AppError};

// TTL for access tokens: 5 minutes
pub const ACCESS_TOKEN_TTL_SECS: i64 = 5 * 60;

// TTL for refresh tokens: 1 day
pub const REFRESH_TOKEN_TTL_SECS: i64 = 24 * 60 * 60;

pub struct JwtService {
    access_encoding_key : EncodingKey,
    access_decoding_key : DecodingKey,
    refresh_encoding_key : EncodingKey,
    refresh_decoding_key : DecodingKey,
}

impl JwtService {
    pub fn new(access_secret: &str, refresh_secret: &str) -> Self {
        JwtService {
            access_encoding_key: EncodingKey::from_secret(access_secret.as_bytes()),
            access_decoding_key: DecodingKey::from_secret(access_secret.as_bytes()),
            refresh_encoding_key: EncodingKey::from_secret(refresh_secret.as_bytes()),
            refresh_decoding_key: DecodingKey::from_secret(refresh_secret.as_bytes()),
        }
    }

    pub fn issue_access_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now().timestamp();
        let claims = AccessClaims {
            sub: user.get_uuid().to_string(),
            iat: now,
            exp: now + ACCESS_TOKEN_TTL_SECS,
            token_type: "access".to_string(),
            provider: user.get_provider().to_string(),
            email: Some(user.get_email().to_string()),
        };

        encode(&Header::new(jsonwebtoken::Algorithm::HS256), &claims, &self.access_encoding_key)
            .map_err(|err| AppError::jwt(&err.to_string()))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<AccessClaims, AppError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;

        let data = decode::<AccessClaims>(token, &self.access_decoding_key, &validation)
            .map_err(|_| AppError::invalid_token())?;

        // Ensure this is actually an access token and not a refresh token
        if data.claims.token_type != "access" {
            return Err(AppError::invalid_token());
        }

        Ok(data.claims)
    }


    pub fn issue_refresh_token(&self, user: &User) -> Result<(String, String), AppError> {
        let now = Utc::now().timestamp();
        let jti = Uuid::new_v4().to_string(); // unique token ID for revocation

        let claims = RefreshClaims {
            sub: user.get_uuid().to_string(),
            jti: jti.clone(),
            iat: now,
            exp: now + REFRESH_TOKEN_TTL_SECS,
            token_type: "refresh".to_string(),
        };

        let token = encode(&Header::new(jsonwebtoken::Algorithm::HS256), &claims, &self.refresh_encoding_key)
            .map_err(|err| AppError::jwt(&err.to_string()))?;

        Ok((token, jti))
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshClaims, AppError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = true;

        let data = decode::<RefreshClaims>(token, &self.refresh_decoding_key, &validation)
            .map_err(|_| AppError::invalid_token())?;

        if data.claims.token_type != "refresh" {
            return Err(AppError::invalid_token());
        }

        Ok(data.claims)
    }


}