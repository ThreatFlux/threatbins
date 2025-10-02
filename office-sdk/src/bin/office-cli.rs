use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use office_sdk::OfficeDocument;
use office_sdk::docx::DocxDocument;
use office_sdk::xlsx::XlsxDocument;
use office_sdk::pptx::PptxDocument;
use office_sdk::pdf::PdfDocument;
use office_sdk::markdown::MarkdownDocument;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

/// Office SDK CLI - Professional document creation and conversion tool
#[derive(Parser)]
#[command(name = "office-cli")]
#[command(about = "A comprehensive CLI for creating and converting Office documents")]
#[command(version = "1.0.0")]
#[command(author = "Office SDK Team")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Force overwrite existing files
    #[arg(short, long, global = true)]
    pub force: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert files between different Office formats
    Convert {
        /// Input file path
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output file path
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Output format (auto-detected from extension if not specified)
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Document title
        #[arg(long)]
        title: Option<String>,

        /// Document author
        #[arg(long)]
        author: Option<String>,

        /// Password protect the document
        #[arg(long)]
        password: Option<String>,
    },

    /// Extract content from existing documents
    Extract {
        /// Input document
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// What to extract
        #[arg(short, long, value_enum, default_value = "text")]
        extract_type: ExtractType,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze and inspect documents
    Analyze {
        /// Input document
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Save analysis to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Microsoft Word document
    Docx,
    /// Microsoft Excel spreadsheet
    Xlsx,
    /// Microsoft PowerPoint presentation
    Pptx,
    /// Portable Document Format
    Pdf,
    /// Markdown document
    Markdown,
    /// Plain text
    Text,
    /// JSON metadata
    Json,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ExtractType {
    /// Extract all text content
    Text,
    /// Extract metadata
    Metadata,
    /// Extract everything
    All,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if !cli.quiet {
        println!("{}", "🏢 Office SDK CLI v1.0.0".bright_blue().bold());
        println!("{}", "Professional document creation and conversion tool".bright_white());
        println!();
    }

    match cli.command {
        Commands::Convert {
            input,
            output,
            format,
            title,
            author,
            password
        } => {
            handle_convert(input, output, format, title, author, password, cli.verbose, cli.quiet, cli.force)?;
        },

        Commands::Extract {
            input,
            extract_type,
            output
        } => {
            handle_extract(input, extract_type, output, cli.verbose, cli.quiet)?;
        },

        Commands::Analyze {
            input,
            output
        } => {
            handle_analyze(input, output, cli.verbose, cli.quiet)?;
        },
    }

    if !cli.quiet {
        println!("\n{}", "✨ Operation completed successfully!".bright_green().bold());
    }

    Ok(())
}

fn handle_convert(
    input: PathBuf,
    output: Option<PathBuf>,
    format: Option<OutputFormat>,
    title: Option<String>,
    author: Option<String>,
    password: Option<String>,
    verbose: bool,
    quiet: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input.display()).into());
    }

    // Auto-detect output format and path
    let (output_path, output_format) = determine_output(&input, output, format)?;

    // Check if output exists and handle force flag
    if output_path.exists() && !force {
        if !quiet {
            use dialoguer::Confirm;
            let overwrite = Confirm::new()
                .with_prompt(format!("Output file '{}' already exists. Overwrite?", output_path.display()))
                .interact()?;

            if !overwrite {
                println!("{}", "Conversion cancelled.".yellow());
                return Ok(());
            }
        } else {
            return Err("Output file exists. Use --force to overwrite.".into());
        }
    }

    if verbose {
        println!("📁 Input: {}", input.display());
        println!("📄 Output: {}", output_path.display());
        println!("🔄 Format: {:?}", output_format);
    }

    // Create progress bar
    let pb = if !quiet {
        let pb = ProgressBar::new(100);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}/{len:3} {msg}")
            .unwrap()
            .progress_chars("#>-"));
        pb.set_message("Starting conversion...");
        Some(pb)
    } else {
        None
    };

    // Load and parse input document
    if let Some(ref pb) = pb { pb.set_position(20); pb.set_message("Reading input file..."); }

    let input_content = fs::read_to_string(&input)?;
    let input_extension = input.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if let Some(ref pb) = pb { pb.set_position(40); pb.set_message("Converting document..."); }

    // Perform conversion
    let output_data = match input_extension.as_str() {
        "md" | "markdown" => {
            convert_markdown(&input_content, &output_format, title.as_deref(), author.as_deref(), password.as_deref())?
        },
        "txt" | "text" => {
            convert_text(&input_content, &output_format, title.as_deref(), author.as_deref(), password.as_deref())?
        },
        "docx" => {
            let doc = DocxDocument::from_file(&input)?;
            convert_from_docx(&doc, &output_format)?
        },
        "xlsx" => {
            let doc = XlsxDocument::from_file(&input)?;
            convert_from_xlsx(&doc, &output_format)?
        },
        "pptx" => {
            let doc = PptxDocument::from_file(&input)?;
            convert_from_pptx(&doc, &output_format)?
        },
        "pdf" => {
            let doc = PdfDocument::from_file(&input)?;
            convert_from_pdf(&doc, &output_format)?
        },
        _ => {
            return Err(format!("Unsupported input format: {}", input_extension).into());
        }
    };

    if let Some(ref pb) = pb { pb.set_position(80); pb.set_message("Writing output file..."); }

    // Write output file
    fs::write(&output_path, output_data)?;

    if let Some(ref pb) = pb {
        pb.set_position(100);
        pb.set_message("Conversion complete!");
        pb.finish_and_clear();
    }

    if !quiet {
        println!("{} Converted {} to {}",
            "✅".green(),
            input.display().to_string().bright_white(),
            output_path.display().to_string().bright_white()
        );

        // Show file size
        let size = fs::metadata(&output_path)?.len();
        println!("📊 Output size: {}", format_size(size).bright_cyan());
    }

    Ok(())
}

