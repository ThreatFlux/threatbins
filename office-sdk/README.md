# Office SDK

A Rust SDK for creating and parsing Microsoft Office compressed formats (DOCX, XLSX, PPTX, and other OpenXML documents) with full Markdown conversion support.

## Features

- ✅ **DOCX Documents** with advanced features:
  - Images (PNG, JPEG, GIF, BMP) with auto-sizing
  - Tables with styling, colors, and alignment
  - VBA macro embedding (for security analysis)
  - Full metadata management (author, dates, properties)
  - Page layout control (size, orientation, margins)
  - Professional formatting and styles
- ✅ **XLSX Spreadsheets** with full functionality
- ✅ **PPTX Presentations** with slide management
- ✅ **Markdown ↔ DOCX conversion** (bidirectional)
- ✅ Built on stable Rust 1.95.0
- ✅ Memory-efficient ZIP-based implementation
- ✅ Comprehensive error handling
- ✅ Type-safe APIs
- ✅ 90%+ code coverage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
office-sdk = "0.1.0"
```

## Quick Start

### Working with DOCX Documents

```rust
use office_sdk::docx::DocxDocument;
use office_sdk::OfficeDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new document
    let mut doc = DocxDocument::new();

    // Add paragraphs
    doc.add_paragraph("Hello, World!")?;
    doc.add_paragraph("This is created with Office SDK.")?;

    // Save to file
    doc.save("document.docx")?;

    // Load from file
    let loaded_doc = DocxDocument::from_file("document.docx")?;
    let text = loaded_doc.get_text()?;
    println!("Document content: {}", text);

    Ok(())
}
```

### Working with XLSX Spreadsheets

```rust
use office_sdk::xlsx::XlsxDocument;
use office_sdk::OfficeDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new spreadsheet
    let mut doc = XlsxDocument::new();

    // Create data
    let data = vec![
        vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
        vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
        vec!["Bob".to_string(), "25".to_string(), "Boston".to_string()],
    ];

    // Add worksheet
    doc.add_worksheet("People", &data)?;

    // Save to file
    doc.save("spreadsheet.xlsx")?;

    // Load from file
    let loaded_doc = XlsxDocument::from_file("spreadsheet.xlsx")?;
    let worksheet_data = loaded_doc.get_worksheet_data("People")?;

    for (i, row) in worksheet_data.iter().enumerate() {
        println!("Row {}: {:?}", i + 1, row);
    }

    Ok(())
}
```

### Converting Between Markdown and DOCX

```rust
use office_sdk::markdown::MarkdownDocument;
use office_sdk::docx::DocxDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse markdown from string
    let markdown_text = r#"# Document Title

This is a paragraph with **bold** text.

## Section

- List item 1
- List item 2

```rust
fn hello() {
    println!("Hello, world!");
}
```

> Blockquote content

---

Final paragraph.
"#;

    // Convert Markdown → DOCX
    let markdown_doc = MarkdownDocument::from_markdown(markdown_text)?;
    let docx_doc = markdown_doc.to_docx()?;
    docx_doc.save("converted.docx")?;

    // Convert DOCX → Markdown
    let loaded_docx = DocxDocument::from_file("converted.docx")?;
    let markdown_from_docx = MarkdownDocument::from_docx(&loaded_docx)?;
    let regenerated_markdown = markdown_from_docx.to_markdown();

    println!("Regenerated markdown:\n{}", regenerated_markdown);

    Ok(())
}
```

### Working with PPTX Presentations

```rust
use office_sdk::pptx::PptxDocument;
use office_sdk::OfficeDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new presentation
    let mut doc = PptxDocument::new();

    // Add slides
    doc.add_slide("Welcome", &["Introduction slide", "Created with Office SDK"])?;
    doc.add_slide("Features", &["Slide management", "Text extraction", "Full OpenXML support"])?;

    // Save to file
    doc.save("presentation.pptx")?;

    // Load from file
    let loaded_doc = PptxDocument::from_file("presentation.pptx")?;
    let slides = loaded_doc.get_all_slides()?;

    for (i, slide) in slides.iter().enumerate() {
        println!("Slide {}: {}", i + 1, slide.title);
        for content in &slide.content {
            println!("  - {}", content);
        }
    }

    // Extract all text
    let all_text = loaded_doc.get_slide_text()?;
    println!("All presentation text:\n{}", all_text);

    Ok(())
}
```

## Core Concepts

### OfficeDocument Trait

All document types implement the `OfficeDocument` trait, providing consistent APIs:

```rust
pub trait OfficeDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> where Self: Sized;
    fn from_bytes(data: &[u8]) -> Result<Self> where Self: Sized;
    fn to_bytes(&self) -> Result<Vec<u8>>;
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}
```

### Error Handling

The SDK uses comprehensive error types for better debugging:

```rust
use office_sdk::error::{OfficeError, Result};

