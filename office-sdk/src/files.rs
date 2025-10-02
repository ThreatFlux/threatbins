use crate::error::{OfficeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod builders;
pub mod converters;
pub mod storage;

pub use builders::*;
pub use converters::*;
pub use storage::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentFormat {
    Docx,
    Xlsx,
    Pptx,
    Pdf,
    Eml,
    Markdown,
}

impl DocumentFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            DocumentFormat::Docx => "docx",
            DocumentFormat::Xlsx => "xlsx",
            DocumentFormat::Pptx => "pptx",
            DocumentFormat::Pdf => "pdf",
            DocumentFormat::Eml => "eml",
            DocumentFormat::Markdown => "md",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            DocumentFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            DocumentFormat::Xlsx => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            DocumentFormat::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            DocumentFormat::Pdf => "application/pdf",
            DocumentFormat::Eml => "message/rfc822",
            DocumentFormat::Markdown => "text/markdown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
    pub metadata: FileMetadata,
    pub format: DocumentFormat,
}

impl FileResult {
    pub fn new(
        content: Vec<u8>,
        filename: String,
        format: DocumentFormat,
        metadata: FileMetadata,
    ) -> Self {
        Self {
            content,
            filename,
            mime_type: format.mime_type().to_string(),
            metadata,
            format,
        }
    }

    pub fn size(&self) -> usize {
        self.content.len()
    }

    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        std::fs::write(path, &self.content).map_err(|e| OfficeError::Io(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_modified_by: Option<String>,
    pub properties: HashMap<String, String>,
}

impl Default for FileMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            title: None,
            author: None,
            subject: None,
            description: None,
            keywords: Vec::new(),
            created: now,
            modified: now,
            last_modified_by: None,
            properties: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequest {
    pub format: DocumentFormat,
    pub content: String,
    pub filename: Option<String>,
    pub metadata: Option<FileMetadata>,
    pub template: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FileInput {
    Path(std::path::PathBuf),
    Content {
        data: Vec<u8>,
        format: DocumentFormat,
    },
    Text(String),
}

pub struct FilesAPI {
    pub builders: DocumentBuilders,
    pub converters: DocumentConverters,
    pub storage: FileStorage,
}

impl FilesAPI {
    pub fn new() -> Self {
        Self {
            builders: DocumentBuilders::new(),
            converters: DocumentConverters::new(),
            storage: FileStorage::new(),
        }
    }

    pub fn create_document(&self, format: DocumentFormat, content: &str) -> Result<FileResult> {
        self.create_document_with_metadata(format, content, FileMetadata::default())
    }

    pub fn create_document_with_metadata(
        &self,
        format: DocumentFormat,
        content: &str,
        metadata: FileMetadata,
    ) -> Result<FileResult> {
        match format {
            DocumentFormat::Docx => self.builders.docx.create_from_content(content, metadata),
            DocumentFormat::Xlsx => self.builders.xlsx.create_from_content(content, metadata),
            DocumentFormat::Pptx => self.builders.pptx.create_from_content(content, metadata),
            DocumentFormat::Pdf => self.builders.pdf.create_from_content(content, metadata),
            DocumentFormat::Eml => self.builders.eml.create_from_content(content, metadata),
            DocumentFormat::Markdown => self
                .builders
                .markdown
                .create_from_content(content, metadata),
        }
    }

    pub fn convert_document(
        &self,
        input: FileInput,
        target_format: DocumentFormat,
    ) -> Result<FileResult> {
        self.converters.convert(input, target_format)
    }

    pub fn batch_create(&self, requests: Vec<CreateRequest>) -> Result<Vec<FileResult>> {
        let mut results = Vec::new();

        for request in requests {
            let metadata = request.metadata.unwrap_or_default();
            let result =
                self.create_document_with_metadata(request.format, &request.content, metadata)?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn get_supported_formats() -> Vec<DocumentFormat> {
        vec![
            DocumentFormat::Docx,
            DocumentFormat::Xlsx,
            DocumentFormat::Pptx,
            DocumentFormat::Pdf,
            DocumentFormat::Eml,
            DocumentFormat::Markdown,
        ]
    }

    pub fn validate_format_support(format: &DocumentFormat) -> bool {
        Self::get_supported_formats().contains(format)
    }
}

impl Default for FilesAPI {
    fn default() -> Self {
        Self::new()
    }
}
