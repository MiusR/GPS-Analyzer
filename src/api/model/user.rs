use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{api::model::{auth::oauth::OAuthProvider, tier::Tier}, errors::domain_error::DomainError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    uuid : Uuid,
    username : UserName,
    email : UserEmail,
    tier : Tier,
    
    provider: OAuthProvider,
    provider_user_id: String,

    avatar_url: Option<String>,
    created_at: DateTime<Utc>,
}


// Validated 
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserName(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserEmail(String);


impl UserName {
    pub fn new(raw : &str) -> Result<Self, DomainError>   {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::empty_field("username"));
        }
        
        // Allowed charaters check
        // if let Some((index, invalid)) = trimmed.char_indices()
        //     .find(|&(_,c)| !(c.is_ascii_alphanumeric() || c == '_'))
        // {
        //     return Err(DomainError::illegal_character("username", invalid, index));
        // }

        Ok(UserName(trimmed.to_string()))
    }

    pub fn to_string(&self) -> String {
        return self.0.clone();
    } 
}


impl UserEmail {

    pub fn new(raw : &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        
        if trimmed.is_empty() {
            return Err(DomainError::empty_field("email"));
        }

        // char makeup check
        if let Some((index, invalid)) = trimmed.char_indices()
            .find(|&(_,c)| c == ' ' || !(c.is_ascii())) {
            return Err(DomainError::illegal_character("username", invalid, index));
        }

        // @ check
        if ! trimmed.contains('@') {
            return Err(DomainError::illegal_data_format("email", "does not contain @"));
        }

        let mut parts = trimmed.split('@');

        // local check
        match parts.next() {
            Some(v) if !v.is_empty() => v,
            _ => return Err(DomainError::illegal_data_format("email", "local part is empty")),
        };

        // domain check
        let domain = match parts.next() {
            Some(v) if !v.is_empty() => v,
            _ => return Err(DomainError::illegal_data_format("email", "domain part is empty")),
        };

        // @ count check
        if parts.next().is_some() {
            return Err(DomainError::illegal_data_format("email", "contains more then 1 @"));
        }

        // . check
        if !domain.contains('.') {
           return Err(DomainError::illegal_data_format("email", "domain does not contain '.'"));
        }

        Ok(UserEmail(trimmed.to_string()))
    }

    
    pub fn to_string(&self) -> String {
        return self.0.clone();
    } 

}

impl User {
    pub fn new(id : &str, name : &str, email : &str, tier : Tier, provider: OAuthProvider, provider_user_id: String, avatar_url: Option<String>) -> Result<Self, DomainError> {
        let validated_id =  Uuid::from_str(id).map_err(
            |err| {
                return DomainError::illegal_data_format("uuid", &err.to_string());
            }
        )?;
        let validated_name = UserName::new(name)?;
        let validated_email = UserEmail::new(email)?;

        Ok(User { uuid: validated_id, 
            username: validated_name, 
            email: validated_email, 
            tier : tier,
            avatar_url : avatar_url,
            created_at : Utc::now(),
            provider : provider,
            provider_user_id : provider_user_id
        })
    }

    pub fn get_name(&self) -> &UserName {
        return &self.username;
    }

    pub fn get_email(&self) -> &UserEmail {
        return &self.email;
    }

    pub fn get_uuid(&self) -> &Uuid {
        return &self.uuid;
    }

    pub fn get_tier(&self) -> &Tier {
        return &self.tier;
    }

    pub fn get_created_time(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn get_provider(&self) -> &OAuthProvider {
        &self.provider
    }

    pub fn get_provider_user_id(&self) -> &str {
        &self.provider_user_id
    }

    pub fn get_avatar_url(&self) -> &Option<String> {
        &self.avatar_url
    }



}
