use crate::OfficeDocument;
use crate::eml::EmlDocument;
use crate::error::Result;
use crate::files::builders::{BuilderConfig, DocumentBuilder};
use crate::files::{DocumentFormat, FileMetadata, FileResult};

#[derive(Debug, Clone)]
pub struct EmlBuilder {
    config: BuilderConfig,
}

impl EmlBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, config: &BuilderConfig) {
        self.config = config.clone();
    }

    pub fn create_email(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut doc = EmlDocument::new();

        if let Some(ref author) = metadata.author {
            doc.set_from(author, None)?;
        }

        if let Some(ref subject) = metadata.subject {
            doc.set_subject(subject);
        }

        doc.set_text_body(content);

        let final_content = doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "eml");

        Ok(FileResult::new(
            final_content,
            filename,
            DocumentFormat::Eml,
            metadata,
        ))
    }

    pub fn with_template(
        &self,
        _template_name: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        // For simplicity, just use basic email creation for all templates
        self.create_email(content, metadata)
    }

    fn generate_filename(&self, metadata: &FileMetadata, extension: &str) -> String {
        let base_name = metadata
            .subject
            .as_ref()
            .or(metadata.title.as_ref())
            .map(|t| t.replace(" ", "_").to_lowercase())
            .unwrap_or_else(|| "email".to_string());

        format!("{}.{}", base_name, extension)
    }
}

impl DocumentBuilder for EmlBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        self.create_email(content, metadata)
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
        vec!["text", "email"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::builders::BuilderConfig;

    #[test]
    fn test_eml_builder_creation() {
        let config = BuilderConfig::default();
        let builder = EmlBuilder::new(&config);

        let supported = builder.supported_input_formats();
        assert!(supported.contains(&"text"));
        assert!(supported.contains(&"email"));
    }

    #[test]
    fn test_create_email() {
        let config = BuilderConfig::default();
        let builder = EmlBuilder::new(&config);
        let mut metadata = FileMetadata::default();
        metadata.subject = Some("Test Email".to_string());

        let content = "This is a test email body.";
        let result = builder.create_from_content(content, metadata);
        assert!(result.is_ok());
    }
}