fn handle_extract(
    input: PathBuf,
    extract_type: ExtractType,
    output: Option<PathBuf>,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Extracting {:?} from {}...", "🔍".bright_blue(), extract_type, input.display());
    }

    let extracted_data = extract_content(&input, &extract_type)?;

    if let Some(output_path) = output {
        fs::write(&output_path, &extracted_data)?;
        if !quiet {
            println!("{} Extracted content saved to {}", "✅".green(), output_path.display());
        }
    } else {
        // Output to stdout
        println!("{}", extracted_data);
    }

    Ok(())
}

fn handle_analyze(
    input: PathBuf,
    output: Option<PathBuf>,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Analyzing document: {}...", "🔬".bright_blue(), input.display());
    }

    let analysis_result = analyze_document(&input)?;

    if let Some(output_path) = output {
        fs::write(&output_path, &analysis_result)?;
        if !quiet {
            println!("{} Analysis saved to {}", "✅".green(), output_path.display());
        }
    } else {
        println!("{}", analysis_result);
    }

    Ok(())
}

// Utility functions
fn determine_output(
    input: &Path,
    output: Option<PathBuf>,
    format: Option<OutputFormat>,
) -> Result<(PathBuf, OutputFormat), Box<dyn std::error::Error>> {
    match (output, format) {
        (Some(path), Some(fmt)) => Ok((path, fmt)),
        (Some(path), None) => {
            // Detect format from extension
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let fmt = match ext.as_str() {
                "docx" => OutputFormat::Docx,
                "xlsx" => OutputFormat::Xlsx,
                "pptx" => OutputFormat::Pptx,
                "pdf" => OutputFormat::Pdf,
                "md" => OutputFormat::Markdown,
                "txt" => OutputFormat::Text,
                "json" => OutputFormat::Json,
                _ => return Err(format!("Cannot determine format from extension: {}", ext).into()),
            };

            Ok((path, fmt))
        },
        (None, Some(fmt)) => {
            // Generate output path from input
            let stem = input.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            let ext = match fmt {
                OutputFormat::Docx => "docx",
                OutputFormat::Xlsx => "xlsx",
                OutputFormat::Pptx => "pptx",
                OutputFormat::Pdf => "pdf",
                OutputFormat::Markdown => "md",
                OutputFormat::Text => "txt",
                OutputFormat::Json => "json",
            };

            let output_path = input.with_file_name(format!("{}.{}", stem, ext));
            Ok((output_path, fmt))
        },
        (None, None) => {
            // Default to PDF
            let stem = input.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let output_path = input.with_file_name(format!("{}.pdf", stem));
            Ok((output_path, OutputFormat::Pdf))
        },
    }
}