match DocxDocument::from_file("missing.docx") {
    Ok(doc) => println!("Document loaded successfully"),
    Err(OfficeError::Io(e)) => println!("IO error: {}", e),
    Err(OfficeError::InvalidFormat(msg)) => println!("Invalid format: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Document Types

### DOCX Documents

The `DocxDocument` struct provides comprehensive Word document support:

**Basic Content:**
- `new()` - Create a new empty document
- `add_paragraph(text)` - Add a text paragraph
- `add_heading(level, text)` - Add styled headings (H1-H6)
- `add_code_block(code)` - Add formatted code blocks
- `get_text()` - Extract all text content

**Lists and Formatting:**
- `add_bulleted_list_item(text)` - Create bullet point lists
- `add_numbered_list_item(text, number)` - Create numbered lists
- `add_quote(text)` - Add blockquote formatting
- `add_horizontal_rule()` - Insert document dividers

**Tables:**
- `create_table(headers, rows)` - Create simple tables
- `add_table(table)` - Add complex tables with custom styling
- Support for cell background colors and text alignment
- Advanced table formatting with borders and spacing

**Images:**
- `add_image(path, alt_text)` - Embed images from files
- `add_image_from_bytes(data, content_type, alt_text)` - Embed from memory
- Support for PNG, JPEG, GIF, BMP formats
- Automatic dimension detection and scaling

**Page Layout:**
- `set_page_size(paper_size)` - Configure paper size (A4, Letter, Legal, etc.)
- `set_orientation(orientation)` - Set portrait/landscape
- `set_margins(top, bottom, left, right)` - Custom margin control

**Document Metadata:**
- `with_author(name)` - Set document author
- `with_title(title)` - Set document title
- `with_subject(subject)` - Set document subject
- `update_metadata()` - Update modification timestamps
- Full metadata tracking (creation time, revision, etc.)

**Advanced Features:**
- `add_vba_macro(data)` - Embed VBA projects (for security analysis)
- Comprehensive styles and formatting
- Professional document structure

### XLSX Spreadsheets

The `XlsxDocument` struct provides:
- `new()` - Create a new empty spreadsheet
- `add_worksheet(name, data)` - Add a worksheet with data
- `get_worksheet_data(name)` - Extract worksheet data
- Support for multiple worksheets
- Mixed text/numeric cell support

### PPTX Presentations

The `PptxDocument` struct provides:
- `new()` - Create a new empty presentation
- `add_slide(title, content)` - Add a slide with title and content
- `get_slide_content(slide_id)` - Get specific slide data
- `get_all_slides()` - Get all slides in order
- `get_slide_text()` - Extract all text from presentation
- Full slide master and layout support

### Markdown Documents

The `MarkdownDocument` struct provides:
- `from_markdown(text)` - Parse markdown text into structured document
- `to_markdown()` - Generate markdown text from document structure
- `to_docx()` - Convert markdown to DOCX format
- `from_docx(docx)` - Extract markdown from DOCX document
- Support for headings, paragraphs, lists, code blocks, quotes, and more

## Examples

Run the included examples:

```bash
# DOCX example
cargo run --example docx_example

# XLSX example
cargo run --example xlsx_example

# PPTX example
cargo run --example pptx_example

# Advanced DOCX features example
cargo run --example advanced_docx

# Markdown conversion example
cargo run --example markdown_conversion
```

## Testing

Run the test suite:

```bash
cargo test
```

The tests include:
- Document creation and parsing
- Advanced DOCX features (images, tables, VBA, metadata)
- Markdown conversion and round-trip testing
- File I/O operations
- Error handling scenarios
- Format validation
- Page layout and formatting

## Architecture

The SDK is built on:
- **ZIP handling**: `zip` crate for compressed format support
- **XML processing**: `quick-xml` for efficient XML parsing
- **Error management**: `thiserror` for structured error types
- **Serialization**: `serde` for data structure handling

## Supported Formats

Currently supported:
- ✅ DOCX (Word documents)
- ✅ XLSX (Excel spreadsheets)
- ✅ PPTX (PowerPoint presentations)
- ✅ Markdown (bidirectional conversion with DOCX)

Supported Markdown elements:
- Headers (H1-H6)
- Paragraphs
- Code blocks (with language detection)
- Ordered and unordered lists
- Blockquotes
- Horizontal rules

Planned:
- Enhanced inline formatting (bold, italic, links)
- Table support in conversions
- Additional OpenXML formats

## Requirements

- Rust 1.95.0 or later
- Standard library support for file I/O

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Security

This SDK is designed for defensive security tasks only. It focuses on:
- Safe parsing of Office documents
- Secure file handling
- Memory-safe operations
- Input validation

Do not use for malicious document creation or credential harvesting.
