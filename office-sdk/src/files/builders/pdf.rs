use crate::OfficeDocument;
use crate::error::Result;
use crate::files::builders::{BuilderConfig, DocumentBuilder};
use crate::files::{DocumentFormat, FileMetadata, FileResult};
use crate::pdf::PdfDocument;

#[derive(Debug, Clone)]
pub struct PdfBuilder {
    config: BuilderConfig,
}

impl PdfBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, config: &BuilderConfig) {
        self.config = config.clone();
    }

    pub fn from_text(&self, text_content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = PdfDocument::new();

        // Add a page
        let _page_id = doc.add_a4_page()?;

        // Add title if available
        if let Some(ref title) = metadata.title {
            doc.add_heading(0, 1, title, 50.0, 750.0)?;
        }

        // Add content
        let mut y_position = 700.0;
        for line in text_content.lines() {
            if !line.trim().is_empty() {
                doc.add_text(0, line, 50.0, y_position, "Helvetica", 12.0)?;
                y_position -= 20.0;

                // Start new page if we're near the bottom
                if y_position < 50.0 {
                    doc.add_a4_page()?;
                    y_position = 750.0;
                }
            }
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "pdf");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Pdf,
            metadata,
        ))
    }

    pub fn from_markdown(
        &self,
        markdown_content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        let mut doc = PdfDocument::new();
        let _page_id = doc.add_a4_page()?;

        let mut y_position = 750.0;
        let mut page_index = 0;

        for line in markdown_content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with("# ") {
                doc.add_heading(page_index, 1, &line[2..], 50.0, y_position)?;
                y_position -= 30.0;
            } else if line.starts_with("## ") {
                doc.add_heading(page_index, 2, &line[3..], 50.0, y_position)?;
                y_position -= 25.0;
            } else if line.starts_with("### ") {
                doc.add_heading(page_index, 3, &line[4..], 50.0, y_position)?;
                y_position -= 20.0;
            } else {
                doc.add_text(page_index, line, 50.0, y_position, "Helvetica", 12.0)?;
                y_position -= 15.0;
            }

            // Start new page if needed
            if y_position < 50.0 {
                doc.add_a4_page()?;
                page_index += 1;
                y_position = 750.0;
            }
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "pdf");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Pdf,
            metadata,
        ))
    }

    pub fn with_template(
        &self,
        _template_name: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        // For simplicity, just use basic text formatting for all templates
        self.from_text(content, metadata)
    }

    fn apply_metadata(&self, mut doc: PdfDocument, metadata: &FileMetadata) -> Result<PdfDocument> {
        if let Some(ref title) = metadata.title {
            doc.set_title(title);
        }

        if let Some(ref author) = metadata.author {
            doc.set_author(author);
        }

        if let Some(ref subject) = metadata.subject {
            doc.set_subject(subject);
        }

        if let Some(ref description) = metadata.description {
            doc.set_description(description);
        }

        doc.set_keywords(&metadata.keywords.join(", "));
        doc.created = metadata.created;
        doc.modified = metadata.modified;

        Ok(doc)
    }

    fn generate_filename(&self, metadata: &FileMetadata, extension: &str) -> String {
        let base_name = metadata
            .title
            .as_ref()
            .map(|t| t.replace(" ", "_").to_lowercase())
            .unwrap_or_else(|| "document".to_string());

        format!("{}.{}", base_name, extension)
    }
}

impl DocumentBuilder for PdfBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        if content.starts_with('#') || content.contains("##") {
            self.from_markdown(content, metadata)
        } else {
            self.from_text(content, metadata)
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
        vec!["text", "markdown"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::builders::BuilderConfig;

    #[test]
    fn test_pdf_builder_creation() {
        let config = BuilderConfig::default();
        let builder = PdfBuilder::new(&config);

        let supported = builder.supported_input_formats();
        assert!(supported.contains(&"text"));
        assert!(supported.contains(&"markdown"));
    }

    #[test]
    fn test_create_from_text() {
        let config = BuilderConfig::default();
        let builder = PdfBuilder::new(&config);
        let metadata = FileMetadata::default();

        let text_content = "This is a test document.\nWith multiple lines.";
        let result = builder
            .create_from_content(text_content, metadata)
            .expect("PDF generation should succeed");
        let parsed = lopdf::Document::load_mem(&result.content)
            .expect("generated bytes should be a valid PDF");

        assert_eq!(parsed.get_pages().len(), 1);
        assert!(
            parsed
                .extract_text(&[1])
                .expect("generated page text should be readable")
                .contains("This is a test document.")
        );
    }
}
