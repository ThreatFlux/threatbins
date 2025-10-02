use crate::cli::config::Config;
use crate::OfficeDocument;
use crate::docx::DocxDocument;
use crate::xlsx::XlsxDocument;
use crate::pptx::PptxDocument;
use crate::pdf::PdfDocument;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ContentExtractor;

impl ContentExtractor {
    pub fn new() -> Self {
        ContentExtractor
    }

    pub fn extract(
        &self,
        input: &Path,
        extract_type: &super::ExtractType,
        pages: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let extension = input.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "docx" => self.extract_from_docx(input, extract_type, pages),
            "xlsx" => self.extract_from_xlsx(input, extract_type, pages),
            "pptx" => self.extract_from_pptx(input, extract_type, pages),
            "pdf" => self.extract_from_pdf(input, extract_type, pages),
            _ => Err(format!("Unsupported file format: {}", extension).into()),
        }
    }

    fn extract_from_docx(
        &self,
        input: &Path,
        extract_type: &super::ExtractType,
        _pages: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let doc = DocxDocument::from_file(input)?;

        match extract_type {
            super::ExtractType::Text => {
                Ok(doc.get_text().unwrap_or_default())
            },
            super::ExtractType::Metadata => {
                let stats = doc.get_document_stats();
                let metadata = serde_json::json!({
                    "type": "docx",
                    "page_count": stats.page_count,
                    "image_count": stats.image_count,
                    "table_count": stats.table_count,
                    "has_vba": stats.has_vba,
                    "created": stats.created.to_rfc3339(),
                    "modified": stats.modified.to_rfc3339(),
                    "author": stats.author
                });
                Ok(serde_json::to_string_pretty(&metadata)?)
            },
            super::ExtractType::Images => {
                Ok(format!("Found {} images in document", doc.images.len()))
            },
            super::ExtractType::Tables => {
                Ok(format!("Found {} tables in document", doc.tables.len()))
            },
            super::ExtractType::All => {
                let mut result = String::new();
                result.push_str("=== DOCUMENT ANALYSIS ===\n\n");
                result.push_str("Text Content:\n");
                result.push_str(&doc.get_text().unwrap_or_default());
                result.push_str("\n\nImages: ");
                result.push_str(&doc.images.len().to_string());
                result.push_str("\nTables: ");
                result.push_str(&doc.tables.len().to_string());
                Ok(result)
            },
            _ => Ok("Extraction type not supported for DOCX".to_string()),
        }
    }

    fn extract_from_xlsx(
        &self,
        input: &Path,
        extract_type: &super::ExtractType,
        _pages: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let doc = XlsxDocument::from_file(input)?;

        match extract_type {
            super::ExtractType::Text => {
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
                Ok(text)
            },
            super::ExtractType::Metadata => {
                let metadata = serde_json::json!({
                    "type": "xlsx",
                    "worksheet_count": doc.worksheets.len(),
                    "author": doc.author,
                    "title": doc.title,
                    "created": doc.created.to_rfc3339(),
                    "modified": doc.modified.to_rfc3339()
                });
                Ok(serde_json::to_string_pretty(&metadata)?)
            },
            super::ExtractType::Tables => {
                let mut tables_info = String::new();
                for (sheet_name, _) in &doc.worksheets {
                    if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                        tables_info.push_str(&format!("Sheet '{}': {} rows, {} columns\n",
                            sheet_name, data.len(),
                            data.first().map(|row| row.len()).unwrap_or(0)));
                    }
                }
                Ok(tables_info)
            },
            super::ExtractType::All => {
                let mut result = String::new();
                result.push_str("=== SPREADSHEET ANALYSIS ===\n\n");
                result.push_str(&format!("Worksheets: {}\n", doc.worksheets.len()));

                for (sheet_name, _) in &doc.worksheets {
                    result.push_str(&format!("\nSheet: {}\n", sheet_name));
                    if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                        result.push_str(&format!("Rows: {}, Columns: {}\n",
                            data.len(),
                            data.first().map(|row| row.len()).unwrap_or(0)));

                        // Show first few rows as sample
                        for (i, row) in data.iter().take(5).enumerate() {
                            result.push_str(&format!("Row {}: {}\n", i + 1, row.join(" | ")));
                        }
                    }
                }

                Ok(result)
            },
            _ => Ok("Extraction type not supported for XLSX".to_string()),
        }
    }

    fn extract_from_pptx(
        &self,
        input: &Path,
        extract_type: &super::ExtractType,
        _pages: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let doc = PptxDocument::from_file(input)?;

        match extract_type {
            super::ExtractType::Text => {
                Ok(doc.get_slide_text().unwrap_or_default())
            },
            super::ExtractType::Metadata => {
                let stats = doc.get_document_stats();
                let metadata = serde_json::json!({
                    "type": "pptx",
                    "slide_count": stats.slide_count,
                    "image_count": stats.image_count,
                    "animation_count": stats.animation_count,
                    "has_vba": stats.has_vba,
                    "author": stats.author,
                    "created": stats.created.to_rfc3339(),
                    "modified": stats.modified.to_rfc3339()
                });
                Ok(serde_json::to_string_pretty(&metadata)?)
            },
            super::ExtractType::All => {
                let mut result = String::new();
                result.push_str("=== PRESENTATION ANALYSIS ===\n\n");

                if let Ok(slides) = doc.get_all_slides() {
                    result.push_str(&format!("Total Slides: {}\n\n", slides.len()));

                    for (i, slide) in slides.iter().enumerate() {
                        result.push_str(&format!("Slide {}: {}\n", i + 1, slide.title));
                        for content in &slide.content {
                            result.push_str(&format!("  • {}\n", content));
                        }
                        result.push('\n');
                    }
                }

                Ok(result)
            },
            _ => Ok("Extraction type not supported for PPTX".to_string()),
        }
    }

    fn extract_from_pdf(
        &self,
        input: &Path,
        extract_type: &super::ExtractType,
        pages: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let doc = PdfDocument::from_file(input)?;

        match extract_type {
            super::ExtractType::Text => {
                Ok(doc.get_text().unwrap_or_default())
            },
            super::ExtractType::Metadata => {
                let stats = doc.get_document_stats();
                let metadata = serde_json::json!({
                    "type": "pdf",
                    "page_count": stats.page_count,
                    "image_count": stats.image_count,
                    "annotation_count": stats.annotation_count,
                    "form_field_count": stats.form_field_count,
                    "has_security": stats.has_security,
                    "has_javascript": stats.has_javascript,
                    "pdf_version": format!("{:?}", stats.pdf_version),
                    "author": doc.author,
                    "title": doc.title,
                    "created": stats.created.to_rfc3339(),
                    "modified": stats.modified.to_rfc3339()
                });
                Ok(serde_json::to_string_pretty(&metadata)?)
            },
            super::ExtractType::Forms => {
                let mut forms_info = String::new();
                forms_info.push_str(&format!("Form Fields: {}\n\n", doc.forms.len()));

                for (id, form) in &doc.forms {
                    forms_info.push_str(&format!("Field: {} ({})\n", form.name, id));
                    forms_info.push_str(&format!("  Type: {:?}\n", form.field_type));
                    forms_info.push_str(&format!("  Value: {}\n", form.value));
                    forms_info.push_str(&format!("  Page: {}\n\n", form.page + 1));
                }

                Ok(forms_info)
            },
            super::ExtractType::Annotations => {
                let mut annotations_info = String::new();
                annotations_info.push_str(&format!("Annotations: {}\n\n", doc.annotations.len()));

                for (id, annotation) in &doc.annotations {
                    annotations_info.push_str(&format!("Annotation: {} ({:?})\n", id, annotation.annotation_type));
                    annotations_info.push_str(&format!("  Author: {}\n", annotation.author));
                    annotations_info.push_str(&format!("  Content: {}\n", annotation.content));
                    annotations_info.push_str(&format!("  Page: {}\n\n", annotation.page + 1));
                }

                Ok(annotations_info)
            },
            super::ExtractType::All => {
                let mut result = String::new();
                result.push_str("=== PDF ANALYSIS ===\n\n");

                let stats = doc.get_document_stats();
                result.push_str(&format!("Pages: {}\n", stats.page_count));
                result.push_str(&format!("Images: {}\n", stats.image_count));
                result.push_str(&format!("Annotations: {}\n", stats.annotation_count));
                result.push_str(&format!("Form Fields: {}\n", stats.form_field_count));
                result.push_str(&format!("Security: {}\n", if stats.has_security { "Enabled" } else { "None" }));
                result.push_str(&format!("JavaScript: {}\n", if stats.has_javascript { "Present" } else { "None" }));
                result.push_str(&format!("PDF Version: {:?}\n\n", stats.pdf_version));

                result.push_str("Text Content:\n");
                result.push_str(&doc.get_text().unwrap_or_default());

                Ok(result)
            },
            _ => Ok("Extraction type not supported for PDF".to_string()),
        }
    }
}

