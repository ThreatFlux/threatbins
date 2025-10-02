use crate::OfficeDocument;
use crate::error::Result;
use crate::files::builders::{BuilderConfig, DocumentBuilder};
use crate::files::{DocumentFormat, FileMetadata, FileResult};
use crate::pptx::PptxDocument;

#[derive(Debug, Clone)]
pub struct PptxBuilder {
    config: BuilderConfig,
}

impl PptxBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, config: &BuilderConfig) {
        self.config = config.clone();
    }

    pub fn from_text(&self, text_content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = PptxDocument::new();

        if let Some(ref title) = metadata.title {
            let author = metadata.author.as_ref().map(|s| s.as_str()).unwrap_or("");
            doc.add_title_slide(title, author)?;
        }

        let slides: Vec<&str> = text_content.split("\n---\n").collect();
        for slide_content in slides {
            if !slide_content.trim().is_empty() {
                let lines: Vec<&str> = slide_content.lines().collect();
                let title = if !lines.is_empty() { lines[0] } else { "Slide" };
                let content = if lines.len() > 1 { &lines[1..] } else { &[] };
                doc.add_slide(title, content)?;
            }
        }

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "pptx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Pptx,
            metadata,
        ))
    }

    pub fn from_markdown(
        &self,
        markdown_content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        let mut doc = PptxDocument::new();

        if let Some(ref title) = metadata.title {
            let author = metadata.author.as_ref().map(|s| s.as_str()).unwrap_or("");
            doc.add_title_slide(title, author)?;
        }

        self.process_markdown_slides(&mut doc, markdown_content)?;

        let final_doc = self.apply_metadata(doc, &metadata)?;
        let content = final_doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "pptx");

        Ok(FileResult::new(
            content,
            filename,
            DocumentFormat::Pptx,
            metadata,
        ))
    }

    pub fn with_template(
        &self,
        _template_name: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        // For simplicity, just use basic slide creation for all templates
        self.from_text(content, metadata)
    }

    fn process_markdown_slides(&self, doc: &mut PptxDocument, content: &str) -> Result<()> {
        let sections: Vec<&str> = content.split("\n---\n").collect();

        for section in sections {
            if section.trim().is_empty() {
                continue;
            }

            let lines: Vec<&str> = section.lines().collect();
            if lines.is_empty() {
                continue;
            }

            let first_line = lines[0].trim();
            if first_line.starts_with("# ") {
                let title = &first_line[2..];
                let content_lines = if lines.len() > 1 { &lines[1..] } else { &[] };
                doc.add_slide(title, content_lines)?;
            } else {
                let content_text = lines.join("\n");
                let content_lines: Vec<&str> = content_text.lines().collect();
                doc.add_slide("Content", &content_lines)?;
            }
        }

        Ok(())
    }

    fn apply_metadata(
        &self,
        mut doc: PptxDocument,
        metadata: &FileMetadata,
    ) -> Result<PptxDocument> {
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
            .unwrap_or_else(|| "presentation".to_string());

        format!("{}.{}", base_name, extension)
    }
}

impl DocumentBuilder for PptxBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        if content.contains("---") || content.starts_with('#') {
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
        vec!["text", "markdown", "slides"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::builders::BuilderConfig;

    #[test]
    fn test_pptx_builder_creation() {
        let config = BuilderConfig::default();
        let builder = PptxBuilder::new(&config);

        let supported = builder.supported_input_formats();
        assert!(supported.contains(&"text"));
        assert!(supported.contains(&"slides"));
    }

    #[test]
    fn test_create_from_slides() {
        let config = BuilderConfig::default();
        let builder = PptxBuilder::new(&config);
        let metadata = FileMetadata::default();

        let slide_content = "# Title\nContent\n---\n# Second Slide\nMore content";
        let result = builder.create_from_content(slide_content, metadata);
        assert!(result.is_ok());
    }
}
