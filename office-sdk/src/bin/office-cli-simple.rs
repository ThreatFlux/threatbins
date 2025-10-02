use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use office_sdk::OfficeDocument;
use office_sdk::docx::DocxDocument;
use office_sdk::eml::EmlDocument;
use office_sdk::markdown::MarkdownDocument;
use office_sdk::pdf::PdfDocument;
use office_sdk::pptx::PptxDocument;
use office_sdk::xlsx::XlsxDocument;
use std::fs;
use std::path::{Path, PathBuf};

/// Office SDK CLI - Document creation and conversion tool
#[derive(Parser)]
#[command(name = "office-cli")]
#[command(about = "Create and convert Office documents from Markdown, text, and other sources")]
#[command(version = "1.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Force overwrite existing files
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert files or text to Office formats
    Convert {
        /// Input file path (optional if using --text)
        input: Option<PathBuf>,

        /// Direct text input (use quotes for multi-word text)
        #[arg(short, long)]
        text: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Document title
        #[arg(long)]
        title: Option<String>,

        /// Document author
        #[arg(long)]
        author: Option<String>,
    },

    /// Create a document from direct text input
    Create {
        /// Text content for the document
        text: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "pdf")]
        format: OutputFormat,

        /// Document title
        #[arg(long)]
        title: Option<String>,

        /// Document author
        #[arg(long)]
        author: Option<String>,
    },

    /// Extract text from documents
    Extract {
        /// Input document
        input: PathBuf,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show document information
    Info {
        /// Input document
        input: PathBuf,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Docx,
    Xlsx,
    Pptx,
    Pdf,
    Eml,
    Text,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("{}", "🏢 Office SDK CLI v1.0.0".bright_blue().bold());
    println!("{}", "Document creation and conversion tool".bright_white());
    println!();

    match cli.command {
        Commands::Convert {
            input,
            text,
            output,
            format,
            title,
            author,
        } => {
            handle_convert(
                input,
                text,
                output,
                format,
                title,
                author,
                cli.verbose,
                cli.force,
            )?;
        }
        Commands::Create {
            text,
            output,
            format,
            title,
            author,
        } => {
            handle_create(text, output, format, title, author, cli.verbose, cli.force)?;
        }
        Commands::Extract { input, output } => {
            handle_extract(input, output)?;
        }
        Commands::Info { input } => {
            handle_info(input)?;
        }
    }

    println!("\n{}", "✨ Operation completed!".bright_green().bold());
    Ok(())
}

fn handle_convert(
    input: Option<PathBuf>,
    text: Option<String>,
    output: Option<PathBuf>,
    format: Option<OutputFormat>,
    title: Option<String>,
    author: Option<String>,
    verbose: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input - need either file or text
    let (input_content, is_markdown, input_file_path) = match (&input, &text) {
        (Some(file_path), None) => {
            if !file_path.exists() {
                return Err(format!("Input file does not exist: {}", file_path.display()).into());
            }
            let content = fs::read_to_string(file_path)?;
            let is_md = file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase()
                == "md";
            (content, is_md, Some(file_path.clone()))
        }
        (None, Some(text_content)) => {
            (text_content.clone(), false, None) // Direct text input
        }
        (Some(_), Some(_)) => {
            return Err(
                "Cannot specify both input file and direct text. Use one or the other.".into(),
            );
        }
        (None, None) => {
            return Err("Must specify either input file or direct text with --text option.".into());
        }
    };

    // Determine output format and path
    let output_format = format.unwrap_or(OutputFormat::Pdf);
    let output_path = output.unwrap_or_else(|| {
        let stem = if let Some(ref file_path) = input_file_path {
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output")
        } else {
            title.as_deref().unwrap_or("document")
        };

        let ext = match output_format {
            OutputFormat::Docx => "docx",
            OutputFormat::Xlsx => "xlsx",
            OutputFormat::Pptx => "pptx",
            OutputFormat::Pdf => "pdf",
            OutputFormat::Eml => "eml",
            OutputFormat::Text => "txt",
        };

        PathBuf::from(format!("{}.{}", stem, ext))
    });

    if output_path.exists() && !force {
        return Err("Output file exists. Use --force to overwrite.".into());
    }

    if verbose {
        if let Some(ref file_path) = input_file_path {
            println!("📁 Input file: {}", file_path.display());
        } else {
            println!("📝 Direct text input: {} characters", input_content.len());
        }
        println!("📄 Output: {}", output_path.display());
        println!("🔄 Format: {:?}", output_format);
    }

    // Show progress
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos:>3}% {msg}")
            .unwrap(),
    );

    pb.set_position(50);
    pb.set_message("Converting...");

    // Convert based on input type
    let output_data = if is_markdown {
        convert_markdown(
            &input_content,
            &output_format,
            title.as_deref(),
            author.as_deref(),
        )?
    } else {
        convert_text(
            &input_content,
            &output_format,
            title.as_deref(),
            author.as_deref(),
        )?
    };

    pb.set_position(80);
    pb.set_message("Writing output...");

    // Write output
    fs::write(&output_path, output_data)?;

    pb.set_position(100);
    pb.finish_with_message("Conversion complete!");

    println!(
        "{} Converted to {}",
        "✅".green(),
        output_path.display().to_string().bright_white()
    );

    Ok(())
}

