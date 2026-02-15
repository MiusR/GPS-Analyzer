use std::{error::Error, fmt::{Display, Result}};

use crate::errors::domain_error::{DomainError};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct IOError {
    pub source: String,
    pub etype: IOErrorType
}

impl IOError {
    pub fn xml_reader(source : &str, reason : &str) -> Self {
        return IOError { etype : IOErrorType::XmlReaderFail(reason.to_string()), source : source.to_string()};
    }

    pub fn xml_parser(source: &str, reason : &str) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::XmlParsingFail(reason.to_string())}
    }

    pub fn format_not_supported(source : &str, reason : &str) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::FormatNotSupported(reason.to_string()) }
    }

    pub fn invalid_path(source : &str, reason : &str) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::InvalidPath(reason.to_string()) }
    }

    pub fn record_not_fround(source : &str, reason : &str) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::RecordNotFound(reason.to_string()) }
    }

    pub fn record_operation(source : &str, reason : &str) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::RecordOperation(reason.to_string()) }
    }

    pub fn domain_error(source : &str, reason : DomainError) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::DomainError(reason) }
    }

    pub fn stream_error(source : &str, reason : &str) -> Self {
        return IOError { source: source.to_string(), etype: IOErrorType::StreamError(reason.to_string()) }
    }
}

impl Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "IO ERROR [{}] : {}", self.source, self.etype)
    }
}


impl Error for IOError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.etype)
	}
}


#[derive(Debug, Clone)]
pub enum IOErrorType {
    XmlReaderFail(String), // Reason

    XmlParsingFail(String), // Reason

    FormatNotSupported(String), // Reason
    
    InvalidPath(String), // Reason

    RecordNotFound(String), // Reason

    RecordOperation(String), // Reason

    DomainError(DomainError), // Wrapper for domain Error

    StreamError(String), // Reason
}

impl Display for IOErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        match &self {
            IOErrorType::XmlReaderFail(reason) =>
                write!(f, "Failed to read xml file : {}", reason),
            IOErrorType::XmlParsingFail(reason) =>
                write!(f, "Failed to parse xml file : {}", reason),
            IOErrorType::FormatNotSupported(reason) =>
                write!(f, "File format not supported : {}", reason),
            IOErrorType::InvalidPath(reason) =>
                write!(f, "Invalid file path : {}", reason),
            IOErrorType::RecordNotFound(reason) =>
                write!(f, "Record not found : {}", reason),
            IOErrorType::RecordOperation(reason) =>
                write!(f, "Failed to execute operation on record : {}", reason),
            IOErrorType::DomainError(reason) =>
                write!(f, "{}", reason),
            IOErrorType::StreamError(reason) =>
                write!(f, "Failed to operate on stream : {}", reason)
        }
    }
}

impl Error for IOErrorType {}
