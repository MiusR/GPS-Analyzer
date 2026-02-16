use std::{error::Error, fmt::{Display, Result}};

use crate::errors::io_errors::IOError;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ServiceError {
    pub etype: ServiceErrorType
}

impl ServiceError {
    pub fn coordinate_conversion(origin : &str, destination : &str, original_coordiante : &str, reason : &str) -> Self {
        return ServiceError { etype : ServiceErrorType::CoordinateConversionError(origin.to_string(), destination.to_string(), original_coordiante.to_string(), reason.to_string())};
    }

    pub fn io_error(io_error : IOError) -> Self {
        return ServiceError { etype: ServiceErrorType::IOError(io_error) }
    }

    pub fn invalid_data(reason : &str) -> Self {
        return ServiceError { etype: ServiceErrorType::InvalidData(reason.to_string()) }
    }

    pub fn empty_track() -> Self {
        return ServiceError { etype: ServiceErrorType::EmptyTrack() }
    }

    pub fn track_snapping_error(reason : &str) -> Self {
        return ServiceError { etype: ServiceErrorType::TrackSnappingError(reason.to_string()) }
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "SERVICE ERROR : {}", self.etype)
    }
}


impl Error for ServiceError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.etype)
	}
}


#[derive(Debug, Clone)]
pub enum ServiceErrorType {
    // Origin, Destination, Original Point, Reason
    CoordinateConversionError(String, String, String, String), 

    // Composed Error
    IOError(IOError),

    // Reason
    InvalidData(String),

    EmptyTrack(),

    // Reason
    TrackSnappingError(String),
}

impl Display for ServiceErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        match &self {
            ServiceErrorType::CoordinateConversionError(origin, destination, original_point, reason) =>
                write!(f, "failed to convert point [{}] from {} to {}: {}", original_point, origin, destination, reason),
            ServiceErrorType::IOError(err) =>
                write!(f, "{}", err.to_string()),
            ServiceErrorType::InvalidData(err) =>
                write!(f, "invalid data format : {}", err.to_string()),
            ServiceErrorType::EmptyTrack() =>
                write!(f, "Tried to process an empty track"),
            ServiceErrorType::TrackSnappingError(err) =>
                write!(f, "failed to snap tracks : {}", err)
        }
    }
}

impl Error for ServiceErrorType {}
