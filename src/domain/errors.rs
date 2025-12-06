use std::fmt;

/// Domain-level errors
#[derive(Debug)]
pub enum DomainError {
    InvalidFormat(String),
    EmptyData(String),
    ColumnCountMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    CellTooLong(usize),
    TooManyRows(usize),
    InvalidToken,
    TokenExpired,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DomainError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            DomainError::EmptyData(msg) => write!(f, "Empty data: {}", msg),
            DomainError::ColumnCountMismatch {
                row,
                expected,
                actual,
            } => write!(
                f,
                "Row {}: column count mismatch (expected {}, got {})",
                row, expected, actual
            ),
            DomainError::CellTooLong(len) => write!(f, "Cell content too long: {} chars", len),
            DomainError::TooManyRows(count) => write!(f, "Too many rows: {} (max 10000)", count),
            DomainError::InvalidToken => write!(f, "Invalid token"),
            DomainError::TokenExpired => write!(f, "Token expired"),
        }
    }
}

impl std::error::Error for DomainError {}