// Conversion functions
fn convert_markdown(
    markdown: &str,
    output_format: &OutputFormat,
    title: Option<&str>,
    author: Option<&str>,
    password: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let md_doc = MarkdownDocument::from_markdown(markdown)?;

    match output_format {
        OutputFormat::Docx => {
            let mut docx = md_doc.to_docx()?;
            // Apply metadata would go here when methods are available
            docx.to_bytes().map_err(|e| e.into())
        },
        OutputFormat::Pdf => {
            let mut pdf = PdfDocument::new();

            if let Some(title) = title {
                pdf.set_title(title);
            }
            if let Some(author) = author {
                pdf.set_author(author);
            }
            if let Some(password) = password {
                pdf.set_password_protection(Some(password), None);
            }

            // Convert markdown to PDF
            let _page_id = pdf.add_a4_page()?;
            let markdown_text = md_doc.to_markdown();
            let mut y_position = 750.0;
            let line_height = 14.0;

            for line in markdown_text.lines() {
                if y_position < 100.0 {
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
                } else if !line.trim().is_empty() {
                    pdf.add_text(pdf.pages.len() - 1, line, 100.0, y_position, "Helvetica", 11.0)?;
                    y_position -= line_height;
                } else {
                    y_position -= line_height * 0.5;
                }
            }

            pdf.to_bytes().map_err(|e| e.into())
        },
        OutputFormat::Pptx => {
            let mut pptx = PptxDocument::new();

            if let Some(title) = title {
                pptx.set_title(title);
            }
            if let Some(author) = author {
                pptx.set_author(author);
            }

            // Convert markdown to slides
            let markdown_text = md_doc.to_markdown();
            let sections = split_markdown_into_sections(&markdown_text);

            for (i, section) in sections.iter().enumerate() {
                let slide_title = section.title.as_deref().unwrap_or(&format!("Slide {}", i + 1));
                let content: Vec<&str> = section.content.iter().map(|s| s.as_str()).collect();
                pptx.add_slide(slide_title, &content)?;
            }

            pptx.to_bytes().map_err(|e| e.into())
        },
        OutputFormat::Xlsx => {
            let mut xlsx = XlsxDocument::new();
            let data = vec![
                vec!["Content".to_string(), "Type".to_string()],
                vec!["Markdown Document".to_string(), "Converted".to_string()],
            ];
            xlsx.add_worksheet("Data", &data)?;
            xlsx.to_bytes().map_err(|e| e.into())
        },
        OutputFormat::Text => {
            Ok(md_doc.to_markdown().into_bytes())
        },
        OutputFormat::Markdown => {
            Ok(markdown.as_bytes().to_vec())
        },
        OutputFormat::Json => {
            let json_data = serde_json::json!({
                "type": "markdown",
                "content": markdown,
                "elements": md_doc.elements.len()
            });
            Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
        },
    }
}

fn convert_text(
    text: &str,
    output_format: &OutputFormat,
    title: Option<&str>,
    author: Option<&str>,
    password: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Convert text to markdown first, then convert to target format
    let markdown = text_to_markdown(text, title);
    convert_markdown(&markdown, output_format, title, author, password)
}

fn convert_from_docx(doc: &DocxDocument, output_format: &OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match output_format {
        OutputFormat::Docx => doc.to_bytes().map_err(|e| e.into()),
        OutputFormat::Text => Ok(doc.get_text().unwrap_or_default().into_bytes()),
        OutputFormat::Json => {
            let stats = doc.get_document_stats();
            let json_data = serde_json::json!({
                "type": "docx",
                "stats": {
                    "page_count": stats.page_count,
                    "image_count": stats.image_count,
                    "table_count": stats.table_count,
                    "has_vba": stats.has_vba
                },
                "text_content": doc.get_text().unwrap_or_default()
            });
            Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
        },
        _ => Err("Conversion from DOCX to this format not yet implemented".into()),
    }
}

fn convert_from_xlsx(doc: &XlsxDocument, output_format: &OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match output_format {
        OutputFormat::Xlsx => doc.to_bytes().map_err(|e| e.into()),
        OutputFormat::Text => {
            let mut text = String::new();
            for (sheet_name, _) in &doc.worksheets {
                text.push_str(&format!("Sheet: {}\\n", sheet_name));
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
            Ok(text.into_bytes())
        },
        OutputFormat::Json => {
            let mut worksheets = std::collections::HashMap::new();
            for (sheet_name, _) in &doc.worksheets {
                if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                    worksheets.insert(sheet_name.clone(), data);
                }
            }

            let json_data = serde_json::json!({
                "type": "xlsx",
                "worksheets": worksheets
            });
            Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
        },
        _ => Err("Conversion from XLSX to this format not yet implemented".into()),
    }
}

fn convert_from_pptx(doc: &PptxDocument, output_format: &OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match output_format {
        OutputFormat::Pptx => doc.to_bytes().map_err(|e| e.into()),
        OutputFormat::Text => Ok(doc.get_slide_text().unwrap_or_default().into_bytes()),
        OutputFormat::Json => {
            let slides = doc.get_all_slides().unwrap_or_default();
            let json_data = serde_json::json!({
                "type": "pptx",
                "slides": slides.len(),
                "content": doc.get_slide_text().unwrap_or_default()
            });
            Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
        },
        _ => Err("Conversion from PPTX to this format not yet implemented".into()),
    }
}

