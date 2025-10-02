use crate::OfficeDocument;
use crate::error::Result;
use crate::files::builders::{BuilderConfig, DocumentBuilder};
use crate::files::{DocumentFormat, FileMetadata, FileResult};
use crate::xlsx::XlsxDocument;

#[derive(Debug, Clone)]
pub struct XlsxBuilder {
    config: BuilderConfig,
}

impl XlsxBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, config: &BuilderConfig) {
        self.config = config.clone();
    }

    pub fn from_csv_data(&self, csv_content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = XlsxDocument::new();

        // Parse CSV content into data structure
        let lines: Vec<&str> = csv_content.lines().collect();
        let mut data: Vec<Vec<String>> = Vec::new();

        for line in lines {
            let cells: Vec<String> = line
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect();
            data.push(cells);
        }

        doc.add_worksheet("Sheet1", &data)?;

        let content = doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "xlsx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Xlsx,
            metadata,
        ))
    }

    pub fn with_template(
        &self,
        _template_name: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        // For now, just use basic CSV parsing for all templates
        self.from_csv_data(content, metadata)
    }

    fn generate_filename(&self, metadata: &FileMetadata, extension: &str) -> String {
        let base_name = metadata
            .title
            .as_ref()
            .map(|t| t.replace(" ", "_").to_lowercase())
            .unwrap_or_else(|| "spreadsheet".to_string());

        format!("{}.{}", base_name, extension)
    }
}

impl DocumentBuilder for XlsxBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        // Try to detect if it's CSV-like content
        if content.contains(',') || content.contains('\t') {
            self.from_csv_data(content, metadata)
        } else {
            // Convert text lines to simple table
            let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            let mut data: Vec<Vec<String>> = Vec::new();

            for line in lines {
                data.push(vec![line.to_string()]);
            }

            let mut doc = XlsxDocument::new();
            doc.add_worksheet("Sheet1", &data)?;

            let content = doc.to_bytes()?;
            let filename = self.generate_filename(&metadata, "xlsx");

            Ok(FileResult::new(
                content,
                filename,
                DocumentFormat::Xlsx,
                metadata,
            ))
        }
    }

    fn create_from_template(
        &self,
        template: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        self.with_template(template, content, metadata)
    }

    fn supported_input_formats(&self) -> Vec<&'static str> {
        vec!["csv", "text", "table"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::builders::BuilderConfig;

    #[test]
    fn test_xlsx_builder_creation() {
        let config = BuilderConfig::default();
        let builder = XlsxBuilder::new(&config);

        let supported = builder.supported_input_formats();
        assert!(supported.contains(&"csv"));
        assert!(supported.contains(&"text"));
    }

    #[test]
    fn test_create_from_csv() {
        let config = BuilderConfig::default();
        let builder = XlsxBuilder::new(&config);
        let metadata = FileMetadata::default();

        let csv_content = "Name,Age,City\nJohn,30,NYC\nJane,25,LA";
        let result = builder.create_from_content(csv_content, metadata);
        assert!(result.is_ok());
    }
}
