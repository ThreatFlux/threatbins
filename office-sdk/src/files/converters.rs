use crate::OfficeDocument;
use crate::docx::DocxDocument;
use crate::eml::EmlDocument;
use crate::error::{OfficeError, Result};
use crate::files::builders::{BuilderConfig, DocumentBuilder, DocumentBuilders};
use crate::files::{DocumentFormat, FileInput, FileMetadata, FileResult};
use crate::pdf::PdfDocument;
use crate::pptx::PptxDocument;
use crate::xlsx::XlsxDocument;
use std::path::Path;

pub struct DocumentConverters {
    builders: DocumentBuilders,
}

impl DocumentConverters {
    pub fn new() -> Self {
        Self {
            builders: DocumentBuilders::new(),
        }
    }

    pub fn with_config(config: BuilderConfig) -> Self {
        Self {
            builders: DocumentBuilders::with_config(config),
        }
    }

    pub fn convert(&self, input: FileInput, target_format: DocumentFormat) -> Result<FileResult> {
        match input {
            FileInput::Path(path) => self.convert_from_file(path, target_format),
            FileInput::Content { data, format } => {
                self.convert_from_content(data, format, target_format)
            }
            FileInput::Text(text) => self.convert_from_text(text, target_format),
        }
    }

    pub fn convert_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        target_format: DocumentFormat,
    ) -> Result<FileResult> {
        let path = path.as_ref();
        let source_format = self.detect_format_from_extension(path)?;
        let content = std::fs::read(path).map_err(|e| OfficeError::Io(e))?;

        self.convert_from_content(content, source_format, target_format)
    }

    pub fn convert_from_content(
        &self,
        content: Vec<u8>,
        source_format: DocumentFormat,
        target_format: DocumentFormat,
    ) -> Result<FileResult> {
        if source_format == target_format {
            let metadata = FileMetadata::default();
            let filename = format!("converted.{}", target_format.file_extension());
            return Ok(FileResult::new(content, filename, target_format, metadata));
        }

        let extracted_content =
            self.extract_content_from_format(&content, source_format.clone())?;
        let metadata = self.extract_metadata_from_format(&content, source_format)?;

        self.builders
            .docx
            .create_from_content(&extracted_content.text, metadata.clone())
            .and_then(|_| match target_format {
                DocumentFormat::Docx => self
                    .builders
                    .docx
                    .create_from_content(&extracted_content.text, metadata),
                DocumentFormat::Xlsx => self
                    .builders
                    .xlsx
                    .create_from_content(&extracted_content.text, metadata),
                DocumentFormat::Pptx => self
                    .builders
                    .pptx
                    .create_from_content(&extracted_content.text, metadata),
                DocumentFormat::Pdf => self
                    .builders
                    .pdf
                    .create_from_content(&extracted_content.text, metadata),
                DocumentFormat::Eml => self
                    .builders
                    .eml
                    .create_from_content(&extracted_content.text, metadata),
                DocumentFormat::Markdown => self
                    .builders
                    .markdown
                    .create_from_content(&extracted_content.text, metadata),
            })
    }

    pub fn convert_from_text(
        &self,
        text: String,
        target_format: DocumentFormat,
    ) -> Result<FileResult> {
        let metadata = FileMetadata::default();

        match target_format {
            DocumentFormat::Docx => self.builders.docx.create_from_content(&text, metadata),
            DocumentFormat::Xlsx => self.builders.xlsx.create_from_content(&text, metadata),
            DocumentFormat::Pptx => self.builders.pptx.create_from_content(&text, metadata),
            DocumentFormat::Pdf => self.builders.pdf.create_from_content(&text, metadata),
            DocumentFormat::Eml => self.builders.eml.create_from_content(&text, metadata),
            DocumentFormat::Markdown => self.builders.markdown.create_from_content(&text, metadata),
        }
    }

    pub fn batch_convert(
        &self,
        inputs: Vec<(FileInput, DocumentFormat)>,
    ) -> Result<Vec<FileResult>> {
        let mut results = Vec::new();

        for (input, target_format) in inputs {
            let result = self.convert(input, target_format)?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn get_supported_conversions() -> Vec<(DocumentFormat, Vec<DocumentFormat>)> {
        let all_formats = [
            DocumentFormat::Docx,
            DocumentFormat::Xlsx,
            DocumentFormat::Pptx,
            DocumentFormat::Pdf,
            DocumentFormat::Eml,
            DocumentFormat::Markdown,
        ];

        all_formats
            .to_vec()
            .iter()
            .map(|format| {
                let targets = all_formats
                    .iter()
                    .filter(|&target| target != format)
                    .cloned()
                    .collect();
                (format.clone(), targets)
            })
            .collect()
    }

    fn detect_format_from_extension<P: AsRef<Path>>(&self, path: P) -> Result<DocumentFormat> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| OfficeError::UnsupportedFormat("Unknown file extension".to_string()))?;

        match extension.as_str() {
            "docx" => Ok(DocumentFormat::Docx),
            "xlsx" => Ok(DocumentFormat::Xlsx),
            "pptx" => Ok(DocumentFormat::Pptx),
            "pdf" => Ok(DocumentFormat::Pdf),
            "eml" => Ok(DocumentFormat::Eml),
            "md" | "markdown" => Ok(DocumentFormat::Markdown),
            _ => Err(OfficeError::UnsupportedFormat(format!(
                "Unsupported file extension: {}",
                extension
            ))),
        }
    }

    fn extract_content_from_format(
        &self,
        content: &[u8],
        format: DocumentFormat,
    ) -> Result<ExtractedContent> {
        match format {
            DocumentFormat::Docx => {
                let doc = DocxDocument::from_bytes(content)?;
                let text = doc.get_text()?;
                Ok(ExtractedContent {
                    text,
                    images: doc.images.clone(),
                    tables: doc.tables.clone(),
                })
            }
            DocumentFormat::Xlsx => {
                let doc = XlsxDocument::from_bytes(content)?;
                // Extract text from all worksheets
                let mut text_parts = Vec::new();
                for worksheet_name in doc.worksheets.keys() {
                    if let Ok(data) = doc.get_worksheet_data(worksheet_name) {
                        text_parts.push(format!("Worksheet: {}", worksheet_name));
                        for row in data {
                            text_parts.push(row.join("\t"));
                        }
                        text_parts.push("".to_string()); // Empty line between worksheets
                    }
                }
                let text = text_parts.join("\n");
                Ok(ExtractedContent {
                    text,
                    images: std::collections::HashMap::new(),
                    tables: Vec::new(),
                })
            }
            DocumentFormat::Pptx => {
                let doc = PptxDocument::from_bytes(content)?;
                let text = doc.get_slide_text()?;
                Ok(ExtractedContent {
                    text,
                    images: std::collections::HashMap::new(),
                    tables: Vec::new(),
                })
            }
            DocumentFormat::Pdf => {
                let doc = PdfDocument::from_bytes(content)?;
                let text = doc.get_text()?;
                Ok(ExtractedContent {
                    text,
                    images: std::collections::HashMap::new(),
                    tables: Vec::new(),
                })
            }
            DocumentFormat::Eml => {
                let doc = EmlDocument::from_bytes(content)?;
                let text = doc.get_text()?;
                Ok(ExtractedContent {
                    text,
                    images: std::collections::HashMap::new(),
                    tables: Vec::new(),
                })
            }
            DocumentFormat::Markdown => {
                let text = String::from_utf8(content.to_vec()).map_err(|_| {
                    OfficeError::InvalidFormat("Invalid UTF-8 in markdown".to_string())
                })?;
                Ok(ExtractedContent {
                    text,
                    images: std::collections::HashMap::new(),
                    tables: Vec::new(),
                })
            }
        }
    }

    fn extract_metadata_from_format(
        &self,
        content: &[u8],
        format: DocumentFormat,
    ) -> Result<FileMetadata> {
        let mut metadata = FileMetadata::default();

        match format {
            DocumentFormat::Docx => {
                let doc = DocxDocument::from_bytes(content)?;
                metadata.title = doc.metadata.title.clone();
                metadata.author = doc.metadata.author.clone();
                metadata.subject = doc.metadata.subject.clone();
                metadata.description = doc.metadata.description.clone();
                metadata.created = doc.metadata.created;
                metadata.modified = doc.metadata.modified;
            }
            DocumentFormat::Xlsx => {
                let doc = XlsxDocument::from_bytes(content)?;
                metadata.title = Some(doc.title.clone());
                metadata.author = Some(doc.author.clone());
                metadata.created = doc.created;
                metadata.modified = doc.modified;
            }
            DocumentFormat::Pptx => {
                let doc = PptxDocument::from_bytes(content)?;
                metadata.title = Some(doc.title.clone());
                metadata.author = Some(doc.author.clone());
                metadata.created = doc.created;
                metadata.modified = doc.modified;
            }
            DocumentFormat::Pdf => {
                let doc = PdfDocument::from_bytes(content)?;
                metadata.title = Some(doc.title.clone());
                metadata.author = Some(doc.author.clone());
                metadata.created = doc.created;
                metadata.modified = doc.modified;
            }
            DocumentFormat::Eml => {
                let doc = EmlDocument::from_bytes(content)?;
                metadata.subject = Some(doc.headers.subject.clone());
                metadata.author = Some(doc.headers.from.email.clone());
                metadata.created = doc.headers.date;
                metadata.modified = doc.headers.date;
            }
            DocumentFormat::Markdown => {
                // Markdown documents typically don't have embedded metadata
            }
        }

        Ok(metadata)
    }
}

impl Default for DocumentConverters {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ExtractedContent {
    text: String,
    #[allow(dead_code)]
    images: std::collections::HashMap<String, crate::docx::ImageData>,
    #[allow(dead_code)]
    tables: Vec<crate::docx::Table>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let _converter = DocumentConverters::new();
        let conversions = DocumentConverters::get_supported_conversions();

        assert!(!conversions.is_empty());
        assert!(
            conversions
                .iter()
                .any(|(format, _)| matches!(format, DocumentFormat::Docx))
        );
    }

    #[test]
    fn test_format_detection() {
        let converter = DocumentConverters::new();

        assert!(matches!(
            converter.detect_format_from_extension("test.docx"),
            Ok(DocumentFormat::Docx)
        ));

        assert!(matches!(
            converter.detect_format_from_extension("test.pdf"),
            Ok(DocumentFormat::Pdf)
        ));
    }
}