fn handle_create(
    text: String,
    output: Option<PathBuf>,
    format: OutputFormat,
    title: Option<String>,
    author: Option<String>,
    verbose: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let stem = title.as_deref().unwrap_or("document");
        let ext = match format {
            OutputFormat::Docx => "docx",
            OutputFormat::Xlsx => "xlsx",
            OutputFormat::Pptx => "pptx",
            OutputFormat::Pdf => "pdf",
            OutputFormat::Eml => "eml",
            OutputFormat::Text => "txt",
        };
        PathBuf::from(format!("{}.{}", stem.replace(" ", "_").to_lowercase(), ext))
    });

    if output_path.exists() && !force {
        return Err("Output file exists. Use --force to overwrite.".into());
    }

    if verbose {
        println!("📝 Text input: {} characters", text.len());
        println!("📄 Output: {}", output_path.display());
        println!("🔄 Format: {:?}", format);
    }

    // Show progress
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos:>3}% {msg}")
            .unwrap(),
    );

    pb.set_position(20);
    pb.set_message("Processing text...");

    // Convert text to target format
    let output_data = convert_text(&text, &format, title.as_deref(), author.as_deref())?;

    pb.set_position(80);
    pb.set_message("Writing output...");

    // Write output
    fs::write(&output_path, output_data)?;

    pb.set_position(100);
    pb.finish_with_message("Document created!");

    println!(
        "{} Created {} document: {}",
        "✅".green(),
        format!("{:?}", format).bright_yellow(),
        output_path.display().to_string().bright_white()
    );

    Ok(())
}

fn handle_extract(
    input: PathBuf,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let extension = input
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let extracted_text = match extension.as_str() {
        "docx" => {
            let doc = DocxDocument::from_file(&input)?;
            doc.get_text().unwrap_or_default()
        }
        "xlsx" => {
            let doc = XlsxDocument::from_file(&input)?;
            let mut text = String::new();
            for (sheet_name, _) in &doc.worksheets {
                text.push_str(&format!("Sheet: {}\n", sheet_name));
                if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                    for row in data {
                        text.push_str(&row.join("\t"));
                        text.push('\n');
                    }
                }
                text.push('\n');
            }
            text
        }
        "pptx" => {
            let doc = PptxDocument::from_file(&input)?;
            doc.get_slide_text().unwrap_or_default()
        }
        "pdf" => {
            let doc = PdfDocument::from_file(&input)?;
            doc.get_text().unwrap_or_default()
        }
        "eml" => {
            let doc = EmlDocument::from_file(&input)?;
            doc.get_text().unwrap_or_default()
        }
        _ => return Err(format!("Unsupported format for extraction: {}", extension).into()),
    };

    if let Some(output_path) = output {
        fs::write(&output_path, &extracted_text)?;
        println!(
            "{} Text extracted to {}",
            "✅".green(),
            output_path.display()
        );
    } else {
        println!("{}", extracted_text);
    }

    Ok(())
}

