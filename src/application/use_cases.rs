use std::sync::Arc;
use crate::domain::models::{ExportData, ExportFormat};
use crate::domain::validators::ExportValidator;
use crate::domain::errors::DomainError;
use super::ports::ExportService;

/// Main export use case
pub struct ExportUseCase {
    validator: Arc<dyn ExportValidator>,
    excel_service: Arc<dyn ExportService>,
    csv_service: Arc<dyn ExportService>,
    pdf_service: Arc<dyn ExportService>,
}

impl ExportUseCase {
    pub fn new(
        validator: Arc<dyn ExportValidator>,
        excel_service: Arc<dyn ExportService>,
        csv_service: Arc<dyn ExportService>,
        pdf_service: Arc<dyn ExportService>,
    ) -> Self {
        Self {
            validator,
            excel_service,
            csv_service,
            pdf_service,
        }
    }

    /// Execute export
    pub fn execute(&self, data: ExportData) -> Result<Vec<u8>, DomainError> {
        // Step 1: Validate data
        self.validator.validate(&data)?;

        // Step 2: Select appropriate service
        let service = match data.format {
            ExportFormat::Excel => self.excel_service.clone(),
            ExportFormat::Csv => self.csv_service.clone(),
            ExportFormat::Pdf => self.pdf_service.clone(),
        };

        // Step 3: Export and return binary data
        service
            .export(&data)
            .map_err(|e| DomainError::InvalidFormat(e.to_string()))
    }
}