pub struct DocumentAnalyzer;

impl DocumentAnalyzer {
    pub fn new() -> Self {
        DocumentAnalyzer
    }

    pub fn analyze(
        &self,
        input: &Path,
        analysis_type: &super::AnalysisType,
        output_format: &super::AnalysisOutputFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let extension = input.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let analysis_data = match extension.as_str() {
            "docx" => self.analyze_docx(input, analysis_type)?,
            "xlsx" => self.analyze_xlsx(input, analysis_type)?,
            "pptx" => self.analyze_pptx(input, analysis_type)?,
            "pdf" => self.analyze_pdf(input, analysis_type)?,
            _ => return Err(format!("Unsupported file format for analysis: {}", extension).into()),
        };

        self.format_analysis(&analysis_data, output_format)
    }

    fn analyze_docx(&self, input: &Path, analysis_type: &super::AnalysisType) -> Result<AnalysisData, Box<dyn std::error::Error>> {
        let doc = DocxDocument::from_file(input)?;
        let stats = doc.get_document_stats();

        let mut data = AnalysisData::new("docx".to_string());
        data.add_metric("page_count", stats.page_count as f64);
        data.add_metric("image_count", stats.image_count as f64);
        data.add_metric("table_count", stats.table_count as f64);
        data.add_property("has_vba", stats.has_vba.to_string());
        data.add_property("author", stats.author);
        data.add_property("created", stats.created.to_rfc3339());

        if let Ok(text) = doc.get_text() {
            data.add_metric("character_count", text.len() as f64);
            data.add_metric("word_count", text.split_whitespace().count() as f64);
        }

        match analysis_type {
            super::AnalysisType::Security => {
                data.add_property("vba_macros", if stats.has_vba { "Present - Security Risk" } else { "None" });
                data.add_property("security_level", if stats.has_vba { "Medium Risk" } else { "Low Risk" });
            },
            super::AnalysisType::Structure => {
                data.add_property("structure", format!("{} pages, {} images, {} tables", stats.page_count, stats.image_count, stats.table_count));
            },
            _ => {}
        }

        Ok(data)
    }