fn convert_from_pdf(doc: &PdfDocument, output_format: &OutputFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match output_format {
        OutputFormat::Pdf => doc.to_bytes().map_err(|e| e.into()),
        OutputFormat::Text => Ok(doc.get_text().unwrap_or_default().into_bytes()),
        OutputFormat::Json => {
            let stats = doc.get_document_stats();
            let json_data = serde_json::json!({
                "type": "pdf",
                "stats": {
                    "page_count": stats.page_count,
                    "image_count": stats.image_count,
                    "has_security": stats.has_security,
                    "has_javascript": stats.has_javascript
                },
                "text_content": doc.get_text().unwrap_or_default()
            });
            Ok(serde_json::to_string_pretty(&json_data)?.into_bytes())
        },
        _ => Err("Conversion from PDF to this format not yet implemented".into()),
    }
}

fn extract_content(input: &Path, extract_type: &ExtractType) -> Result<String, Box<dyn std::error::Error>> {
    let extension = input.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "docx" => {
            let doc = DocxDocument::from_file(input)?;
            match extract_type {
                ExtractType::Text => Ok(doc.get_text().unwrap_or_default()),
                ExtractType::Metadata => {
                    let stats = doc.get_document_stats();
                    Ok(format!("DOCX Document\\nPages: {}\\nImages: {}\\nTables: {}\\nVBA: {}\\nAuthor: {}",
                        stats.page_count, stats.image_count, stats.table_count, stats.has_vba, stats.author))
                },
                ExtractType::All => {
                    let mut result = String::new();
                    result.push_str("=== DOCX ANALYSIS ===\\n\\n");
                    result.push_str(&doc.get_text().unwrap_or_default());
                    Ok(result)
                },
            }
        },
        "xlsx" => {
            let doc = XlsxDocument::from_file(input)?;
            match extract_type {
                ExtractType::Text => {
                    let mut text = String::new();
                    for (sheet_name, _) in &doc.worksheets {
                        text.push_str(&format!("Sheet: {}\\n", sheet_name));
                        if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                            for row in data {
                                for cell in row {
                                    text.push_str(&cell);
                                    text.push('\\t');
                                }
                                text.push('\n');
                            }
                        }
                    }
                    Ok(text)
                },
                ExtractType::Metadata => {
                    Ok(format!("XLSX Document\\nWorksheets: {}\\nAuthor: {}",
                        doc.worksheets.len(), doc.author))
                },
                ExtractType::All => {
                    let mut result = String::new();
                    result.push_str("=== XLSX ANALYSIS ===\\n\\n");
                    result.push_str(&format!("Worksheets: {}\\n", doc.worksheets.len()));
                    Ok(result)
                },
            }
        },
        "pptx" => {
            let doc = PptxDocument::from_file(input)?;
            match extract_type {
                ExtractType::Text => Ok(doc.get_slide_text().unwrap_or_default()),
                ExtractType::Metadata => {
                    let stats = doc.get_document_stats();
                    Ok(format!("PPTX Presentation\\nSlides: {}\\nImages: {}\\nAuthor: {}",
                        stats.slide_count, stats.image_count, stats.author))
                },
                ExtractType::All => {
                    let mut result = String::new();
                    result.push_str("=== PPTX ANALYSIS ===\\n\\n");
                    result.push_str(&doc.get_slide_text().unwrap_or_default());
                    Ok(result)
                },
            }
        },
        "pdf" => {
            let doc = PdfDocument::from_file(input)?;
            match extract_type {
                ExtractType::Text => Ok(doc.get_text().unwrap_or_default()),
                ExtractType::Metadata => {
                    let stats = doc.get_document_stats();
                    Ok(format!("PDF Document\\nPages: {}\\nImages: {}\\nSecurity: {}\\nJavaScript: {}",
                        stats.page_count, stats.image_count, stats.has_security, stats.has_javascript))
                },
                ExtractType::All => {
                    let mut result = String::new();
                    result.push_str("=== PDF ANALYSIS ===\\n\\n");
                    let stats = doc.get_document_stats();
                    result.push_str(&format!("Pages: {}, Images: {}, Forms: {}\\n\\n",
                        stats.page_count, stats.image_count, stats.form_field_count));
                    result.push_str(&doc.get_text().unwrap_or_default());
                    Ok(result)
                },
            }
        },
        _ => Err(format!("Unsupported file format: {}", extension).into()),
    }
}

