use rust_xlsxwriter::*;
use crate::application::ports::ExportService;
use crate::domain::models::ExportData;

pub struct ExcelExporter;

impl ExportService for ExcelExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Write headers (row 0)
        for (col, header) in data.headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, header)?;
            worksheet.set_column_width(col as u16, 20)?;
        }

        // Write data rows
        for (row_idx, row) in data.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                worksheet.write_string((row_idx + 1) as u32, col_idx as u16, cell)?;
            }
        }

        // Apply options
        if let Some(opts) = &data.options {
            if opts.freeze_headers.unwrap_or(false) {
                worksheet.set_freeze_panes(1, 0)?;
            }
        }

        // Return as bytes
        workbook
            .save_to_buffer()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
