use super::models::ExportData;
use super::errors::DomainError;

/// Validator trait
pub trait ExportValidator: Send + Sync {
    fn validate(&self, data: &ExportData) -> Result<(), DomainError>;
}

/// Default validator implementation
pub struct DefaultExportValidator;

impl ExportValidator for DefaultExportValidator {
    fn validate(&self, data: &ExportData) -> Result<(), DomainError> {
        // Check headers
        if data.headers.is_empty() {
            return Err(DomainError::EmptyData("Headers cannot be empty".to_string()));
        }

        // Check rows
        if data.rows.is_empty() {
            return Err(DomainError::EmptyData("Data rows cannot be empty".to_string()));
        }

        // Check row count limit
        if data.rows.len() > 10000 {
            return Err(DomainError::TooManyRows(data.rows.len()));
        }

        let header_count = data.headers.len();

        // Validate each row
        for (i, row) in data.rows.iter().enumerate() {
            // Column count match
            if row.len() != header_count {
                return Err(DomainError::ColumnCountMismatch {
                    row: i + 1,
                    expected: header_count,
                    actual: row.len(),
                });
            }

            // Cell length check
            for cell in row.iter() {
                if cell.len() > 1000 {
                    return Err(DomainError::CellTooLong(cell.len()));
                }
            }
        }

        // Check header length
        for header in &data.headers {
            if header.len() > 1000 {
                return Err(DomainError::CellTooLong(header.len()));
            }
        }

        Ok(())
    }
}
