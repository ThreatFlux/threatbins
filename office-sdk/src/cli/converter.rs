use crate::cli::config::Config;
use crate::cli::utils::*;
use crate::OfficeDocument;
use crate::docx::DocxDocument;
use crate::xlsx::XlsxDocument;
use crate::pptx::PptxDocument;
use crate::pdf::PdfDocument;
use crate::markdown::MarkdownDocument;
use std::collections::HashMap;
use std::path::Path;

pub struct DocumentConverter {
    config: Config,
    template: Option<String>,
    metadata: DocumentMetadata,
    options: HashMap<String, ConversionOption>,
}

#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConversionOption {
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<String>),
}

impl DocumentConverter {
    pub fn new(config: &Config, template: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(DocumentConverter {
            config: config.clone(),
            template: template.map(|s| s.to_string()),
            metadata: DocumentMetadata::default(),
            options: HashMap::new(),
        })
    }

    pub fn set_title(&mut self, title: &str) {
        self.metadata.title = Some(title.to_string());
    }

    pub fn set_author(&mut self, author: &str) {
        self.metadata.author = Some(author.to_string());
    }

    pub fn set_subject(&mut self, subject: &str) {
        self.metadata.subject = Some(subject.to_string());
    }

    pub fn set_option<T: Into<ConversionOption>>(&mut self, key: &str, value: T) {
        self.options.insert(key.to_string(), value.into());
    }

    pub fn convert_markdown(&mut self, markdown: &str, output_format: &super::OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Parse markdown
        let md_doc = MarkdownDocument::from_markdown(markdown)?;

        match output_format {
            super::OutputFormat::Docx => self.markdown_to_docx(&md_doc),
            super::OutputFormat::Xlsx => self.markdown_to_xlsx(&md_doc),
            super::OutputFormat::Pptx => self.markdown_to_pptx(&md_doc),
            super::OutputFormat::Pdf => self.markdown_to_pdf(&md_doc),
            super::OutputFormat::Html => self.markdown_to_html(&md_doc),
            super::OutputFormat::Text => Ok(md_doc.to_markdown().into_bytes()),
            super::OutputFormat::Json => self.to_json(&md_doc),
            super::OutputFormat::Markdown => Ok(markdown.as_bytes().to_vec()),
        }
    }

    pub fn convert_text(&mut self, text: &str, output_format: &super::OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Convert plain text to markdown first, then convert
        let markdown = self.text_to_markdown(text);
        self.convert_markdown(&markdown, output_format)
    }

    pub fn convert_from_docx(&mut self, doc: &DocxDocument, output_format: &super::OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match output_format {
            super::OutputFormat::Docx => doc.to_bytes().map_err(|e| e.into()),
            super::OutputFormat::Pdf => self.docx_to_pdf(doc),
            super::OutputFormat::Markdown => self.docx_to_markdown(doc),
            super::OutputFormat::Text => Ok(doc.get_text().unwrap_or_default().into_bytes()),
            super::OutputFormat::Html => self.docx_to_html(doc),
            super::OutputFormat::Json => self.docx_to_json(doc),
            _ => Err("Unsupported conversion from DOCX".into()),
        }
    }

    pub fn convert_from_xlsx(&mut self, doc: &XlsxDocument, output_format: &super::OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match output_format {
            super::OutputFormat::Xlsx => doc.to_bytes().map_err(|e| e.into()),
            super::OutputFormat::Pdf => self.xlsx_to_pdf(doc),
            super::OutputFormat::Text => Ok(self.xlsx_to_text(doc).into_bytes()),
            super::OutputFormat::Json => self.xlsx_to_json(doc),
            super::OutputFormat::Html => self.xlsx_to_html(doc),
            _ => Err("Unsupported conversion from XLSX".into()),
        }
    }

    pub fn convert_from_pptx(&mut self, doc: &PptxDocument, output_format: &super::OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match output_format {
            super::OutputFormat::Pptx => doc.to_bytes().map_err(|e| e.into()),
            super::OutputFormat::Pdf => self.pptx_to_pdf(doc),
            super::OutputFormat::Text => Ok(doc.get_slide_text().unwrap_or_default().into_bytes()),
            super::OutputFormat::Json => self.pptx_to_json(doc),
            super::OutputFormat::Html => self.pptx_to_html(doc),
            _ => Err("Unsupported conversion from PPTX".into()),
        }
    }

