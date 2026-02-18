use std::{error::Error, fmt::{Display, Result}};

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::errors::{domain_error::DomainError, io_errors::{IOError, IOErrorType}, service_errors::ServiceError};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AppError {
    pub etype: AppErrorType
}

impl AppError {
    pub fn io_error(io_error : IOError) -> Self {
        return AppError { etype: AppErrorType::IOError(io_error) }
    }

    pub fn service_error(service_error : ServiceError) -> Self {
        return AppError { etype: AppErrorType::ServiceError(service_error) }
    }

    pub fn domain_error(domain_error : DomainError) -> Self {
        return AppError { etype: AppErrorType::DomainError(domain_error) }
    }

    pub fn auth_error(reason : &str) -> Self {
        return AppError { etype: AppErrorType::AuthError(reason.to_string()) }
    }

    pub fn jwt(reason : &str) -> Self {
        return AppError { etype: AppErrorType::JWT(reason.to_string()) }
    }

    pub fn invalid_token() -> Self {
        return AppError { etype: AppErrorType::InvalidToken }
    }

    pub fn oauth2(reason : &str) -> Self {
        return AppError { etype: AppErrorType::OAuth2(reason.to_string()) }
    }

    pub fn token_revoked() -> Self {
        return AppError { etype: AppErrorType::TokenRevoked };
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "APP ERROR : {}", self.etype)
    }
}


impl Error for AppError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.etype)
	}
}


#[derive(Debug, Clone)]
pub enum AppErrorType {
    DomainError(DomainError),
    IOError(IOError),
    ServiceError(ServiceError),
    AuthError(String), // Reason
    JWT(String),
    InvalidToken,
    OAuth2(String), // Reason
    TokenRevoked
}

impl Display for AppErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        match &self {
            AppErrorType::DomainError(domain) => {
                write!(f, "{}", domain.to_string())
            },
            AppErrorType::ServiceError(domain) => {
                write!(f, "{}", domain.to_string())
            },
            AppErrorType::IOError(domain) => {
                write!(f, "{}", domain.to_string())
            },
            AppErrorType::AuthError(reason) => {
                write!(f, "failed to authenticate : {}", reason)
            },
            AppErrorType::JWT(reason ) => {
                write!(f, "JWT error : {}", reason)
            },
            AppErrorType::InvalidToken => {
                write!(f, "Invalid token given")
            },
            AppErrorType::OAuth2(reason) => {
                write!(f, "provider error : {}", reason)
            },
            AppErrorType::TokenRevoked => {
                write!(f, "Due to security concerns your token has been revoked. Try to login again!")
            }
        }
    }
}

impl Error for AppErrorType {}

fn map_io_to_status_code(kind : &IOErrorType) -> StatusCode {
    match kind {
                    crate::errors::io_errors::IOErrorType::FormatNotSupported(_) => StatusCode::BAD_REQUEST,
                    crate::errors::io_errors::IOErrorType::RecordNotFound(_) => StatusCode::NOT_FOUND,
                    crate::errors::io_errors::IOErrorType::InvalidPath(_) => StatusCode::BAD_REQUEST,
                    crate::errors::io_errors::IOErrorType::RecordOperation(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    crate::errors::io_errors::IOErrorType::StreamError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    crate::errors::io_errors::IOErrorType::XmlParsingFail(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    crate::errors::io_errors::IOErrorType::XmlReaderFail(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    crate::errors::io_errors::IOErrorType::DomainError(_) => StatusCode::BAD_REQUEST
    }
}


impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.etype {
            AppErrorType::DomainError(_) => {
                StatusCode::BAD_REQUEST
            },
            AppErrorType::IOError(io_error) => {
                map_io_to_status_code(&io_error.etype)
            },
            AppErrorType::ServiceError(service_error) => {
                match &service_error.etype {
                    crate::errors::service_errors::ServiceErrorType::CoordinateConversionError(_, _, _, _) => StatusCode::INTERNAL_SERVER_ERROR,
                    crate::errors::service_errors::ServiceErrorType::EmptyTrack() => StatusCode::BAD_REQUEST,
                    crate::errors::service_errors::ServiceErrorType::IOError(err) => map_io_to_status_code(&err.etype),
                    crate::errors::service_errors::ServiceErrorType::InvalidData(_) => StatusCode::BAD_REQUEST,
                    crate::errors::service_errors::ServiceErrorType::TrackSnappingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            },
            AppErrorType::AuthError(_) => {
                StatusCode::FORBIDDEN
            },
            AppErrorType::JWT(_) => {
                StatusCode::UNAUTHORIZED
            },
            AppErrorType::OAuth2(_) => {
                StatusCode::FAILED_DEPENDENCY
            },
            AppErrorType::TokenRevoked => {
                StatusCode::UNAUTHORIZED
            }
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));

        (status, body).into_response()
    }
}