use crate::OfficeDocument;
use crate::error::Result;
use crate::files::builders::{BuilderConfig, DocumentBuilder};
use crate::files::{DocumentFormat, FileMetadata, FileResult};
use crate::markdown::MarkdownDocument;

#[derive(Debug, Clone)]
pub struct MarkdownBuilder {
    config: BuilderConfig,
}

impl MarkdownBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, config: &BuilderConfig) {
        self.config = config.clone();
    }

    pub fn from_text(&self, text_content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut content = String::new();

        if let Some(ref title) = metadata.title {
            content.push_str(&format!("# {}\n\n", title));
        }

        if let Some(ref author) = metadata.author {
            content.push_str(&format!("**Author:** {}\n", author));
        }

        if let Some(ref subject) = metadata.subject {
            content.push_str(&format!("**Subject:** {}\n", subject));
        }

        content.push_str(&format!(
            "**Date:** {}\n\n",
            metadata.created.format("%Y-%m-%d")
        ));

        content.push_str("---\n\n");

        for line in text_content.lines() {
            content.push_str(line);
            content.push('\n');
        }

        let doc = MarkdownDocument::from_markdown(&content)?;
        let final_content = doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "md");

        Ok(FileResult::new(
            final_content,
            filename,
            DocumentFormat::Markdown,
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
            "documentation" => self.create_documentation(content, metadata),
            "readme" => self.create_readme(content, metadata),
            "blog" => self.create_blog_post(content, metadata),
            "notes" => self.create_notes(content, metadata),
            _ => self.from_text(content, metadata),
        }
    }

    fn create_documentation(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut md_content = String::new();

        let title = metadata
            .title
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Documentation");
        md_content.push_str(&format!("# {}\n\n", title));

        md_content.push_str("## Table of Contents\n\n");
        md_content.push_str("- [Overview](#overview)\n");
        md_content.push_str("- [Installation](#installation)\n");
        md_content.push_str("- [Usage](#usage)\n");
        md_content.push_str("- [API Reference](#api-reference)\n\n");

        md_content.push_str("## Overview\n\n");
        md_content.push_str(content);
        md_content.push_str("\n\n");

        if let Some(ref author) = metadata.author {
            md_content.push_str(&format!("---\n\n**Maintained by:** {}\n", author));
        }

        md_content.push_str(&format!(
            "**Last updated:** {}\n",
            metadata.modified.format("%Y-%m-%d")
        ));

        let doc = MarkdownDocument::from_markdown(&md_content)?;
        let final_content = doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "md");

        Ok(FileResult::new(
            final_content,
            filename,
            DocumentFormat::Markdown,
            metadata,
        ))
    }

    fn create_readme(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut md_content = String::new();

        let title = metadata
            .title
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Project");
        md_content.push_str(&format!("# {}\n\n", title));

        if let Some(ref description) = metadata.description {
            md_content.push_str(&format!("{}\n\n", description));
        }

        md_content.push_str("## Installation\n\n");
        md_content.push_str("```bash\n# Installation instructions here\n```\n\n");

        md_content.push_str("## Usage\n\n");
        md_content.push_str(content);
        md_content.push_str("\n\n");

        md_content.push_str("## Contributing\n\n");
        md_content
            .push_str("Contributions are welcome! Please read our contributing guidelines.\n\n");

        md_content.push_str("## License\n\n");
        md_content.push_str("This project is licensed under the MIT License.\n");

        let doc = MarkdownDocument::from_markdown(&md_content)?;
        let final_content = doc.to_bytes()?;
        let filename = "README.md".to_string();

        Ok(FileResult::new(
            final_content,
            filename,
            DocumentFormat::Markdown,
            metadata,
        ))
    }

    fn create_blog_post(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut md_content = String::new();

        md_content.push_str("---\n");
        if let Some(ref title) = metadata.title {
            md_content.push_str(&format!("title: \"{}\"\n", title));
        }
        if let Some(ref author) = metadata.author {
            md_content.push_str(&format!("author: {}\n", author));
        }
        md_content.push_str(&format!("date: {}\n", metadata.created.format("%Y-%m-%d")));
        if !metadata.keywords.is_empty() {
            md_content.push_str(&format!("tags: [{}]\n", metadata.keywords.join(", ")));
        }
        md_content.push_str("---\n\n");

        if let Some(ref title) = metadata.title {
            md_content.push_str(&format!("# {}\n\n", title));
        }

        md_content.push_str(&format!(
            "*Published on {}*\n\n",
            metadata.created.format("%B %d, %Y")
        ));

        md_content.push_str(content);
        md_content.push_str("\n\n---\n\n");

        if let Some(ref author) = metadata.author {
            md_content.push_str(&format!("*Written by {}*\n", author));
        }

        let doc = MarkdownDocument::from_markdown(&md_content)?;
        let final_content = doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "md");

        Ok(FileResult::new(
            final_content,
            filename,
            DocumentFormat::Markdown,
            metadata,
        ))
    }

    fn create_notes(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        let mut md_content = String::new();

        if let Some(ref title) = metadata.title {
            md_content.push_str(&format!("# {}\n\n", title));
        } else {
            md_content.push_str("# Notes\n\n");
        }

        md_content.push_str(&format!(
            "**Date:** {}\n\n",
            metadata.created.format("%Y-%m-%d %H:%M")
        ));

        if !metadata.keywords.is_empty() {
            md_content.push_str(&format!("**Tags:** {}\n\n", metadata.keywords.join(", ")));
        }

        md_content.push_str("---\n\n");
        md_content.push_str(content);

        let doc = MarkdownDocument::from_markdown(&md_content)?;
        let final_content = doc.to_bytes()?;
        let filename = self.generate_filename(&metadata, "md");

        Ok(FileResult::new(
            final_content,
            filename,
            DocumentFormat::Markdown,
            metadata,
        ))
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

impl DocumentBuilder for MarkdownBuilder {
    fn create_from_content(&self, content: &str, metadata: FileMetadata) -> Result<FileResult> {
        self.from_text(content, metadata)
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
    fn test_markdown_builder_creation() {
        let config = BuilderConfig::default();
        let builder = MarkdownBuilder::new(&config);

        let supported = builder.supported_input_formats();
        assert!(supported.contains(&"text"));
        assert!(supported.contains(&"markdown"));
    }

    #[test]
    fn test_create_notes() {
        let config = BuilderConfig::default();
        let builder = MarkdownBuilder::new(&config);
        let metadata = FileMetadata {
            title: Some("Meeting Notes".to_string()),
            ..Default::default()
        };

        let result = builder.create_from_template("notes", "Discussion points", metadata);
        assert!(result.is_ok());
    }
}
