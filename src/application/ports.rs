use crate::domain::models::ExportData;

/// Export service trait (interface)
pub trait ExportService: Send + Sync {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