    pub fn convert_from_pdf(&mut self, doc: &PdfDocument, output_format: &super::OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match output_format {
            super::OutputFormat::Pdf => doc.to_bytes().map_err(|e| e.into()),
            super::OutputFormat::Text => Ok(doc.get_text().unwrap_or_default().into_bytes()),
            super::OutputFormat::Markdown => self.pdf_to_markdown(doc),
            super::OutputFormat::Json => self.pdf_to_json(doc),
            super::OutputFormat::Html => self.pdf_to_html(doc),
            _ => Err("Unsupported conversion from PDF".into()),
        }
    }

    // Conversion implementations
    fn markdown_to_docx(&self, md_doc: &MarkdownDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut docx = md_doc.to_docx()?;

        // Apply metadata
        if let Some(ref title) = self.metadata.title {
            // docx.set_title(title);  // These methods need to be implemented in the DOCX module
        }
        if let Some(ref author) = self.metadata.author {
            // docx.set_author(author);
        }

        // Apply template and options
        self.apply_docx_template(&mut docx)?;
        self.apply_docx_options(&mut docx)?;

        docx.to_bytes().map_err(|e| e.into())
    }

    fn markdown_to_xlsx(&self, md_doc: &MarkdownDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut xlsx = XlsxDocument::new();

        // Apply metadata
        if let Some(ref title) = self.metadata.title {
            // xlsx.set_title(title);
        }

        // Convert markdown to spreadsheet data
        self.apply_markdown_to_xlsx(&md_doc, &mut xlsx)?;
        self.apply_xlsx_options(&mut xlsx)?;

        xlsx.to_bytes().map_err(|e| e.into())
    }

    fn markdown_to_pptx(&self, md_doc: &MarkdownDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut pptx = PptxDocument::new();

        // Apply metadata
        if let Some(ref title) = self.metadata.title {
            pptx.set_title(title);
        }
        if let Some(ref author) = self.metadata.author {
            pptx.set_author(author);
        }

        // Convert markdown to slides
        self.apply_markdown_to_pptx(&md_doc, &mut pptx)?;
        self.apply_pptx_options(&mut pptx)?;

        pptx.to_bytes().map_err(|e| e.into())
    }

    fn markdown_to_pdf(&self, md_doc: &MarkdownDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut pdf = PdfDocument::new();

        // Apply metadata
        if let Some(ref title) = self.metadata.title {
            pdf.set_title(title);
        }
        if let Some(ref author) = self.metadata.author {
            pdf.set_author(author);
        }
        if let Some(ref subject) = self.metadata.subject {
            pdf.set_subject(subject);
        }

        // Convert markdown to PDF
        self.apply_markdown_to_pdf(&md_doc, &mut pdf)?;
        self.apply_pdf_options(&mut pdf)?;

        pdf.to_bytes().map_err(|e| e.into())
    }