    fn analyze_xlsx(&self, input: &Path, analysis_type: &super::AnalysisType) -> Result<AnalysisData, Box<dyn std::error::Error>> {
        let doc = XlsxDocument::from_file(input)?;

        let mut data = AnalysisData::new("xlsx".to_string());
        data.add_metric("worksheet_count", doc.worksheets.len() as f64);

        let mut total_cells = 0;
        for (sheet_name, _) in &doc.worksheets {
            if let Ok(sheet_data) = doc.get_worksheet_data(sheet_name) {
                let cells_in_sheet = sheet_data.iter().map(|row| row.len()).sum::<usize>();
                total_cells += cells_in_sheet;
                data.add_property(&format!("sheet_{}_rows", sheet_name), sheet_data.len().to_string());
                data.add_property(&format!("sheet_{}_cols", sheet_name), sheet_data.first().map(|row| row.len()).unwrap_or(0).to_string());
            }
        }

        data.add_metric("total_cells", total_cells as f64);
        data.add_property("author", doc.author.clone());

        Ok(data)
    }

    fn analyze_pptx(&self, input: &Path, analysis_type: &super::AnalysisType) -> Result<AnalysisData, Box<dyn std::error::Error>> {
        let doc = PptxDocument::from_file(input)?;
        let stats = doc.get_document_stats();

        let mut data = AnalysisData::new("pptx".to_string());
        data.add_metric("slide_count", stats.slide_count as f64);
        data.add_metric("image_count", stats.image_count as f64);
        data.add_metric("animation_count", stats.animation_count as f64);
        data.add_property("has_vba", stats.has_vba.to_string());
        data.add_property("author", stats.author);

        if let Ok(slides) = doc.get_all_slides() {
            let total_text_length: usize = slides.iter()
                .map(|slide| slide.title.len() + slide.content.iter().map(|c| c.len()).sum::<usize>())
                .sum();

            data.add_metric("total_text_length", total_text_length as f64);
            data.add_metric("avg_content_per_slide", total_text_length as f64 / slides.len() as f64);
        }

        Ok(data)
    }

