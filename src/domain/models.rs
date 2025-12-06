use serde::{Deserialize, Serialize};

/// Column data type for proper formatting and alignment
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    #[default]
    Text,
    Number,
    Currency,
    Percentage,
    Date,
}

impl ColumnType {
    /// Returns true if this column type should be right-aligned
    pub fn is_right_aligned(&self) -> bool {
        matches!(self, Self::Number | Self::Currency | Self::Percentage)
    }
}

/// Metadata for a single column
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ColumnMetadata {
    /// Column data type (affects alignment and formatting)
    #[serde(default)]
    pub column_type: ColumnType,
    /// Optional custom width hint (percentage or fixed)
    pub width_hint: Option<f32>,
}

impl ColumnMetadata {
    pub fn text() -> Self {
        Self { column_type: ColumnType::Text, width_hint: None }
    }

    pub fn number() -> Self {
        Self { column_type: ColumnType::Number, width_hint: None }
    }

    pub fn currency() -> Self {
        Self { column_type: ColumnType::Currency, width_hint: None }
    }

    pub fn percentage() -> Self {
        Self { column_type: ColumnType::Percentage, width_hint: None }
    }

    pub fn date() -> Self {
        Self { column_type: ColumnType::Date, width_hint: None }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width_hint = Some(width);
        self
    }
}

/// Main export data structure
#[derive(Debug, Clone)]
pub struct ExportData {
    pub title: String,
    pub format: ExportFormat,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub options: Option<ExportOptions>,
    /// Optional column metadata for proper formatting
    /// If None or shorter than headers, defaults are used
    pub column_metadata: Option<Vec<ColumnMetadata>>,
}

/// Export format types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Excel,
    Csv,
    Pdf,
}

impl ExportFormat {
    /// Get file extension
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Excel => "xlsx",
            ExportFormat::Csv => "csv",
            ExportFormat::Pdf => "pdf",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &str {
        match self {
            ExportFormat::Excel => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            ExportFormat::Csv => "text/csv",
            ExportFormat::Pdf => "application/pdf",
        }
    }
}

/// Export options for formatting
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportOptions {
    pub freeze_headers: Option<bool>,
    pub auto_fit_columns: Option<bool>,
    pub header_bold: Option<bool>,
    pub header_background: Option<String>,
    pub include_header_row: Option<bool>,
    pub delimiter: Option<String>,
}
