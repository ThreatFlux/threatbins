use crate::error::Result;
use crate::files::{FileMetadata, FileResult};

pub mod docx;
pub mod eml;
pub mod markdown;
pub mod pdf;
pub mod pptx;
pub mod xlsx;

pub use docx::*;
pub use eml::*;
pub use markdown::*;
pub use pdf::*;
pub use pptx::*;
pub use xlsx::*;

pub trait DocumentBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult>;
    fn create_from_template(
        &self,
        template: &str,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult>;
    fn supported_input_formats(&self) -> Vec<&'static str>;
}

#[derive(Debug, Clone)]
pub struct BuilderConfig {
    pub default_author: Option<String>,
    pub default_template: Option<String>,
    pub enable_macros: bool,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Strict,
    Standard,
    Permissive,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            default_author: None,
            default_template: None,
            enable_macros: false,
            security_level: SecurityLevel::Standard,
        }
    }
}

pub struct DocumentBuilders {
    pub docx: DocxBuilder,
    pub xlsx: XlsxBuilder,
    pub pptx: PptxBuilder,
    pub pdf: PdfBuilder,
    pub eml: EmlBuilder,
    pub markdown: MarkdownBuilder,
    pub config: BuilderConfig,
}

impl DocumentBuilders {
    pub fn new() -> Self {
        Self::with_config(BuilderConfig::default())
    }

    pub fn with_config(config: BuilderConfig) -> Self {
        Self {
            docx: DocxBuilder::new(&config),
            xlsx: XlsxBuilder::new(&config),
            pptx: PptxBuilder::new(&config),
            pdf: PdfBuilder::new(&config),
            eml: EmlBuilder::new(&config),
            markdown: MarkdownBuilder::new(&config),
            config,
        }
    }

    pub fn update_config(&mut self, config: BuilderConfig) {
        self.config = config.clone();
        self.docx.update_config(&config);
        self.xlsx.update_config(&config);
        self.pptx.update_config(&config);
        self.pdf.update_config(&config);
        self.eml.update_config(&config);
        self.markdown.update_config(&config);
    }
}

impl Default for DocumentBuilders {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builders_creation() {
        let builders = DocumentBuilders::new();
        assert!(matches!(
            builders.config.security_level,
            SecurityLevel::Standard
        ));
    }

    #[test]
    fn test_config_update() {
        let mut builders = DocumentBuilders::new();
        let mut config = BuilderConfig::default();
        config.security_level = SecurityLevel::Strict;

        builders.update_config(config);
        assert!(matches!(
            builders.config.security_level,
            SecurityLevel::Strict
        ));
    }
}
