use std::error::Error;
use std::fmt;

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum TokenType{
    Instruction,
    Operand,
    Label,
}
#[derive(Debug, Clone)]
pub struct AssemblyError{
    pub line_number: usize,
    pub line_text: String,
    pub message: String
}

impl Error for AssemblyError {}
impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error in line {} - {}:\n{}", self.line_number, self.line_text, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidTokenError {
    pub token: String,
    pub token_type: TokenType,
    pub additional_info: Option<String>
}

impl Error for InvalidTokenError {}
impl fmt::Display for InvalidTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut error_message: String;
        match self.token_type {
            TokenType::Instruction => error_message = "Invalid instruction".into(),
            TokenType::Operand => error_message = "Invalid operand".into(),
            TokenType::Label => error_message = "Invalid label".into(),
        }
        error_message.push_str(&format!(": {}.", self.token));
        if let Some(x) = &self.additional_info {
            error_message.push_str(&format!("Additional info: {}", x));
        }
        write!(f, "{}", error_message)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidTokenAtLineError {
    pub line: usize,
    pub source: InvalidTokenError,
}

#[derive(Debug, Clone)]
pub struct OverflowError;

impl Error for OverflowError {}
impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory overflow")
    }
}

#[derive(Debug, Clone)]
pub enum TokenOrOverflowError {
    InvalidToken(InvalidTokenError),
    Overflow(OverflowError)
}

impl From<InvalidTokenError> for TokenOrOverflowError {
    fn from(err: InvalidTokenError) -> Self {
        TokenOrOverflowError::InvalidToken(err)
    }
}

impl From<OverflowError> for TokenOrOverflowError {
    fn from(err: OverflowError) -> Self {
        TokenOrOverflowError::Overflow(err)
    }
}