    fn analyze_pdf(&self, input: &Path, analysis_type: &super::AnalysisType) -> Result<AnalysisData, Box<dyn std::error::Error>> {
        let doc = PdfDocument::from_file(input)?;
        let stats = doc.get_document_stats();

        let mut data = AnalysisData::new("pdf".to_string());
        data.add_metric("page_count", stats.page_count as f64);
        data.add_metric("image_count", stats.image_count as f64);
        data.add_metric("annotation_count", stats.annotation_count as f64);
        data.add_metric("form_field_count", stats.form_field_count as f64);
        data.add_metric("attachment_count", stats.attachment_count as f64);
        data.add_property("has_security", stats.has_security.to_string());
        data.add_property("has_javascript", stats.has_javascript.to_string());
        data.add_property("pdf_version", format!("{:?}", stats.pdf_version));
        data.add_property("author", doc.author.clone());
        data.add_property("title", doc.title.clone());

        match analysis_type {
            super::AnalysisType::Security => {
                data.add_property("security_level", if stats.has_security { "High" } else { "None" });
                data.add_property("javascript_risk", if stats.has_javascript { "Present - Review Required" } else { "None" });
                data.add_property("form_risk", if stats.has_forms { "Interactive Forms Present" } else { "Static Document" });
            },
            super::AnalysisType::Structure => {
                data.add_property("structure_complexity",
                    if stats.form_field_count > 0 || stats.annotation_count > 0 { "Interactive" } else { "Static" });
            },
            _ => {}
        }

        Ok(data)
    }

    fn format_analysis(&self, data: &AnalysisData, format: &super::AnalysisOutputFormat) -> Result<String, Box<dyn std::error::Error>> {
        match format {
            super::AnalysisOutputFormat::Human => Ok(data.to_human_readable()),
            super::AnalysisOutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
            super::AnalysisOutputFormat::Csv => Ok(data.to_csv()),
            super::AnalysisOutputFormat::Markdown => Ok(data.to_markdown()),
            super::AnalysisOutputFormat::Html => Ok(data.to_html()),
        }
    }
}

pub struct DocumentMerger {
    config: Config,
}

impl DocumentMerger {
    pub fn new(config: &Config) -> Self {
        DocumentMerger {
            config: config.clone(),
        }
    }

    pub fn merge(
        &self,
        inputs: &[PathBuf],
        strategy: &super::MergeStrategy,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match strategy {
            super::MergeStrategy::Convert => self.merge_to_common_format(inputs),
            super::MergeStrategy::Preserve => Err("Preserve strategy not yet implemented".into()),
            super::MergeStrategy::Package => Err("Package strategy not yet implemented".into()),
        }
    }

    fn merge_to_common_format(&self, inputs: &[PathBuf]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Merge all documents into a single PDF
        let mut merged_pdf = PdfDocument::new();
        merged_pdf.set_title("Merged Document");
        merged_pdf.set_author("Office CLI");

        for input in inputs {
            let extension = input.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            match extension.as_str() {
                "docx" => {
                    let doc = DocxDocument::from_file(input)?;
                    self.add_docx_to_pdf(&doc, &mut merged_pdf)?;
                },
                "xlsx" => {
                    let doc = XlsxDocument::from_file(input)?;
                    self.add_xlsx_to_pdf(&doc, &mut merged_pdf)?;
                },
                "pptx" => {
                    let doc = PptxDocument::from_file(input)?;
                    self.add_pptx_to_pdf(&doc, &mut merged_pdf)?;
                },
                "pdf" => {
                    let doc = PdfDocument::from_file(input)?;
                    merged_pdf.merge_pages(&doc)?;
                },
                "md" | "markdown" => {
                    let content = fs::read_to_string(input)?;
                    let md_doc = office_sdk::markdown::MarkdownDocument::from_markdown(&content)?;
                    self.add_markdown_to_pdf(&md_doc, &mut merged_pdf)?;
                },
                _ => {
                    // Try to read as text
                    let content = fs::read_to_string(input)?;
                    let _page_id = merged_pdf.add_a4_page()?;
                    merged_pdf.add_paragraph(merged_pdf.pages.len() - 1, &content, 100.0, 750.0, 400.0, 11.0)?;
                }
            }
        }

        merged_pdf.to_bytes().map_err(|e| e.into())
    }