    fn markdown_to_html(&self, md_doc: &MarkdownDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let markdown_text = md_doc.to_markdown();
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1, h2, h3 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        code {{ background-color: #f5f5f5; padding: 2px 4px; border-radius: 3px; }}
        pre {{ background-color: #f5f5f5; padding: 10px; border-radius: 5px; overflow-x: auto; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            self.metadata.title.as_deref().unwrap_or("Document"),
            self.markdown_to_html_body(&markdown_text)
        );

        Ok(html.into_bytes())
    }

    fn text_to_markdown(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut markdown = String::new();

        // Add title if provided
        if let Some(ref title) = self.metadata.title {
            markdown.push_str(&format!("# {}\n\n", title));
        }

        // Convert text to markdown with basic formatting
        let mut in_code_block = false;
        let mut current_paragraph = String::new();

        for line in lines {
            let trimmed = line.trim();

            // Detect code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("    ") {
                if !current_paragraph.is_empty() {
                    markdown.push_str(&current_paragraph);
                    markdown.push_str("\n\n");
                    current_paragraph.clear();
                }

                if trimmed.starts_with("```") {
                    in_code_block = !in_code_block;
                }
                markdown.push_str(line);
                markdown.push('\n');
                continue;
            }

            if in_code_block {
                markdown.push_str(line);
                markdown.push('\n');
                continue;
            }

            // Detect headings (lines that are short and end with colon or are all caps)
            if trimmed.len() < 80 && (trimmed.ends_with(':') || trimmed == trimmed.to_uppercase()) && trimmed.len() > 3 {
                if !current_paragraph.is_empty() {
                    markdown.push_str(&current_paragraph);
                    markdown.push_str("\n\n");
                    current_paragraph.clear();
                }

                let heading_text = trimmed.trim_end_matches(':');
                markdown.push_str(&format!("## {}\n\n", heading_text));
                continue;
            }

            // Detect list items
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") ||
               (trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) && trimmed.contains(". ")) {
                if !current_paragraph.is_empty() {
                    markdown.push_str(&current_paragraph);
                    markdown.push_str("\n\n");
                    current_paragraph.clear();
                }
                markdown.push_str(line);
                markdown.push('\n');
                continue;
            }

            // Regular paragraph text
            if trimmed.is_empty() {
                if !current_paragraph.is_empty() {
                    markdown.push_str(&current_paragraph);
                    markdown.push_str("\n\n");
                    current_paragraph.clear();
                }
            } else {
                if !current_paragraph.is_empty() {
                    current_paragraph.push(' ');
                }
                current_paragraph.push_str(trimmed);
            }
        }

        // Add final paragraph
        if !current_paragraph.is_empty() {
            markdown.push_str(&current_paragraph);
            markdown.push_str("\n");
        }

        markdown
    }

    fn apply_docx_template(&self, _docx: &mut DocxDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Template application logic would go here
        Ok(())
    }

    fn apply_docx_options(&self, _docx: &mut DocxDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Apply various options like TOC, page numbers, etc.
        Ok(())
    }

    fn apply_markdown_to_xlsx(&self, _md_doc: &MarkdownDocument, xlsx: &mut XlsxDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Convert markdown tables to spreadsheet data
        // This is a simplified implementation - can be enhanced
        let data = vec![
            vec!["Content".to_string(), "Type".to_string(), "Notes".to_string()],
            vec!["Markdown Document".to_string(), "Text".to_string(), "Converted from MD".to_string()],
        ];

        xlsx.add_worksheet("Content", &data)?;
        Ok(())
    }

    fn apply_xlsx_options(&self, _xlsx: &mut XlsxDocument) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn apply_markdown_to_pptx(&self, md_doc: &MarkdownDocument, pptx: &mut PptxDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Convert markdown sections to slides
        let markdown_text = md_doc.to_markdown();
        let sections = self.split_markdown_into_sections(&markdown_text);

        for (i, section) in sections.iter().enumerate() {
            let title = section.title.as_deref().unwrap_or(&format!("Slide {}", i + 1));
            let content: Vec<&str> = section.content.iter().map(|s| s.as_str()).collect();

            pptx.add_slide(title, &content)?;
        }

        Ok(())
    }

    fn apply_pptx_options(&self, _pptx: &mut PptxDocument) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn apply_markdown_to_pdf(&self, md_doc: &MarkdownDocument, pdf: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Add pages and content
        let _page_id = pdf.add_a4_page()?;

        let markdown_text = md_doc.to_markdown();
        let mut y_position = 750.0;
        let line_height = 14.0;

        // Parse and add content section by section
        for line in markdown_text.lines() {
            if y_position < 100.0 {
                // Add new page if we're running out of space
                let _new_page = pdf.add_a4_page()?;
                y_position = 750.0;
            }

            if line.starts_with("# ") {
                let heading_text = line.trim_start_matches("# ");
                pdf.add_heading(pdf.pages.len() - 1, 1, heading_text, 100.0, y_position)?;
                y_position -= line_height * 2.0;
            } else if line.starts_with("## ") {
                let heading_text = line.trim_start_matches("## ");
                pdf.add_heading(pdf.pages.len() - 1, 2, heading_text, 100.0, y_position)?;
                y_position -= line_height * 1.8;
            } else if line.starts_with("### ") {
                let heading_text = line.trim_start_matches("### ");
                pdf.add_heading(pdf.pages.len() - 1, 3, heading_text, 100.0, y_position)?;
                y_position -= line_height * 1.6;
            } else if !line.trim().is_empty() {
                pdf.add_text(pdf.pages.len() - 1, line, 100.0, y_position, "Helvetica", 11.0)?;
                y_position -= line_height;
            } else {
                y_position -= line_height * 0.5; // Smaller gap for empty lines
            }
        }

        Ok(())
    }

    fn apply_pdf_options(&self, pdf: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Apply password protection if specified
        if let Some(ConversionOption::String(password)) = self.options.get("password") {
            pdf.set_password_protection(Some(password), None);
        }

        Ok(())
    }

    fn docx_to_pdf(&self, doc: &DocxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut pdf = PdfDocument::new();

        // Extract text from DOCX and add to PDF
        if let Ok(text) = doc.get_text() {
            let _page_id = pdf.add_a4_page()?;
            pdf.add_paragraph(0, &text, 100.0, 750.0, 400.0, 11.0)?;
        }

        pdf.to_bytes().map_err(|e| e.into())
    }

    fn docx_to_markdown(&self, doc: &DocxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Ok(text) = doc.get_text() {
            let markdown = self.text_to_markdown(&text);
            Ok(markdown.into_bytes())
        } else {
            Ok(b"# Document\n\nContent could not be extracted.".to_vec())
        }
    }

    fn docx_to_html(&self, doc: &DocxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Ok(text) = doc.get_text() {
            let html = format!(
                r#"<!DOCTYPE html>
<html><head><title>{}</title></head>
<body><pre>{}</pre></body></html>"#,
                self.metadata.title.as_deref().unwrap_or("Document"),
                html_escape(&text)
            );
            Ok(html.into_bytes())
        } else {
            Err("Could not extract text from DOCX".into())
        }
    }

    fn docx_to_json(&self, doc: &DocxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let stats = doc.get_document_stats();
        let json_data = serde_json::json!({
            "type": "docx",
            "title": self.metadata.title,
            "author": self.metadata.author,
            "stats": {
                "page_count": stats.page_count,
                "image_count": stats.image_count,
                "table_count": stats.table_count,
                "has_vba": stats.has_vba,
                "created": stats.created.to_rfc3339(),
                "modified": stats.modified.to_rfc3339()
            },
            "text_content": doc.get_text().unwrap_or_default()
        });

        Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
    }

    fn xlsx_to_pdf(&self, doc: &XlsxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut pdf = PdfDocument::new();
        let _page_id = pdf.add_a4_page()?;

        // Extract and add spreadsheet data as table
        if let Ok(data) = doc.get_worksheet_data("sheet1") {
            if !data.is_empty() {
                let headers: Vec<&str> = data[0].iter().map(|s| s.as_str()).collect();
                let rows: Vec<Vec<&str>> = data.iter().skip(1)
                    .map(|row| row.iter().map(|s| s.as_str()).collect())
                    .collect();

                pdf.add_table(0, 100.0, 700.0, &headers, &rows)?;
            }
        }

        pdf.to_bytes().map_err(|e| e.into())
    }

    fn xlsx_to_text(&self, doc: &XlsxDocument) -> String {
        // Extract all worksheet data as text
        let mut text = String::new();

        for (sheet_name, _) in &doc.worksheets {
            text.push_str(&format!("Sheet: {}\n", sheet_name));

            if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                for row in data {
                    for cell in row {
                        text.push_str(&cell);
                        text.push('\t');
                    }
                    text.push('\n');
                }
            }
            text.push('\n');
        }

        text
    }

