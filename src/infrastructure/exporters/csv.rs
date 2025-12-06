use csv::Writer;
use crate::application::ports::ExportService;
use crate::domain::models::ExportData;

pub struct CsvExporter;

impl ExportService for CsvExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        {
            let mut writer = Writer::from_writer(&mut buffer);

            // Write headers
            writer.write_record(&data.headers)?;

            // Write rows
            for row in &data.rows {
                writer.write_record(row)?;
            }

            writer.flush()?;
        }
        Ok(buffer)
    }
}