    fn add_docx_to_pdf(&self, doc: &DocxDocument, pdf: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        let _page_id = pdf.add_a4_page()?;
        let page_index = pdf.pages.len() - 1;

        if let Ok(text) = doc.get_text() {
            pdf.add_paragraph(page_index, &text, 100.0, 750.0, 400.0, 11.0)?;
        }

        Ok(())
    }

    fn add_xlsx_to_pdf(&self, doc: &XlsxDocument, pdf: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        let _page_id = pdf.add_a4_page()?;
        let page_index = pdf.pages.len() - 1;

        // Add each worksheet as a table
        for (sheet_name, _) in &doc.worksheets {
            pdf.add_heading(page_index, 2, &format!("Worksheet: {}", sheet_name), 100.0, 750.0)?;

            if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                if !data.is_empty() {
                    let headers: Vec<&str> = data[0].iter().map(|s| s.as_str()).collect();
                    let rows: Vec<Vec<&str>> = data.iter().skip(1)
                        .map(|row| row.iter().map(|s| s.as_str()).collect())
                        .collect();

                    if !headers.is_empty() && !rows.is_empty() {
                        pdf.add_table(page_index, 100.0, 700.0, &headers, &rows)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn add_pptx_to_pdf(&self, doc: &PptxDocument, pdf: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(slides) = doc.get_all_slides() {
            for slide in slides {
                let _page_id = pdf.add_a4_page()?;
                let page_index = pdf.pages.len() - 1;

                // Add slide title
                if !slide.title.is_empty() {
                    pdf.add_heading(page_index, 1, &slide.title, 100.0, 750.0)?;
                }

                // Add slide content
                let mut y_pos = 700.0;
                for content in &slide.content {
                    pdf.add_text(page_index, &format!("• {}", content), 120.0, y_pos, "Helvetica", 12.0)?;
                    y_pos -= 20.0;
                }
            }
        }

        Ok(())
    }

    fn add_markdown_to_pdf(&self, md_doc: &office_sdk::markdown::MarkdownDocument, pdf: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        let _page_id = pdf.add_a4_page()?;
        let page_index = pdf.pages.len() - 1;

        let markdown_text = md_doc.to_markdown();
        let mut y_position = 750.0;

        for line in markdown_text.lines() {
            if y_position < 100.0 {
                let _new_page = pdf.add_a4_page()?;
                y_position = 750.0;
            }

            if line.starts_with("# ") {
                let heading = line.trim_start_matches("# ");
                pdf.add_heading(pdf.pages.len() - 1, 1, heading, 100.0, y_position)?;
                y_position -= 20.0;
            } else if !line.trim().is_empty() {
                pdf.add_text(pdf.pages.len() - 1, line, 100.0, y_position, "Helvetica", 11.0)?;
                y_position -= 14.0;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisData {
    pub document_type: String,
    pub metrics: HashMap<String, f64>,
    pub properties: HashMap<String, String>,
    pub analysis_timestamp: String,
}

impl AnalysisData {
    pub fn new(doc_type: String) -> Self {
        AnalysisData {
            document_type: doc_type,
            metrics: HashMap::new(),
            properties: HashMap::new(),
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add_metric(&mut self, key: &str, value: f64) {
        self.metrics.insert(key.to_string(), value);
    }

    pub fn add_property(&mut self, key: &str, value: impl Into<String>) {
        self.properties.insert(key.to_string(), value.into());
    }

    pub fn to_human_readable(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("📄 Document Analysis - {} Format\n", self.document_type.to_uppercase()));
        output.push_str(&format!("🕒 Analyzed: {}\n\n", self.analysis_timestamp));

        output.push_str("📊 Metrics:\n");
        for (key, value) in &self.metrics {
            output.push_str(&format!("   {}: {}\n", key.replace('_', " "),
                if value.fract() == 0.0 { format!("{}", *value as i64) } else { format!("{:.2}", value) }));
        }

        output.push_str("\n🔍 Properties:\n");
        for (key, value) in &self.properties {
            output.push_str(&format!("   {}: {}\n", key.replace('_', " "), value));
        }

        output
    }

    pub fn to_csv(&self) -> String {
        let mut csv = String::from("key,value,type\n");

        for (key, value) in &self.metrics {
            csv.push_str(&format!("{},{},metric\n", key, value));
        }

        for (key, value) in &self.properties {
            csv.push_str(&format!("{},{},property\n", key, value));
        }

        csv
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Document Analysis - {}\n\n", self.document_type.to_uppercase()));
        md.push_str(&format!("**Analyzed:** {}\n\n", self.analysis_timestamp));

        if !self.metrics.is_empty() {
            md.push_str("## Metrics\n\n");
            md.push_str("| Metric | Value |\n");
            md.push_str("|--------|-------|\n");

            for (key, value) in &self.metrics {
                md.push_str(&format!("| {} | {} |\n",
                    key.replace('_', " "),
                    if value.fract() == 0.0 { format!("{}", *value as i64) } else { format!("{:.2}", value) }
                ));
            }
            md.push_str("\n");
        }

        if !self.properties.is_empty() {
            md.push_str("## Properties\n\n");
            md.push_str("| Property | Value |\n");
            md.push_str("|----------|-------|\n");

            for (key, value) in &self.properties {
                md.push_str(&format!("| {} | {} |\n", key.replace('_', " "), value));
            }
        }

        md
    }

    pub fn to_html(&self) -> String {
        let mut html = String::from(r#"<!DOCTYPE html>
<html><head><title>Document Analysis</title>
<style>
body{font-family:Arial,sans-serif;max-width:800px;margin:0 auto;padding:20px;}
table{border-collapse:collapse;width:100%;margin:10px 0;}
th,td{border:1px solid #ddd;padding:8px;text-align:left;}
th{background-color:#f2f2f2;}
h1{color:#333;}h2{color:#666;}
</style></head><body>"#);

        html.push_str(&format!("<h1>Document Analysis - {}</h1>", self.document_type.to_uppercase()));
        html.push_str(&format!("<p><strong>Analyzed:</strong> {}</p>", self.analysis_timestamp));

        if !self.metrics.is_empty() {
            html.push_str("<h2>Metrics</h2><table><tr><th>Metric</th><th>Value</th></tr>");
            for (key, value) in &self.metrics {
                html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>",
                    key.replace('_', " "),
                    if value.fract() == 0.0 { format!("{}", *value as i64) } else { format!("{:.2}", value) }
                ));
            }
            html.push_str("</table>");
        }

        if !self.properties.is_empty() {
            html.push_str("<h2>Properties</h2><table><tr><th>Property</th><th>Value</th></tr>");
            for (key, value) in &self.properties {
                html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", key.replace('_', " "), html_escape(value)));
            }
            html.push_str("</table>");
        }

        html.push_str("</body></html>");
        html
    }
}

// Utility functions
pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn detect_file_type(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_type = match extension.as_str() {
        "md" | "markdown" => "markdown",
        "txt" | "text" => "text",
        "docx" => "docx",
        "xlsx" => "xlsx",
        "pptx" => "pptx",
        "pdf" => "pdf",
        "html" | "htm" => "html",
        "json" => "json",
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" => "image",
        _ => "unknown",
    };

    Ok(file_type.to_string())
}

pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn validate_input_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()).into());
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()).into());
    }

    let metadata = fs::metadata(path)?;
    if metadata.len() == 0 {
        return Err(format!("File is empty: {}", path.display()).into());
    }

    // Check if file is too large (>100MB warning)
    if metadata.len() > 100 * 1024 * 1024 {
        eprintln!("⚠️  Warning: Large file detected ({}) - processing may take time", format_file_size(metadata.len()));
    }

    Ok(())
}

pub fn create_backup(original: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let backup_path = original.with_extension(
        format!("{}.backup",
            original.extension().and_then(|ext| ext.to_str()).unwrap_or("bak"))
    );

    fs::copy(original, &backup_path)?;
    Ok(backup_path)
}

pub fn ensure_output_directory(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}