    fn xlsx_to_json(&self, doc: &XlsxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut worksheets = HashMap::new();

        for (sheet_name, _) in &doc.worksheets {
            if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                worksheets.insert(sheet_name.clone(), data);
            }
        }

        let json_data = serde_json::json!({
            "type": "xlsx",
            "title": self.metadata.title,
            "author": self.metadata.author,
            "worksheets": worksheets
        });

        Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
    }

    fn xlsx_to_html(&self, doc: &XlsxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut html = String::from(r#"<!DOCTYPE html>
<html><head><title>Spreadsheet</title>
<style>table{border-collapse:collapse;width:100%;}th,td{border:1px solid #ddd;padding:8px;}th{background-color:#f2f2f2;}</style>
</head><body>"#);

        for (sheet_name, _) in &doc.worksheets {
            html.push_str(&format!("<h2>{}</h2>", sheet_name));

            if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                html.push_str("<table>");

                for (i, row) in data.iter().enumerate() {
                    html.push_str("<tr>");
                    for cell in row {
                        let tag = if i == 0 { "th" } else { "td" };
                        html.push_str(&format!("<{}>{}</{}>", tag, html_escape(cell), tag));
                    }
                    html.push_str("</tr>");
                }

                html.push_str("</table>");
            }
        }

        html.push_str("</body></html>");
        Ok(html.into_bytes())
    }

    fn pptx_to_pdf(&self, doc: &PptxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut pdf = PdfDocument::new();

        if let Ok(slides) = doc.get_all_slides() {
            for (i, slide) in slides.iter().enumerate() {
                let _page_id = pdf.add_a4_page()?;

                // Add slide title
                if !slide.title.is_empty() {
                    pdf.add_heading(i, 1, &slide.title, 100.0, 750.0)?;
                }

                // Add slide content
                let mut y_pos = 700.0;
                for content in &slide.content {
                    pdf.add_text(i, content, 120.0, y_pos, "Helvetica", 12.0)?;
                    y_pos -= 20.0;
                }
            }
        }

        pdf.to_bytes().map_err(|e| e.into())
    }

    fn pptx_to_json(&self, doc: &PptxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let slides = doc.get_all_slides().unwrap_or_default();
        let slides_data: Vec<_> = slides.iter().map(|slide| {
            serde_json::json!({
                "id": slide.id,
                "title": slide.title,
                "content": slide.content
            })
        }).collect();

        let json_data = serde_json::json!({
            "type": "pptx",
            "title": self.metadata.title,
            "author": self.metadata.author,
            "slides": slides_data
        });

        Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
    }

    fn pptx_to_html(&self, doc: &PptxDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut html = String::from(r#"<!DOCTYPE html>
<html><head><title>Presentation</title>
<style>
.slide{page-break-after:always;margin:20px;padding:20px;border:1px solid #ccc;}
.slide h1{color:#333;border-bottom:2px solid #333;}
.slide ul{list-style-type:disc;margin-left:20px;}
</style></head><body>"#);

        if let Ok(slides) = doc.get_all_slides() {
            for slide in slides {
                html.push_str("<div class='slide'>");
                html.push_str(&format!("<h1>{}</h1>", html_escape(&slide.title)));

                if !slide.content.is_empty() {
                    html.push_str("<ul>");
                    for content in &slide.content {
                        html.push_str(&format!("<li>{}</li>", html_escape(content)));
                    }
                    html.push_str("</ul>");
                }

                html.push_str("</div>");
            }
        }

        html.push_str("</body></html>");
        Ok(html.into_bytes())
    }

    fn pdf_to_markdown(&self, doc: &PdfDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Ok(text) = doc.get_text() {
            let markdown = self.text_to_markdown(&text);
            Ok(markdown.into_bytes())
        } else {
            Ok(b"# PDF Document\n\nContent could not be extracted.".to_vec())
        }
    }

    fn pdf_to_json(&self, doc: &PdfDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let stats = doc.get_document_stats();
        let json_data = serde_json::json!({
            "type": "pdf",
            "title": doc.title,
            "author": doc.author,
            "stats": {
                "page_count": stats.page_count,
                "image_count": stats.image_count,
                "annotation_count": stats.annotation_count,
                "form_field_count": stats.form_field_count,
                "has_security": stats.has_security,
                "has_javascript": stats.has_javascript,
                "pdf_version": format!("{:?}", stats.pdf_version),
                "file_size": stats.file_size
            },
            "text_content": doc.get_text().unwrap_or_default()
        });

        Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
    }

    fn pdf_to_html(&self, doc: &PdfDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Ok(text) = doc.get_text() {
            let html = format!(
                r#"<!DOCTYPE html>
<html><head><title>{}</title></head>
<body><pre>{}</pre></body></html>"#,
                doc.title,
                html_escape(&text)
            );
            Ok(html.into_bytes())
        } else {
            Err("Could not extract text from PDF".into())
        }
    }

    fn to_json(&self, md_doc: &MarkdownDocument) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json_data = serde_json::json!({
            "type": "markdown",
            "title": self.metadata.title,
            "author": self.metadata.author,
            "elements": md_doc.elements.len(),
            "content": md_doc.to_markdown()
        });

        Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
    }

    fn split_markdown_into_sections(&self, markdown: &str) -> Vec<MarkdownSection> {
        let mut sections = Vec::new();
        let mut current_section = MarkdownSection {
            title: None,
            content: Vec::new(),
        };

        for line in markdown.lines() {
            if line.starts_with("# ") {
                // New section
                if current_section.title.is_some() || !current_section.content.is_empty() {
                    sections.push(current_section);
                }
                current_section = MarkdownSection {
                    title: Some(line.trim_start_matches("# ").to_string()),
                    content: Vec::new(),
                };
            } else if line.starts_with("## ") {
                // Subsection - add as content
                current_section.content.push(line.trim_start_matches("## ").to_string());
            } else if !line.trim().is_empty() {
                current_section.content.push(line.to_string());
            }
        }

        // Add final section
        if current_section.title.is_some() || !current_section.content.is_empty() {
            sections.push(current_section);
        }

        sections
    }

    fn markdown_to_html_body(&self, markdown: &str) -> String {
        // Simple markdown to HTML conversion
        let mut html = String::new();
        let mut in_code_block = false;

        for line in markdown.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    html.push_str("</pre>\n");
                } else {
                    html.push_str("<pre><code>");
                }
                in_code_block = !in_code_block;
            } else if in_code_block {
                html.push_str(&html_escape(line));
                html.push('\n');
            } else if line.starts_with("# ") {
                html.push_str(&format!("<h1>{}</h1>\n", html_escape(line.trim_start_matches("# "))));
            } else if line.starts_with("## ") {
                html.push_str(&format!("<h2>{}</h2>\n", html_escape(line.trim_start_matches("## "))));
            } else if line.starts_with("### ") {
                html.push_str(&format!("<h3>{}</h3>\n", html_escape(line.trim_start_matches("### "))));
            } else if line.starts_with("- ") || line.starts_with("* ") {
                html.push_str(&format!("<li>{}</li>\n", html_escape(line.trim_start_matches(if line.starts_with("- ") { "- " } else { "* " }))));
            } else if !line.trim().is_empty() {
                html.push_str(&format!("<p>{}</p>\n", html_escape(line)));
            }
        }

        html
    }
}

#[derive(Debug, Clone)]
struct MarkdownSection {
    title: Option<String>,
    content: Vec<String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            keywords: None,
            description: None,
            category: None,
        }
    }
}

impl From<bool> for ConversionOption {
    fn from(value: bool) -> Self {
        ConversionOption::Boolean(value)
    }
}

impl From<String> for ConversionOption {
    fn from(value: String) -> Self {
        ConversionOption::String(value)
    }
}

impl From<&str> for ConversionOption {
    fn from(value: &str) -> Self {
        ConversionOption::String(value.to_string())
    }
}

impl From<f64> for ConversionOption {
    fn from(value: f64) -> Self {
        ConversionOption::Number(value)
    }
}

impl From<Vec<String>> for ConversionOption {
    fn from(value: Vec<String>) -> Self {
        ConversionOption::Array(value)
    }
}