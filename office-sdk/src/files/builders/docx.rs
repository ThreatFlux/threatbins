use crate::OfficeDocument;
use crate::docx::DocxDocument;
use crate::error::Result;
use crate::files::builders::{BuilderConfig, DocumentBuilder};
use crate::files::{DocumentFormat, FileMetadata, FileResult};
use crate::markdown::MarkdownDocument;

#[derive(Debug, Clone)]
pub struct DocxBuilder {
    config: BuilderConfig,
}

impl DocxBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, config: &BuilderConfig) {
        self.config = config.clone();
    }

    pub fn from_markdown(
        &self,
        markdown_content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        let md_doc = MarkdownDocument::from_markdown(markdown_content)?;
        let docx_doc = md_doc.to_docx()?;

        let final_doc = self.apply_metadata(docx_doc, &metadata)?;

        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "docx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Docx,
            metadata,
        ))
    }

    pub fn from_text(&self, text_content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = DocxDocument::new();

        for line in text_content.lines() {
            if !line.trim().is_empty() {
                doc.add_paragraph(line)?;
            }
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;

        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "docx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Docx,
            metadata,
        ))
    }

    pub fn with_template(
        &self,
        template_name: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        match template_name {
            "business" => self.create_business_document(content, metadata),
            "report" => self.create_report_document(content, metadata),
            "letter" => self.create_letter_document(content, metadata),
            _ => self.from_text(content, metadata),
        }
    }

    fn create_business_document(
        &self,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        let mut doc = DocxDocument::new();

        if let Some(ref title) = metadata.title {
            doc.add_heading(1, title)?;
        }

        if let Some(ref author) = metadata.author {
            doc.add_paragraph(&format!("Prepared by: {}", author))?;
        }

        doc.add_paragraph("")?;

        for line in content.lines() {
            if !line.trim().is_empty() {
                doc.add_paragraph(line)?;
            }
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "docx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Docx,
            metadata,
        ))
    }

    fn create_report_document(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = DocxDocument::new();

        if let Some(ref title) = metadata.title {
            doc.add_heading(1, title)?;
        }

        doc.add_paragraph(&format!(
            "Generated on: {}",
            metadata.created.format("%Y-%m-%d %H:%M:%S UTC")
        ))?;
        doc.add_paragraph("")?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                doc.add_heading(1, &trimmed[2..])?;
            } else if trimmed.starts_with("## ") {
                doc.add_heading(2, &trimmed[3..])?;
            } else if trimmed.starts_with("### ") {
                doc.add_heading(3, &trimmed[4..])?
            } else if !trimmed.is_empty() {
                doc.add_paragraph(line)?;
            }
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "docx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Docx,
            metadata,
        ))
    }

    fn create_letter_document(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = DocxDocument::new();

        doc.add_paragraph(&format!("Date: {}", metadata.created.format("%B %d, %Y")))?;
        doc.add_paragraph("")?;

        for line in content.lines() {
            if !line.trim().is_empty() {
                doc.add_paragraph(line)?;
            }
        }

        doc.add_paragraph("")?;
        if let Some(ref author) = metadata.author {
            doc.add_paragraph("Sincerely,")?;
            doc.add_paragraph("")?;
            doc.add_paragraph(author)?;
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "docx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Docx,
            metadata,
        ))
    }

    fn apply_metadata(
        &self,
        mut doc: DocxDocument,
        metadata: &FileMetadata,
    ) -> Result<DocxDocument> {
        if let Some(ref title) = metadata.title {
            doc.metadata.title = Some(title.clone());
        }

        if let Some(ref author) = metadata.author {
            doc.metadata.author = Some(author.clone());
        }

        if let Some(ref subject) = metadata.subject {
            doc.metadata.subject = Some(subject.clone());
        }

        if let Some(ref description) = metadata.description {
            doc.metadata.description = Some(description.clone());
        }

        doc.metadata.keywords = Some(metadata.keywords.join(", "));
        doc.metadata.created = metadata.created;
        doc.metadata.modified = metadata.modified;

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

impl DocumentBuilder for DocxBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        if content.starts_with('#') || content.contains("## ") {
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
        vec!["text", "markdown", "md"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::builders::BuilderConfig;

    #[test]
    fn test_docx_builder_creation() {
        let config = BuilderConfig::default();
        let builder = DocxBuilder::new(&config);

        let supported = builder.supported_input_formats();
        assert!(supported.contains(&"text"));
        assert!(supported.contains(&"markdown"));
    }

    #[test]
    fn test_create_from_text() {
        let config = BuilderConfig::default();
        let builder = DocxBuilder::new(&config);
        let metadata = FileMetadata::default();

        let result = builder.create_from_content("Hello World", metadata);
        assert!(result.is_ok());
    }
}
