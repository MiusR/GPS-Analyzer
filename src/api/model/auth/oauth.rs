use serde::{Deserialize, Serialize};

use crate::errors::domain_error::DomainError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    GitHub
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "google"),
            OAuthProvider::GitHub => write!(f, "github"),
        }
    }
}

impl TryFrom<String> for OAuthProvider {
    type Error = DomainError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
         match value.as_str() {
            "google" => Ok(OAuthProvider::Google),
            "github" => Ok(OAuthProvider::GitHub),
            val => Err(DomainError::illegal_data_format("provider", &format!("Provider [{}] not in known list of providers.", val)))
        }
    }
}


// Normalised user info returned from any OAuth provider
#[derive(Debug)]
pub struct ProviderUserInfo {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

// Google's userinfo endpoint response
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

// GitHub's user endpoint response
#[derive(Debug, Deserialize)]
pub struct GitHubUserInfo {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

// Callback data given by validation with oauth services
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}
