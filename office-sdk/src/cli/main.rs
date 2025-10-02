use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use console::Term;
use dialoguer::{Confirm, Input, Select, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use office_sdk::OfficeDocument;
use office_sdk::docx::DocxDocument;
use office_sdk::xlsx::XlsxDocument;
use office_sdk::pptx::PptxDocument;
use office_sdk::pdf::PdfDocument;
use office_sdk::markdown::MarkdownDocument;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod converter;
mod templates;
mod config;
mod utils;

use converter::*;
use templates::*;
use config::*;
use utils::*;

/// Office SDK CLI - Professional document creation and conversion tool
#[derive(Parser)]
#[command(name = "office-cli")]
#[command(about = "A comprehensive CLI for creating and converting Office documents (DOCX, XLSX, PPTX, PDF) from Markdown, text, and other sources")]
#[command(version = "1.0.0")]
#[command(author = "Office SDK Team")]
#[command(long_about = "
Office CLI is a powerful command-line tool for creating professional Office documents
from various input sources. Convert Markdown to DOCX, create presentations from text,
generate PDFs with advanced features, and much more.

Features:
• Convert Markdown → DOCX, XLSX, PPTX, PDF
• Create documents from text files and images
• Professional templates for business documents
• Batch processing for multiple files
• Interactive document creation wizard
• Advanced customization and styling options
• Security features (password protection, encryption)
• Metadata management and document properties

Examples:
  office-cli convert document.md --output report.docx --template business
  office-cli create presentation --input slides.txt --output demo.pptx
  office-cli batch-convert *.md --format pdf --output-dir ./pdfs/
  office-cli wizard --interactive
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

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

        /// Document template to use
        #[arg(short, long)]
        template: Option<String>,

        /// Document title
        #[arg(long)]
        title: Option<String>,

        /// Document author
        #[arg(long)]
        author: Option<String>,

        /// Document subject
        #[arg(long)]
        subject: Option<String>,

        /// Include table of contents
        #[arg(long)]
        toc: bool,

        /// Add page numbers
        #[arg(long)]
        page_numbers: bool,

        /// Password protect the document
        #[arg(long)]
        password: Option<String>,
    },

    /// Create new documents from various inputs
    Create {
        /// Document type to create
        #[arg(value_enum)]
        doc_type: DocumentType,

        /// Input source (file, directory, or stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Document template
        #[arg(short, long)]
        template: Option<String>,

        /// Additional images to include
        #[arg(long)]
        images: Vec<PathBuf>,

        /// Document title
        #[arg(long)]
        title: Option<String>,

        /// Document author
        #[arg(long)]
        author: Option<String>,
    },

    /// Batch convert multiple files
    BatchConvert {
        /// Input pattern or directory
        #[arg(value_name = "PATTERN")]
        pattern: String,

        /// Output format
        #[arg(short, long, value_enum)]
        format: OutputFormat,

        /// Output directory
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Template to apply to all documents
        #[arg(short, long)]
        template: Option<String>,

        /// Process subdirectories recursively
        #[arg(short, long)]
        recursive: bool,

        /// Number of parallel workers
        #[arg(short = 'j', long, default_value = "4")]
        workers: usize,
    },

    /// Interactive document creation wizard
    Wizard {
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,

        /// Start with a specific template
        #[arg(short, long)]
        template: Option<String>,
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

        /// Extract from specific pages (e.g., "1,3-5,7")
        #[arg(long)]
        pages: Option<String>,
    },

    /// Analyze and inspect documents
    Analyze {
        /// Input document
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Analysis type
        #[arg(short, long, value_enum, default_value = "overview")]
        analysis_type: AnalysisType,

        /// Output format for analysis
        #[arg(long, value_enum, default_value = "human")]
        output_format: AnalysisOutputFormat,

        /// Save analysis to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Merge multiple documents
    Merge {
        /// Input files to merge
        #[arg(value_name = "INPUTS", required = true)]
        inputs: Vec<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// How to handle different formats
        #[arg(long, value_enum, default_value = "convert")]
        format_strategy: MergeStrategy,
    },

    /// Template management
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
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
    /// HTML document
    Html,
    /// JSON metadata
    Json,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DocumentType {
    /// Word document
    Document,
    /// Spreadsheet
    Spreadsheet,
    /// Presentation
    Presentation,
    /// PDF document
    Pdf,
    /// Report (auto-format selection)
    Report,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ExtractType {
    /// Extract all text content
    Text,
    /// Extract images
    Images,
    /// Extract metadata
    Metadata,
    /// Extract form data
    Forms,
    /// Extract annotations
    Annotations,
    /// Extract table data
    Tables,
    /// Extract everything
    All,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum AnalysisType {
    /// Basic document overview
    Overview,
    /// Detailed structure analysis
    Structure,
    /// Security analysis
    Security,
    /// Performance metrics
    Performance,
    /// Accessibility compliance
    Accessibility,
    /// Content statistics
    Statistics,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum AnalysisOutputFormat {
    /// Human-readable format
    Human,
    /// JSON structured data
    Json,
    /// CSV for spreadsheet analysis
    Csv,
    /// Markdown report
    Markdown,
    /// HTML report
    Html,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum MergeStrategy {
    /// Convert all to common format
    Convert,
    /// Preserve individual formats
    Preserve,
    /// Create multi-format package
    Package,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// List available templates
    List,
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
    /// Create new template
    Create {
        /// Template name
        name: String,
        /// Base template to copy from
        #[arg(long)]
        from: Option<String>,
    },
    /// Edit existing template
    Edit {
        /// Template name
        name: String,
    },
    /// Delete template
    Delete {
        /// Template name
        name: String,
    },
    /// Install template from file
    Install {
        /// Template file path
        path: PathBuf,
    },
    /// Export template to file
    Export {
        /// Template name
        name: String,
        /// Output path
        output: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Reset configuration to defaults
    Reset,
    /// Edit configuration interactively
    Edit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize console and colors
    let term = Term::stdout();

    // Load configuration
    let config = Config::load(cli.config.as_ref())?;

    // Set up logging level
    let verbose = cli.verbose && !cli.quiet;
    let quiet = cli.quiet;

    if !quiet {
        println!("{}", "🏢 Office SDK CLI v1.0.0".bright_blue().bold());
        println!("{}", "Professional document creation and conversion tool".bright_white());
        println!();
    }

    match cli.command {
        Commands::Convert {
            input,
            output,
            format,
            template,
            title,
            author,
            subject,
            toc,
            page_numbers,
            password
        } => {
            handle_convert(
                input, output, format, template, title, author, subject,
                toc, page_numbers, password, &config, verbose, quiet, cli.force
            ).await?;
        },

        Commands::Create {
            doc_type,
            input,
            output,
            template,
            images,
            title,
            author
        } => {
            handle_create(
                doc_type, input, output, template, images, title, author,
                &config, verbose, quiet, cli.force
            ).await?;
        },

        Commands::BatchConvert {
            pattern,
            format,
            output_dir,
            template,
            recursive,
            workers
        } => {
            handle_batch_convert(
                pattern, format, output_dir, template, recursive, workers,
                &config, verbose, quiet, cli.force
            ).await?;
        },

        Commands::Wizard { yes, template } => {
            handle_wizard(yes, template, &config, verbose).await?;
        },

        Commands::Extract {
            input,
            extract_type,
            output,
            pages
        } => {
            handle_extract(input, extract_type, output, pages, verbose, quiet).await?;
        },

        Commands::Analyze {
            input,
            analysis_type,
            output_format,
            output
        } => {
            handle_analyze(input, analysis_type, output_format, output, verbose, quiet).await?;
        },

        Commands::Merge {
            inputs,
            output,
            format_strategy
        } => {
            handle_merge(inputs, output, format_strategy, &config, verbose, quiet, cli.force).await?;
        },

        Commands::Template { action } => {
            handle_template(action, &config, verbose, quiet).await?;
        },

        Commands::Config { action } => {
            handle_config(action, &config, verbose, quiet).await?;
        },
    }

    if !quiet {
        println!("\n{}", "✨ Operation completed successfully!".bright_green().bold());
    }

    Ok(())
}

// Command handlers
async fn handle_convert(
    input: PathBuf,
    output: Option<PathBuf>,
    format: Option<OutputFormat>,
    template: Option<String>,
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    toc: bool,
    page_numbers: bool,
    password: Option<String>,
    config: &Config,
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
        if quiet {
            return Err("Output file exists. Use --force to overwrite.".into());
        }

        let overwrite = Confirm::new()
            .with_prompt(format!("Output file '{}' already exists. Overwrite?", output_path.display()))
            .interact()?;

        if !overwrite {
            println!("{}", "Conversion cancelled.".yellow());
            return Ok(());
        }
    }

    if verbose {
        println!("📁 Input: {}", input.display());
        println!("📄 Output: {}", output_path.display());
        println!("🔄 Format: {:?}", output_format);
        if let Some(ref tpl) = template {
            println!("📋 Template: {}", tpl);
        }
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
    if let Some(ref pb) = pb { pb.set_position(10); pb.set_message("Reading input file..."); }

    let input_content = fs::read_to_string(&input)?;
    let input_extension = input.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if let Some(ref pb) = pb { pb.set_position(30); pb.set_message("Parsing document..."); }

    // Create converter based on input type
    let mut converter = DocumentConverter::new(config, template.as_deref())?;

    // Set metadata if provided
    if let Some(title) = title {
        converter.set_title(&title);
    }
    if let Some(author) = author {
        converter.set_author(&author);
    }
    if let Some(subject) = subject {
        converter.set_subject(&subject);
    }

    // Configure options
    converter.set_option("toc", toc);
    converter.set_option("page_numbers", page_numbers);
    if let Some(pwd) = password {
        converter.set_option("password", pwd);
    }

    if let Some(ref pb) = pb { pb.set_position(50); pb.set_message("Converting document..."); }

    // Perform conversion
    let output_data = match input_extension.as_str() {
        "md" | "markdown" => {
            converter.convert_markdown(&input_content, &output_format)?
        },
        "txt" | "text" => {
            converter.convert_text(&input_content, &output_format)?
        },
        "docx" => {
            let doc = DocxDocument::from_file(&input)?;
            converter.convert_from_docx(&doc, &output_format)?
        },
        "xlsx" => {
            let doc = XlsxDocument::from_file(&input)?;
            converter.convert_from_xlsx(&doc, &output_format)?
        },
        "pptx" => {
            let doc = PptxDocument::from_file(&input)?;
            converter.convert_from_pptx(&doc, &output_format)?
        },
        "pdf" => {
            let doc = PdfDocument::from_file(&input)?;
            converter.convert_from_pdf(&doc, &output_format)?
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

async fn handle_create(
    doc_type: DocumentType,
    input: Option<PathBuf>,
    output: PathBuf,
    template: Option<String>,
    images: Vec<PathBuf>,
    title: Option<String>,
    author: Option<String>,
    config: &Config,
    verbose: bool,
    quiet: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Creating {:?} document...", "🚀".bright_yellow(), doc_type);
    }

    let mut creator = DocumentCreator::new(config, template.as_deref())?;

    // Set metadata
    if let Some(title) = title {
        creator.set_title(&title);
    }
    if let Some(author) = author {
        creator.set_author(&author);
    }

    // Add content from input
    if let Some(input_path) = input {
        if input_path.is_file() {
            let content = fs::read_to_string(&input_path)?;
            creator.add_text_content(&content)?;
        } else if input_path.is_dir() {
            creator.add_directory_content(&input_path)?;
        }
    }

    // Add images
    for image_path in images {
        creator.add_image(&image_path)?;
    }

    // Create document
    let document_data = creator.create_document(&doc_type)?;

    // Write output
    fs::write(&output, document_data)?;

    if !quiet {
        let size = fs::metadata(&output)?.len();
        println!("{} Created {} ({})",
            "✅".green(),
            output.display().to_string().bright_white(),
            format_size(size).bright_cyan()
        );
    }

    Ok(())
}

async fn handle_batch_convert(
    pattern: String,
    format: OutputFormat,
    output_dir: PathBuf,
    template: Option<String>,
    recursive: bool,
    workers: usize,
    config: &Config,
    verbose: bool,
    quiet: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Starting batch conversion...", "⚡".bright_yellow());
        println!("🎯 Target format: {:?}", format);
        println!("📁 Output directory: {}", output_dir.display());
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    // Find input files
    let input_files = find_input_files(&pattern, recursive)?;

    if input_files.is_empty() {
        if !quiet {
            println!("{} No files found matching pattern: {}", "⚠️".yellow(), pattern);
        }
        return Ok(());
    }

    if !quiet {
        println!("📋 Found {} files to convert", input_files.len());
    }

    // Create progress bar
    let pb = if !quiet {
        let pb = ProgressBar::new(input_files.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}/{len:3} Converting: {msg}")
            .unwrap()
            .progress_chars("#>-"));
        Some(pb)
    } else {
        None
    };

    let mut converter = DocumentConverter::new(config, template.as_deref())?;
    let mut success_count = 0;
    let mut error_count = 0;

    // Process files
    for input_file in input_files {
        if let Some(ref pb) = pb {
            pb.set_message(input_file.file_name().unwrap().to_string_lossy().to_string());
        }

        // Determine output filename
        let output_file = determine_batch_output(&input_file, &output_dir, &format)?;

        // Convert file
        match convert_single_file(&mut converter, &input_file, &output_file, &format, force).await {
            Ok(_) => {
                success_count += 1;
                if verbose {
                    println!("✅ {}", input_file.display());
                }
            },
            Err(e) => {
                error_count += 1;
                if !quiet {
                    println!("❌ {}: {}", input_file.display(), e);
                }
            }
        }

        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    if !quiet {
        println!("\n{} Batch conversion completed!", "🎉".bright_green());
        println!("✅ Successful: {}", success_count.to_string().bright_green());
        if error_count > 0 {
            println!("❌ Failed: {}", error_count.to_string().bright_red());
        }
    }

    Ok(())
}

async fn handle_wizard(
    yes: bool,
    template: Option<String>,
    config: &Config,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🧙 Office Document Creation Wizard".bright_magenta().bold());
    println!("{}", "Let's create a professional document step by step!".bright_white());
    println!();

    // Step 1: Choose document type
    let doc_types = vec!["Word Document (DOCX)", "Excel Spreadsheet (XLSX)", "PowerPoint Presentation (PPTX)", "PDF Document", "Auto-detect from content"];
    let doc_type_selection = Select::new()
        .with_prompt("What type of document would you like to create?")
        .items(&doc_types)
        .default(0)
        .interact()?;

    let doc_type = match doc_type_selection {
        0 => DocumentType::Document,
        1 => DocumentType::Spreadsheet,
        2 => DocumentType::Presentation,
        3 => DocumentType::Pdf,
        4 => DocumentType::Report,
        _ => DocumentType::Document,
    };

    // Step 2: Choose input source
    let input_sources = vec!["Markdown file", "Text file", "Multiple text files", "Directory of content", "Start from scratch"];
    let input_selection = Select::new()
        .with_prompt("What's your content source?")
        .items(&input_sources)
        .default(0)
        .interact()?;

    let input_path = match input_selection {
        0..=3 => {
            Some(Input::<String>::new()
                .with_prompt("Enter the file/directory path")
                .interact_text()?
                .into())
        },
        _ => None,
    };

    // Step 3: Document metadata
    let title: String = Input::new()
        .with_prompt("Document title")
        .default("New Document".to_string())
        .interact_text()?;

    let author: String = Input::new()
        .with_prompt("Author name")
        .default(config.default_author.clone().unwrap_or_else(|| "Office SDK User".to_string()))
        .interact_text()?;

    let subject: String = Input::new()
        .with_prompt("Subject (optional)")
        .allow_empty(true)
        .interact_text()?;

    // Step 4: Choose template
    let available_templates = get_available_templates(config)?;
    let template_names: Vec<String> = available_templates.keys().cloned().collect();

    let template_selection = if !template_names.is_empty() {
        let mut items = vec!["No template (basic formatting)".to_string()];
        items.extend(template_names);

        let selection = Select::new()
            .with_prompt("Choose a template")
            .items(&items)
            .default(0)
            .interact()?;

        if selection > 0 {
            Some(items[selection].clone())
        } else {
            None
        }
    } else {
        None
    };

    // Step 5: Advanced options
    let advanced_options = MultiSelect::new()
        .with_prompt("Select additional features (use Space to select, Enter to confirm)")
        .items(&vec![
            "Table of contents",
            "Page numbers",
            "Password protection",
            "Advanced formatting",
            "Include images",
            "Interactive forms (PDF only)",
            "Document security"
        ])
        .interact()?;

    let toc = advanced_options.contains(&0);
    let page_numbers = advanced_options.contains(&1);
    let password_protect = advanced_options.contains(&2);
    let advanced_formatting = advanced_options.contains(&3);
    let include_images = advanced_options.contains(&4);
    let interactive_forms = advanced_options.contains(&5);
    let document_security = advanced_options.contains(&6);

    // Step 6: Output location
    let output: String = Input::new()
        .with_prompt("Output file path")
        .default(format!("{}.{}", title.replace(" ", "_").to_lowercase(),
            match doc_type {
                DocumentType::Document => "docx",
                DocumentType::Spreadsheet => "xlsx",
                DocumentType::Presentation => "pptx",
                DocumentType::Pdf => "pdf",
                DocumentType::Report => "pdf",
            }))
        .interact_text()?;

    let output_path = PathBuf::from(output);

    // Step 7: Confirmation
    if !yes {
        println!("\n{}", "📋 Document Creation Summary:".bright_cyan().bold());
        println!("   Type: {:?}", doc_type);
        println!("   Title: {}", title);
        println!("   Author: {}", author);
        if !subject.is_empty() {
            println!("   Subject: {}", subject);
        }
        if let Some(ref tpl) = template_selection {
            println!("   Template: {}", tpl);
        }
        println!("   Output: {}", output_path.display());
        println!("   Features: {}", if advanced_options.is_empty() { "Basic".to_string() } else { format!("{} advanced features", advanced_options.len()) });

        let confirm = Confirm::new()
            .with_prompt("Create this document?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("{}", "Document creation cancelled.".yellow());
            return Ok(());
        }
    }

    // Step 8: Create document
    println!("\n{} Creating document...", "⚡".bright_yellow());

    let mut creator = DocumentCreator::new(config, template_selection.as_deref())?;
    creator.set_title(&title);
    creator.set_author(&author);
    if !subject.is_empty() {
        creator.set_subject(&subject);
    }

    // Configure advanced options
    creator.set_option("toc", toc);
    creator.set_option("page_numbers", page_numbers);
    creator.set_option("advanced_formatting", advanced_formatting);
    creator.set_option("include_images", include_images);
    creator.set_option("interactive_forms", interactive_forms);
    creator.set_option("document_security", document_security);

    if password_protect {
        let password: String = Input::new()
            .with_prompt("Enter password for document protection")
            .interact_text()?;
        creator.set_option("password", password);
    }

    // Add content from input
    if let Some(input_path) = input_path {
        let path = PathBuf::from(input_path);
        if path.exists() {
            if path.is_file() {
                let content = fs::read_to_string(&path)?;
                creator.add_text_content(&content)?;
            } else if path.is_dir() {
                creator.add_directory_content(&path)?;
            }
        }
    }

    // Create document
    let document_data = creator.create_document(&doc_type)?;
    fs::write(&output_path, document_data)?;

    let size = fs::metadata(&output_path)?.len();
    println!("{} Document created successfully!", "🎉".bright_green());
    println!("📄 File: {}", output_path.display().to_string().bright_white());
    println!("📊 Size: {}", format_size(size).bright_cyan());

    Ok(())
}

async fn handle_extract(
    input: PathBuf,
    extract_type: ExtractType,
    output: Option<PathBuf>,
    pages: Option<String>,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Extracting {:?} from {}...", "🔍".bright_blue(), extract_type, input.display());
    }

    let extractor = ContentExtractor::new();
    let extracted_data = extractor.extract(&input, &extract_type, pages.as_deref())?;

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

async fn handle_analyze(
    input: PathBuf,
    analysis_type: AnalysisType,
    output_format: AnalysisOutputFormat,
    output: Option<PathBuf>,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Analyzing document: {}...", "🔬".bright_blue(), input.display());
    }

    let analyzer = DocumentAnalyzer::new();
    let analysis_result = analyzer.analyze(&input, &analysis_type, &output_format)?;

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

async fn handle_merge(
    inputs: Vec<PathBuf>,
    output: PathBuf,
    format_strategy: MergeStrategy,
    config: &Config,
    verbose: bool,
    quiet: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("{} Merging {} documents...", "🔗".bright_blue(), inputs.len());
    }

    let merger = DocumentMerger::new(config);
    let merged_data = merger.merge(&inputs, &format_strategy)?;

    fs::write(&output, merged_data)?;

    if !quiet {
        let size = fs::metadata(&output)?.len();
        println!("{} Merged document created: {} ({})",
            "✅".green(),
            output.display().to_string().bright_white(),
            format_size(size).bright_cyan()
        );
    }

    Ok(())
}

async fn handle_template(
    action: TemplateAction,
    config: &Config,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let template_manager = TemplateManager::new(config);

    match action {
        TemplateAction::List => {
            let templates = template_manager.list_templates()?;
            if !quiet {
                println!("{} Available templates:", "📋".bright_blue());
                for (name, info) in templates {
                    println!("  {} {} - {}", "•".bright_white(), name.bright_white(), info.description.bright_black());
                }
            }
        },
        TemplateAction::Show { name } => {
            template_manager.show_template(&name)?;
        },
        TemplateAction::Create { name, from } => {
            template_manager.create_template(&name, from.as_deref())?;
            if !quiet {
                println!("{} Template '{}' created", "✅".green(), name);
            }
        },
        TemplateAction::Edit { name } => {
            template_manager.edit_template(&name)?;
        },
        TemplateAction::Delete { name } => {
            template_manager.delete_template(&name)?;
            if !quiet {
                println!("{} Template '{}' deleted", "✅".green(), name);
            }
        },
        TemplateAction::Install { path } => {
            let name = template_manager.install_template(&path)?;
            if !quiet {
                println!("{} Template '{}' installed", "✅".green(), name);
            }
        },
        TemplateAction::Export { name, output } => {
            template_manager.export_template(&name, &output)?;
            if !quiet {
                println!("{} Template '{}' exported to {}", "✅".green(), name, output.display());
            }
        },
    }

    Ok(())
}

async fn handle_config(
    action: ConfigAction,
    config: &Config,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new();

    match action {
        ConfigAction::Show => {
            config_manager.show_config(config)?;
        },
        ConfigAction::Set { key, value } => {
            config_manager.set_value(&key, &value)?;
            if !quiet {
                println!("{} Configuration updated: {} = {}", "✅".green(), key, value);
            }
        },
        ConfigAction::Get { key } => {
            let value = config_manager.get_value(&key)?;
            println!("{}", value);
        },
        ConfigAction::Reset => {
            config_manager.reset_to_defaults()?;
            if !quiet {
                println!("{} Configuration reset to defaults", "✅".green());
            }
        },
        ConfigAction::Edit => {
            config_manager.edit_interactive()?;
        },
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
                "html" | "htm" => OutputFormat::Html,
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
                OutputFormat::Html => "html",
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

fn find_input_files(pattern: &str, recursive: bool) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    if pattern.contains('*') {
        // Handle glob patterns
        // For now, simple implementation - can be enhanced with glob crate
        let path = PathBuf::from(pattern.replace("*", ""));
        let parent = path.parent().unwrap_or(Path::new("."));

        if recursive {
            for entry in WalkDir::new(parent) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext, "md" | "markdown" | "txt" | "docx" | "xlsx" | "pptx" | "pdf") {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        } else {
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries {
                    let entry = entry?;
                    if entry.file_type()?.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if matches!(ext, "md" | "markdown" | "txt" | "docx" | "xlsx" | "pptx" | "pdf") {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Single file or directory
        let path = PathBuf::from(pattern);
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            if recursive {
                for entry in WalkDir::new(&path) {
                    let entry = entry?;
                    if entry.file_type().is_file() {
                        files.push(entry.path().to_path_buf());
                    }
                }
            } else {
                for entry in fs::read_dir(&path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_file() {
                        files.push(entry.path());
                    }
                }
            }
        }
    }

    Ok(files)
}

fn determine_batch_output(
    input: &Path,
    output_dir: &Path,
    format: &OutputFormat,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let ext = match format {
        OutputFormat::Docx => "docx",
        OutputFormat::Xlsx => "xlsx",
        OutputFormat::Pptx => "pptx",
        OutputFormat::Pdf => "pdf",
        OutputFormat::Markdown => "md",
        OutputFormat::Text => "txt",
        OutputFormat::Html => "html",
        OutputFormat::Json => "json",
    };

    Ok(output_dir.join(format!("{}.{}", stem, ext)))
}

async fn convert_single_file(
    converter: &mut DocumentConverter,
    input: &Path,
    output: &Path,
    format: &OutputFormat,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if output.exists() && !force {
        return Err("Output file exists".into());
    }

    let input_content = fs::read_to_string(input)?;
    let input_ext = input.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let output_data = match input_ext.as_str() {
        "md" | "markdown" => converter.convert_markdown(&input_content, format)?,
        "txt" | "text" => converter.convert_text(&input_content, format)?,
        _ => return Err("Unsupported input format".into()),
    };

    fs::write(output, output_data)?;
    Ok(())
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

fn get_available_templates(config: &Config) -> Result<HashMap<String, TemplateInfo>, Box<dyn std::error::Error>> {
    // This would load from template directory
    let mut templates = HashMap::new();

    templates.insert("business".to_string(), TemplateInfo {
        name: "business".to_string(),
        description: "Professional business document template".to_string(),
        doc_type: DocumentType::Document,
        features: vec!["header".to_string(), "footer".to_string(), "toc".to_string()],
    });

    templates.insert("academic".to_string(), TemplateInfo {
        name: "academic".to_string(),
        description: "Academic paper template with citations".to_string(),
        doc_type: DocumentType::Document,
        features: vec!["bibliography".to_string(), "footnotes".to_string()],
    });

    templates.insert("presentation".to_string(), TemplateInfo {
        name: "presentation".to_string(),
        description: "Modern presentation template".to_string(),
        doc_type: DocumentType::Presentation,
        features: vec!["slide_numbers".to_string(), "animations".to_string()],
    });

    Ok(templates)
}