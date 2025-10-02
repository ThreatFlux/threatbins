use crate::cli::config::Config;
use crate::cli::utils::*;
use crate::OfficeDocument;
use crate::docx::DocxDocument;
use crate::xlsx::XlsxDocument;
use crate::pptx::PptxDocument;
use crate::pdf::PdfDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub doc_type: super::DocumentType,
    pub features: Vec<String>,
}

pub struct TemplateManager {
    config: Config,
    template_dir: PathBuf,
}

impl TemplateManager {
    pub fn new(config: &Config) -> Self {
        let template_dir = config.template_dir.clone()
            .unwrap_or_else(|| PathBuf::from("templates"));

        TemplateManager {
            config: config.clone(),
            template_dir,
        }
    }

    pub fn list_templates(&self) -> Result<HashMap<String, TemplateInfo>, Box<dyn std::error::Error>> {
        let mut templates = HashMap::new();

        // Add built-in templates
        templates.extend(self.get_builtin_templates());

        // Add user templates from directory
        if self.template_dir.exists() {
            for entry in fs::read_dir(&self.template_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(template_info) = self.load_template_info(&path) {
                        templates.insert(template_info.name.clone(), template_info);
                    }
                }
            }
        }

        Ok(templates)
    }

    pub fn show_template(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let templates = self.list_templates()?;

        if let Some(template) = templates.get(name) {
            println!("📋 Template: {}", template.name);
            println!("   Description: {}", template.description);
            println!("   Type: {:?}", template.doc_type);
            println!("   Features: {}", template.features.join(", "));
        } else {
            return Err(format!("Template '{}' not found", name).into());
        }

        Ok(())
    }

    pub fn create_template(&self, name: &str, from: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.template_dir)?;

        let template_info = if let Some(base_template) = from {
            // Copy from existing template
            let templates = self.list_templates()?;
            if let Some(base) = templates.get(base_template) {
                TemplateInfo {
                    name: name.to_string(),
                    description: format!("Based on {}", base.description),
                    doc_type: base.doc_type.clone(),
                    features: base.features.clone(),
                }
            } else {
                return Err(format!("Base template '{}' not found", base_template).into());
            }
        } else {
            // Create new template
            TemplateInfo {
                name: name.to_string(),
                description: "Custom template".to_string(),
                doc_type: super::DocumentType::Document,
                features: vec!["basic".to_string()],
            }
        };

        let template_path = self.template_dir.join(format!("{}.json", name));
        let json_data = serde_json::to_string_pretty(&template_info)?;
        fs::write(&template_path, json_data)?;

        Ok(())
    }

    pub fn edit_template(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Interactive template editing would go here
        println!("Template editing not yet implemented. Use external editor for now.");
        Ok(())
    }

    pub fn delete_template(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let template_path = self.template_dir.join(format!("{}.json", name));

        if template_path.exists() {
            fs::remove_file(&template_path)?;
            Ok(())
        } else {
            Err(format!("Template '{}' not found", name).into())
        }
    }

    pub fn install_template(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let template_info: TemplateInfo = serde_json::from_str(&fs::read_to_string(path)?)?;

        fs::create_dir_all(&self.template_dir)?;
        let dest_path = self.template_dir.join(format!("{}.json", template_info.name));

        fs::copy(path, &dest_path)?;

        Ok(template_info.name)
    }

    pub fn export_template(&self, name: &str, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let template_path = self.template_dir.join(format!("{}.json", name));

        if template_path.exists() {
            fs::copy(&template_path, output)?;
            Ok(())
        } else {
            Err(format!("Template '{}' not found", name).into())
        }
    }

    fn get_builtin_templates(&self) -> HashMap<String, TemplateInfo> {
        let mut templates = HashMap::new();

        templates.insert("business".to_string(), TemplateInfo {
            name: "business".to_string(),
            description: "Professional business document with header, footer, and corporate styling".to_string(),
            doc_type: super::DocumentType::Document,
            features: vec![
                "header".to_string(),
                "footer".to_string(),
                "toc".to_string(),
                "page_numbers".to_string(),
                "corporate_colors".to_string(),
            ],
        });

        templates.insert("academic".to_string(), TemplateInfo {
            name: "academic".to_string(),
            description: "Academic paper template with proper formatting and citation support".to_string(),
            doc_type: super::DocumentType::Document,
            features: vec![
                "title_page".to_string(),
                "abstract".to_string(),
                "bibliography".to_string(),
                "footnotes".to_string(),
                "double_spacing".to_string(),
            ],
        });

        templates.insert("report".to_string(), TemplateInfo {
            name: "report".to_string(),
            description: "Technical report template with sections and appendices".to_string(),
            doc_type: super::DocumentType::Document,
            features: vec![
                "cover_page".to_string(),
                "executive_summary".to_string(),
                "sections".to_string(),
                "appendices".to_string(),
                "charts".to_string(),
            ],
        });

        templates.insert("presentation".to_string(), TemplateInfo {
            name: "presentation".to_string(),
            description: "Modern presentation template with consistent styling".to_string(),
            doc_type: super::DocumentType::Presentation,
            features: vec![
                "title_slide".to_string(),
                "content_slides".to_string(),
                "slide_numbers".to_string(),
                "animations".to_string(),
                "modern_design".to_string(),
            ],
        });

        templates.insert("spreadsheet".to_string(), TemplateInfo {
            name: "spreadsheet".to_string(),
            description: "Professional spreadsheet template with formatting and charts".to_string(),
            doc_type: super::DocumentType::Spreadsheet,
            features: vec![
                "headers".to_string(),
                "formatting".to_string(),
                "charts".to_string(),
                "data_validation".to_string(),
                "formulas".to_string(),
            ],
        });

        templates.insert("invoice".to_string(), TemplateInfo {
            name: "invoice".to_string(),
            description: "Professional invoice template with calculations".to_string(),
            doc_type: super::DocumentType::Document,
            features: vec![
                "company_header".to_string(),
                "client_details".to_string(),
                "line_items".to_string(),
                "calculations".to_string(),
                "terms".to_string(),
            ],
        });

        templates.insert("form".to_string(), TemplateInfo {
            name: "form".to_string(),
            description: "Interactive PDF form template with validation".to_string(),
            doc_type: super::DocumentType::Pdf,
            features: vec![
                "form_fields".to_string(),
                "validation".to_string(),
                "submit_button".to_string(),
                "professional_layout".to_string(),
            ],
        });

        templates.insert("manual".to_string(), TemplateInfo {
            name: "manual".to_string(),
            description: "User manual template with table of contents and sections".to_string(),
            doc_type: super::DocumentType::Document,
            features: vec![
                "cover_page".to_string(),
                "toc".to_string(),
                "chapters".to_string(),
                "images".to_string(),
                "appendix".to_string(),
            ],
        });

        templates
    }

    fn load_template_info(&self, path: &Path) -> Result<TemplateInfo, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let template_info: TemplateInfo = serde_json::from_str(&content)?;
        Ok(template_info)
    }
}