fn handle_info(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let extension = input
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_size = fs::metadata(&input)?.len();

    println!("📄 Document: {}", input.display());
    println!("📊 Size: {}", format_size(file_size));
    println!("🔧 Format: {}", extension.to_uppercase());

    match extension.as_str() {
        "docx" => {
            let doc = DocxDocument::from_file(&input)?;
            println!("📝 Type: Word Document");
            if let Ok(text) = doc.get_text() {
                println!("📏 Text Length: {} characters", text.len());
                println!("📖 Word Count: {}", text.split_whitespace().count());
            }
        }
        "xlsx" => {
            let doc = XlsxDocument::from_file(&input)?;
            println!("📝 Type: Excel Spreadsheet");
            println!("📋 Worksheets: {}", doc.worksheets.len());
            println!("👤 Author: {}", doc.author);

            let mut total_cells = 0;
            for (sheet_name, _) in &doc.worksheets {
                if let Ok(data) = doc.get_worksheet_data(sheet_name) {
                    let cells = data.iter().map(|row| row.len()).sum::<usize>();
                    total_cells += cells;
                    println!(
                        "  📊 Sheet '{}': {} rows, {} columns",
                        sheet_name,
                        data.len(),
                        data.first().map(|row| row.len()).unwrap_or(0)
                    );
                }
            }
            println!("🔢 Total Cells: {}", total_cells);
        }
        "pptx" => {
            let doc = PptxDocument::from_file(&input)?;
            println!("📝 Type: PowerPoint Presentation");
            if let Ok(slides) = doc.get_all_slides() {
                println!("🎯 Slides: {}", slides.len());
                for (i, slide) in slides.iter().enumerate() {
                    println!("  📄 Slide {}: {}", i + 1, slide.title);
                }
            }
        }
        "pdf" => {
            let doc = PdfDocument::from_file(&input)?;
            let stats = doc.get_document_stats();
            println!("📝 Type: PDF Document");
            println!("📄 Pages: {}", stats.page_count);
            println!("🖼️ Images: {}", stats.image_count);
            println!("📝 Annotations: {}", stats.annotation_count);
            println!("📋 Form Fields: {}", stats.form_field_count);
            println!(
                "🔒 Security: {}",
                if stats.has_security {
                    "Enabled"
                } else {
                    "None"
                }
            );
            println!(
                "⚡ JavaScript: {}",
                if stats.has_javascript {
                    "Present"
                } else {
                    "None"
                }
            );
            println!("👤 Author: {}", doc.author);
            println!("🏷️ Title: {}", doc.title);
        }
        "eml" => {
            let doc = EmlDocument::from_file(input)?;
            let stats = doc.get_document_stats();
            println!("📝 Type: Email Message");
            println!("📧 From: {}", stats.sender);
            println!("📋 Subject: {}", stats.subject);
            println!("👥 Recipients: {}", stats.recipient_count);
            println!("📎 Attachments: {}", stats.attachment_count);
            if stats.attachment_count > 0 {
                println!(
                    "📊 Attachment Size: {}",
                    format_size(stats.attachment_total_size as u64)
                );
            }
            println!("📝 Text Length: {} characters", stats.text_length);
            if stats.has_html_content {
                println!("🌐 HTML Content: {} characters", stats.html_length);
            }
            println!(
                "🔒 Encryption: {}",
                if stats.has_encryption { "Yes" } else { "None" }
            );
            println!(
                "✍️ Digital Signature: {}",
                if stats.has_digital_signature {
                    "Yes"
                } else {
                    "None"
                }
            );
            println!(
                "📊 Tracking: {}",
                if stats.has_tracking {
                    "Enabled"
                } else {
                    "None"
                }
            );
            println!(
                "📅 Date: {}",
                stats.date_sent.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!("⚡ Priority: {:?}", stats.priority);
        }
        _ => {
            println!("📝 Type: Unknown");
        }
    }

    Ok(())
}

// Conversion implementations
fn convert_markdown(
    markdown: &str,
    output_format: &OutputFormat,
    title: Option<&str>,
    author: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let md_doc = MarkdownDocument::from_markdown(markdown)?;

    match output_format {
        OutputFormat::Docx => {
            let docx = md_doc.to_docx()?;
            docx.to_bytes().map_err(|e| e.into())
        }
        OutputFormat::Pdf => {
            let mut pdf = PdfDocument::new();

            if let Some(title) = title {
                pdf.set_title(title);
            }
            if let Some(author) = author {
                pdf.set_author(author);
            }

            // Add content
            let _page_id = pdf.add_a4_page()?;
            let mut y_position = 750.0;

            for line in markdown.lines() {
                if y_position < 100.0 {
                    let _new_page = pdf.add_a4_page()?;
                    y_position = 750.0;
                }

                if line.starts_with("# ") {
                    let text = line.trim_start_matches("# ");
                    pdf.add_heading(pdf.pages.len() - 1, 1, text, 100.0, y_position)?;
                    y_position -= 30.0;
                } else if line.starts_with("## ") {
                    let text = line.trim_start_matches("## ");
                    pdf.add_heading(pdf.pages.len() - 1, 2, text, 100.0, y_position)?;
                    y_position -= 25.0;
                } else if !line.trim().is_empty() {
                    pdf.add_text(
                        pdf.pages.len() - 1,
                        line,
                        100.0,
                        y_position,
                        "Helvetica",
                        11.0,
                    )?;
                    y_position -= 15.0;
                } else {
                    y_position -= 8.0;
                }
            }

            pdf.to_bytes().map_err(|e| e.into())
        }
        OutputFormat::Pptx => {
            let mut pptx = PptxDocument::new();

            if let Some(title) = title {
                pptx.set_title(title);
            }
            if let Some(author) = author {
                pptx.set_author(author);
            }

            // Split into slides based on headings
            let mut current_title = "Slide 1".to_string();
            let mut current_content = Vec::new();
            let mut slide_count = 0;

            for line in markdown.lines() {
                if line.starts_with("# ") {
                    // Create slide with previous content
                    if slide_count > 0 {
                        let content_refs: Vec<&str> = current_content
                            .iter()
                            .map(|s: &String| s.as_str())
                            .collect();
                        pptx.add_slide(&current_title, &content_refs)?;
                    }

                    current_title = line.trim_start_matches("# ").to_string();
                    current_content.clear();
                    slide_count += 1;
                } else if line.starts_with("## ") {
                    current_content.push(line.trim_start_matches("## ").to_string());
                } else if !line.trim().is_empty() {
                    current_content.push(line.to_string());
                }
            }

            // Add final slide
            if slide_count > 0 {
                let content_refs: Vec<&str> = current_content
                    .iter()
                    .map(|s: &String| s.as_str())
                    .collect();
                pptx.add_slide(&current_title, &content_refs)?;
            }

            pptx.to_bytes().map_err(|e| e.into())
        }
        OutputFormat::Xlsx => {
            let mut xlsx = XlsxDocument::new();
            let data = vec![vec!["Line".to_string(), "Content".to_string()]];
            let mut sheet_data = data;

            for (i, line) in markdown.lines().enumerate() {
                if !line.trim().is_empty() {
                    sheet_data.push(vec![(i + 1).to_string(), line.to_string()]);
                }
            }

            xlsx.add_worksheet("Markdown Content", &sheet_data)?;
            xlsx.to_bytes().map_err(|e| e.into())
        }
        OutputFormat::Eml => {
            let mut eml = EmlDocument::new();

            if let Some(title) = title {
                eml.set_title(title);
            }
            if let Some(author) = author {
                eml.set_from(author, None)?;
            }

            // Convert markdown to email content
            eml.set_markdown_body(markdown);

            // Add default recipient if none specified
            eml.add_to("recipient@example.com", Some("Recipient"))?;

            eml.to_bytes().map_err(|e| e.into())
        }
        OutputFormat::Text => Ok(markdown.as_bytes().to_vec()),
    }
}

fn convert_text(
    text: &str,
    output_format: &OutputFormat,
    title: Option<&str>,
    author: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Simple text to markdown conversion
    let mut markdown = String::new();

    if let Some(title) = title {
        markdown.push_str(&format!("# {}\n\n", title));
    }

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            markdown.push_str(&format!("{}\n", trimmed));
        } else {
            markdown.push('\n');
        }
    }

    convert_markdown(&markdown, output_format, title, author)
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{:.1} {}", size, UNITS[unit])
}
