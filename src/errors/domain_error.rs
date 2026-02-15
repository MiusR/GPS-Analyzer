use std::{error::Error, fmt::{Display, Result}};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct DomainError {
    pub kind: DomainErrorKind
}

impl DomainError {
    pub fn empty_field(field_name : &str) -> Self {
        return DomainError { kind: DomainErrorKind::EmptyField(field_name.to_string()) };
    }

    pub fn illegal_character(field_name : &str, illegal_char : char, position : usize) -> Self {
        return DomainError { kind: DomainErrorKind::IllegalCharacter(field_name.to_string(), illegal_char, position)};
    }

    pub fn illegal_data_format(field_name : &str, reason : &str) -> Self {
        return DomainError { kind: DomainErrorKind::IllegalDataFormat(field_name.to_string(), reason.to_string()) };
    }

}

impl Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "provided invalid domain data -> {}", self.kind)
    }
}


impl Error for DomainError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.kind)
	}
}


#[derive(Debug, Clone)]
pub enum DomainErrorKind {
    // Field name
    EmptyField(String),
    
    //Field name, invalid character, position
    IllegalCharacter(String, char, usize),
    
    //Field name, reason
    IllegalDataFormat(String, String)
}

impl Display for DomainErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        match &self {
            DomainErrorKind::EmptyField(field) =>
                write!(f, "empty field: {}", field),
            DomainErrorKind::IllegalCharacter(field, illegal_char, position ) =>
                write!(f, "found illegal character '{}' in field {} at position {}", illegal_char, field, position),
            DomainErrorKind::IllegalDataFormat(field, reason) => 
                write!(f, "data in field {} does not match expected pattern: {}", field, reason),
        }
    }
}

impl Error for DomainErrorKind {}
