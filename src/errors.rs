use std::fmt;
use std::error;

#[derive(Debug)]
pub enum CircuitBreakerErrorKind {
    StateOpenError,
    TooManyRequestsError,
}

#[derive(Debug)]
pub struct CircuitBreakerError {
    pub kind: CircuitBreakerErrorKind,
    pub message: String,
}

impl error::Error for CircuitBreakerError {
    fn description(&self) -> &str {
        self.message.as_ref()
    }
}

impl fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