pub struct DocumentCreator {
    config: Config,
    template: Option<String>,
    metadata: super::converter::DocumentMetadata,
    content: Vec<ContentBlock>,
    images: Vec<PathBuf>,
    options: HashMap<String, super::converter::ConversionOption>,
}

#[derive(Debug, Clone)]
pub enum ContentBlock {
    Text(String),
    Heading { level: u32, text: String },
    List(Vec<String>),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Image { path: PathBuf, caption: Option<String> },
    Code { language: Option<String>, content: String },
}

impl DocumentCreator {
    pub fn new(config: &Config, template: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(DocumentCreator {
            config: config.clone(),
            template: template.map(|s| s.to_string()),
            metadata: super::converter::DocumentMetadata::default(),
            content: Vec::new(),
            images: Vec::new(),
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

    pub fn set_option<T: Into<super::converter::ConversionOption>>(&mut self, key: &str, value: T) {
        self.options.insert(key.to_string(), value.into());
    }

    pub fn add_text_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse content and create content blocks
        let blocks = self.parse_text_content(content);
        self.content.extend(blocks);
        Ok(())
    }

    pub fn add_directory_content(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Read all text files in directory and combine
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "md" | "markdown" | "txt" => {
                            let content = fs::read_to_string(&path)?;
                            self.add_text_content(&content)?;
                        },
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" => {
                            self.images.push(path);
                        },
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    pub fn add_image(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if path.exists() {
            self.images.push(path.to_path_buf());
        }
        Ok(())
    }

    pub fn create_document(&self, doc_type: &super::DocumentType) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match doc_type {
            super::DocumentType::Document => self.create_docx(),
            super::DocumentType::Spreadsheet => self.create_xlsx(),
            super::DocumentType::Presentation => self.create_pptx(),
            super::DocumentType::Pdf => self.create_pdf(),
            super::DocumentType::Report => {
                // Auto-detect best format based on content
                if self.has_tabular_data() {
                    self.create_xlsx()
                } else if self.has_slide_structure() {
                    self.create_pptx()
                } else {
                    self.create_pdf() // Default to PDF for reports
                }
            },
        }
    }

    fn create_docx(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut doc = DocxDocument::new();

        // Apply template-based configuration
        if let Some(ref template_name) = self.template {
            self.apply_docx_template(&mut doc, template_name)?;
        }

        // Add metadata
        if let Some(ref title) = self.metadata.title {
            // doc.set_title(title); // Method needs to be implemented
        }
        if let Some(ref author) = self.metadata.author {
            // doc.set_author(author);
        }

        // Add content blocks
        for block in &self.content {
            match block {
                ContentBlock::Heading { level, text } => {
                    doc.add_heading(*level, text)?;
                },
                ContentBlock::Text(text) => {
                    doc.add_paragraph(text)?;
                },
                ContentBlock::List(items) => {
                    for (i, item) in items.iter().enumerate() {
                        doc.add_bulleted_list_item(item)?;
                    }
                },
                ContentBlock::Table { headers, rows } => {
                    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
                    let row_refs: Vec<Vec<&str>> = rows.iter()
                        .map(|row| row.iter().map(|s| s.as_str()).collect())
                        .collect();
                    doc.create_table(&header_refs, &row_refs)?;
                },
                ContentBlock::Code { language: _, content } => {
                    doc.add_code_block(content)?;
                },
                ContentBlock::Image { path, caption } => {
                    if path.exists() {
                        // doc.add_image(path, caption.as_deref())?; // Method needs implementation
                    }
                },
            }
        }

        // Apply options
        self.apply_docx_options(&mut doc)?;

        doc.to_bytes().map_err(|e| e.into())
    }

    fn create_xlsx(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut doc = XlsxDocument::new();

        // Find table data in content blocks
        let mut worksheet_data = Vec::new();

        // Add title row if title exists
        if let Some(ref title) = self.metadata.title {
            worksheet_data.push(vec![title.clone()]);
            worksheet_data.push(vec![]); // Empty row
        }

        // Process content blocks
        for block in &self.content {
            match block {
                ContentBlock::Table { headers, rows } => {
                    worksheet_data.push(headers.clone());
                    worksheet_data.extend(rows.clone());
                    worksheet_data.push(vec![]); // Empty row between tables
                },
                ContentBlock::Heading { level: _, text } => {
                    worksheet_data.push(vec![text.clone()]);
                },
                ContentBlock::Text(text) => {
                    // Split long text into cells
                    let words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
                    if !words.is_empty() {
                        worksheet_data.push(words);
                    }
                },
                ContentBlock::List(items) => {
                    for item in items {
                        worksheet_data.push(vec![format!("• {}", item)]);
                    }
                },
                _ => {} // Skip other types for spreadsheet
            }
        }

        if !worksheet_data.is_empty() {
            doc.add_worksheet("Data", &worksheet_data)?;
        }

        doc.to_bytes().map_err(|e| e.into())
    }

    fn create_pptx(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut doc = PptxDocument::new();

        // Apply metadata
        if let Some(ref title) = self.metadata.title {
            doc.set_title(title);
        }
        if let Some(ref author) = self.metadata.author {
            doc.set_author(author);
        }

        // Create title slide if title exists
        if let Some(ref title) = self.metadata.title {
            let subtitle = self.metadata.subject.as_deref().unwrap_or("Presentation");
            doc.add_title_slide(title, subtitle)?;
        }

        // Convert content blocks to slides
        let mut current_slide_title = "Content".to_string();
        let mut current_slide_content = Vec::new();

        for block in &self.content {
            match block {
                ContentBlock::Heading { level, text } => {
                    // Create new slide with previous content
                    if !current_slide_content.is_empty() {
                        let content_refs: Vec<&str> = current_slide_content.iter().map(|s| s.as_str()).collect();
                        doc.add_slide(&current_slide_title, &content_refs)?;
                        current_slide_content.clear();
                    }

                    if *level <= 2 {
                        // Major heading - new slide title
                        current_slide_title = text.clone();
                    } else {
                        // Minor heading - add to content
                        current_slide_content.push(text.clone());
                    }
                },
                ContentBlock::Text(text) => {
                    current_slide_content.push(text.clone());
                },
                ContentBlock::List(items) => {
                    current_slide_content.extend(items.clone());
                },
                _ => {} // Skip complex types for now
            }
        }

        // Add final slide
        if !current_slide_content.is_empty() {
            let content_refs: Vec<&str> = current_slide_content.iter().map(|s| s.as_str()).collect();
            doc.add_slide(&current_slide_title, &content_refs)?;
        }

        doc.to_bytes().map_err(|e| e.into())
    }

    fn create_pdf(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut doc = PdfDocument::new();

        // Apply metadata
        if let Some(ref title) = self.metadata.title {
            doc.set_title(title);
        }
        if let Some(ref author) = self.metadata.author {
            doc.set_author(author);
        }
        if let Some(ref subject) = self.metadata.subject {
            doc.set_subject(subject);
        }

        // Add first page
        let _page_id = doc.add_a4_page()?;
        let mut current_page = 0;
        let mut y_position = 750.0;
        let line_height = 14.0;

        // Add title if exists
        if let Some(ref title) = self.metadata.title {
            doc.add_heading(current_page, 1, title, 100.0, y_position)?;
            y_position -= line_height * 3.0;
        }

        // Process content blocks
        for block in &self.content {
            // Check if we need a new page
            if y_position < 100.0 {
                let _new_page = doc.add_a4_page()?;
                current_page += 1;
                y_position = 750.0;
            }

            match block {
                ContentBlock::Heading { level, text } => {
                    doc.add_heading(current_page, *level, text, 100.0, y_position)?;
                    y_position -= line_height * (3.0 - (*level as f64 * 0.3));
                },
                ContentBlock::Text(text) => {
                    // Use paragraph method for better text wrapping
                    let lines_needed = (text.len() as f64 / 80.0).ceil() as u32; // Estimate lines
                    let space_needed = lines_needed as f64 * line_height;

                    if y_position - space_needed < 100.0 {
                        let _new_page = doc.add_a4_page()?;
                        current_page += 1;
                        y_position = 750.0;
                    }

                    doc.add_paragraph(current_page, text, 100.0, y_position, 400.0, 11.0)?;
                    y_position -= space_needed + 10.0; // Add some spacing
                },
                ContentBlock::List(items) => {
                    for item in items {
                        if y_position < 100.0 {
                            let _new_page = doc.add_a4_page()?;
                            current_page += 1;
                            y_position = 750.0;
                        }
                        doc.add_text(current_page, &format!("• {}", item), 120.0, y_position, "Helvetica", 11.0)?;
                        y_position -= line_height;
                    }
                    y_position -= 10.0; // Extra spacing after list
                },
                ContentBlock::Table { headers, rows } => {
                    if y_position < 200.0 { // Tables need more space
                        let _new_page = doc.add_a4_page()?;
                        current_page += 1;
                        y_position = 750.0;
                    }

                    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
                    let row_refs: Vec<Vec<&str>> = rows.iter()
                        .map(|row| row.iter().map(|s| s.as_str()).collect())
                        .collect();

                    doc.add_table(current_page, 100.0, y_position, &header_refs, &row_refs)?;
                    let table_height = (rows.len() + 1) as f64 * 20.0; // Estimate table height
                    y_position -= table_height + 20.0;
                },
                ContentBlock::Code { language: _, content } => {
                    let lines = content.lines().count();
                    let space_needed = lines as f64 * 12.0 + 20.0;

                    if y_position - space_needed < 100.0 {
                        let _new_page = doc.add_a4_page()?;
                        current_page += 1;
                        y_position = 750.0;
                    }

                    // Add code block with background
                    doc.add_rectangle(current_page, 95.0, y_position - 15.0, 410.0, space_needed, Some(office_sdk::pdf::Color::gray(0.95)), Some(office_sdk::pdf::Color::gray(0.8)))?;

                    let mut code_y = y_position;
                    for line in content.lines() {
                        doc.add_text(current_page, line, 105.0, code_y, "Courier", 10.0)?;
                        code_y -= 12.0;
                    }

                    y_position -= space_needed + 10.0;
                },
                ContentBlock::Image { path, caption } => {
                    if path.exists() {
                        if y_position < 300.0 { // Images need space
                            let _new_page = doc.add_a4_page()?;
                            current_page += 1;
                            y_position = 750.0;
                        }

                        if let Ok(image_id) = doc.add_image(path, caption.as_deref()) {
                            doc.add_image_to_page(current_page, &image_id, 150.0, y_position - 200.0, 300.0, 200.0)?;
                            if let Some(ref cap) = caption {
                                doc.add_text(current_page, cap, 150.0, y_position - 220.0, "Helvetica", 10.0)?;
                            }
                            y_position -= 250.0;
                        }
                    }
                },
            }
        }

        // Apply options like password protection
        self.apply_pdf_options(&mut doc)?;

        doc.to_bytes().map_err(|e| e.into())
    }

    fn parse_text_content(&self, content: &str) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check for headings
            if line.starts_with('#') {
                let level = line.chars().take_while(|&c| c == '#').count() as u32;
                let text = line.trim_start_matches('#').trim().to_string();
                blocks.push(ContentBlock::Heading { level, text });
            }
            // Check for code blocks
            else if line.starts_with("```") {
                let language = if line.len() > 3 {
                    Some(line[3..].trim().to_string())
                } else {
                    None
                };

                i += 1;
                let mut code_content = String::new();

                while i < lines.len() && !lines[i].trim().starts_with("```") {
                    if !code_content.is_empty() {
                        code_content.push('\n');
                    }
                    code_content.push_str(lines[i]);
                    i += 1;
                }

                blocks.push(ContentBlock::Code { language, content: code_content });
            }
            // Check for lists
            else if line.starts_with("- ") || line.starts_with("* ") ||
                    (line.chars().next().map_or(false, |c| c.is_ascii_digit()) && line.contains(". ")) {
                let mut list_items = Vec::new();

                while i < lines.len() {
                    let list_line = lines[i].trim();
                    if list_line.starts_with("- ") || list_line.starts_with("* ") {
                        let item = list_line.trim_start_matches(&['-', '*', ' '][..]).trim().to_string();
                        list_items.push(item);
                    } else if list_line.chars().next().map_or(false, |c| c.is_ascii_digit()) && list_line.contains(". ") {
                        if let Some(dot_pos) = list_line.find(". ") {
                            let item = list_line[dot_pos + 2..].trim().to_string();
                            list_items.push(item);
                        }
                    } else if list_line.is_empty() {
                        break;
                    } else {
                        break;
                    }
                    i += 1;
                }

                if !list_items.is_empty() {
                    blocks.push(ContentBlock::List(list_items));
                }
                i -= 1; // Adjust for the outer loop increment
            }
            // Check for tables (simple detection)
            else if line.contains('|') && lines.get(i + 1).map_or(false, |next| next.contains('|')) {
                let mut table_lines = Vec::new();

                while i < lines.len() && lines[i].contains('|') {
                    table_lines.push(lines[i]);
                    i += 1;
                }

                if table_lines.len() >= 2 {
                    let headers: Vec<String> = table_lines[0]
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    let mut rows = Vec::new();
                    for line in table_lines.iter().skip(2) { // Skip header and separator
                        let row: Vec<String> = line
                            .split('|')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        if row.len() == headers.len() {
                            rows.push(row);
                        }
                    }

                    if !headers.is_empty() && !rows.is_empty() {
                        blocks.push(ContentBlock::Table { headers, rows });
                    }
                }
                i -= 1; // Adjust for outer loop
            }
            // Regular text
            else {
                let mut paragraph = String::new();

                while i < lines.len() {
                    let text_line = lines[i].trim();
                    if text_line.is_empty() {
                        break;
                    }

                    // Check if next line starts a new block type
                    if text_line.starts_with('#') || text_line.starts_with("```") ||
                       text_line.starts_with("- ") || text_line.starts_with("* ") ||
                       text_line.contains('|') {
                        break;
                    }

                    if !paragraph.is_empty() {
                        paragraph.push(' ');
                    }
                    paragraph.push_str(text_line);
                    i += 1;
                }

                if !paragraph.is_empty() {
                    blocks.push(ContentBlock::Text(paragraph));
                }
                i -= 1; // Adjust for outer loop
            }

            i += 1;
        }

        blocks
    }

    fn has_tabular_data(&self) -> bool {
        self.content.iter().any(|block| matches!(block, ContentBlock::Table { .. }))
    }

    fn has_slide_structure(&self) -> bool {
        let heading_count = self.content.iter()
            .filter(|block| matches!(block, ContentBlock::Heading { level, .. } if *level <= 2))
            .count();

        heading_count >= 3 // If we have 3+ major headings, it's probably slide-like
    }

    fn apply_docx_template(&self, _doc: &mut DocxDocument, _template_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Template application logic for DOCX
        Ok(())
    }

    fn apply_docx_options(&self, _doc: &mut DocxDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Apply various DOCX options
        Ok(())
    }

    fn apply_pdf_options(&self, doc: &mut PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Apply PDF-specific options
        if let Some(super::converter::ConversionOption::String(password)) = self.options.get("password") {
            doc.set_password_protection(Some(password), None);
        }

        if let Some(super::converter::ConversionOption::Boolean(true)) = self.options.get("document_security") {
            let mut permissions = office_sdk::pdf::PermissionFlags::default();
            permissions.allow_copy = false;
            permissions.allow_modify_contents = false;
            doc.set_permissions(permissions);
        }

        Ok(())
    }
}