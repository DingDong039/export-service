use serde::{Deserialize, Serialize};
use crate::domain::models::{ColumnMetadata, ExportData, ExportFormat, ExportOptions};

/// HTTP request DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub title: String,
    pub format: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    #[serde(default)]
    pub options: Option<ExportOptions>,
    /// Optional column metadata for proper formatting (alignment, width hints)
    #[serde(default)]
    pub column_metadata: Option<Vec<ColumnMetadata>>,
}

impl ExportRequest {
    /// Convert to domain model
    pub fn to_domain(&self) -> Result<ExportData, String> {
        let format = match self.format.to_lowercase().as_str() {
            "excel" => ExportFormat::Excel,
            "csv" => ExportFormat::Csv,
            "pdf" => ExportFormat::Pdf,
            _ => return Err(format!("Invalid format: {}", self.format)),
        };

        Ok(ExportData {
            title: self.title.clone(),
            format,
            headers: self.headers.clone(),
            rows: self.rows.clone(),
            options: self.options.clone(),
            column_metadata: self.column_metadata.clone(),
        })
    }
}