fn analyze_document(input: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let extension = input.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_size = fs::metadata(input)?.len();

    let mut analysis = String::new();
    analysis.push_str(&format!("📄 Document Analysis: {}\\n", input.display()));
    analysis.push_str(&format!("📊 File Size: {}\\n", format_size(file_size)));
    analysis.push_str(&format!("🔧 Format: {}\\n\\n", extension.to_uppercase()));

    match extension.as_str() {
        "docx" => {
            let doc = DocxDocument::from_file(input)?;
            let stats = doc.get_document_stats();
            analysis.push_str(&format!("Pages: {}\\n", stats.page_count));
            analysis.push_str(&format!("Images: {}\\n", stats.image_count));
            analysis.push_str(&format!("Tables: {}\\n", stats.table_count));
            analysis.push_str(&format!("VBA Macros: {}\\n", if stats.has_vba { "Present" } else { "None" }));
            analysis.push_str(&format!("Author: {}\\n", stats.author));
            analysis.push_str(&format!("Created: {}\\n", stats.created));
        },
        "xlsx" => {
            let doc = XlsxDocument::from_file(input)?;
            analysis.push_str(&format!("Worksheets: {}\\n", doc.worksheets.len()));
            analysis.push_str(&format!("Author: {}\\n", doc.author));
            analysis.push_str(&format!("Title: {}\\n", doc.title));

            let mut total_cells = 0;
            for (sheet_name, _) in &doc.worksheets {
                if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                    let cells = data.iter().map(|row| row.len()).sum::<usize>();
                    total_cells += cells;
                    analysis.push_str(&format!("  Sheet '{}': {} rows\\n", sheet_name, data.len()));
                }
            }
            analysis.push_str(&format!("Total Cells: {}\\n", total_cells));
        },
        "pptx" => {
            let doc = PptxDocument::from_file(input)?;
            let stats = doc.get_document_stats();
            analysis.push_str(&format!("Slides: {}\\n", stats.slide_count));
            analysis.push_str(&format!("Images: {}\\n", stats.image_count));
            analysis.push_str(&format!("Animations: {}\\n", stats.animation_count));
            analysis.push_str(&format!("Author: {}\\n", stats.author));

            if let Ok(slides) = doc.get_all_slides() {
                analysis.push_str("\\nSlide Titles:\\n");
                for (i, slide) in slides.iter().enumerate() {
                    analysis.push_str(&format!("  {}: {}\\n", i + 1, slide.title));
                }
            }
        },
        "pdf" => {
            let doc = PdfDocument::from_file(input)?;
            let stats = doc.get_document_stats();
            analysis.push_str(&format!("Pages: {}\\n", stats.page_count));
            analysis.push_str(&format!("Images: {}\\n", stats.image_count));
            analysis.push_str(&format!("Annotations: {}\\n", stats.annotation_count));
            analysis.push_str(&format!("Form Fields: {}\\n", stats.form_field_count));
            analysis.push_str(&format!("Security: {}\\n", if stats.has_security { "Enabled" } else { "None" }));
            analysis.push_str(&format!("JavaScript: {}\\n", if stats.has_javascript { "Present" } else { "None" }));
            analysis.push_str(&format!("PDF Version: {:?}\\n", stats.pdf_version));
            analysis.push_str(&format!("Author: {}\\n", doc.author));
            analysis.push_str(&format!("Title: {}\\n", doc.title));
        },
        _ => {
            return Err(format!("Unsupported file format for analysis: {}", extension).into());
        }
    }

    Ok(analysis)
}

fn text_to_markdown(text: &str, title: Option<&str>) -> String {
    let mut markdown = String::new();

    // Add title if provided
    if let Some(title) = title {
        markdown.push_str(&format!("# {}\\n\\n", title));
    }

    // Convert text to markdown with basic formatting
    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            markdown.push('\\n');
            continue;
        }

        // Detect potential headings
        if trimmed.len() < 80 && (trimmed.ends_with(':') || trimmed == trimmed.to_uppercase()) && trimmed.len() > 3 {
            let heading_text = trimmed.trim_end_matches(':');
            markdown.push_str(&format!("## {}\\n\\n", heading_text));
        }
        // Detect list items
        else if trimmed.starts_with("- ") || trimmed.starts_with("* ") ||
                (trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) && trimmed.contains(". ")) {
            markdown.push_str(&format!("{}\\n", line));
        }
        // Regular text
        else {
            markdown.push_str(&format!("{}\\n\\n", trimmed));
        }
    }

    markdown
}

fn split_markdown_into_sections(markdown: &str) -> Vec<MarkdownSection> {
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

#[derive(Debug, Clone)]
struct MarkdownSection {
    title: Option<String>,
    content: Vec<String>,
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}