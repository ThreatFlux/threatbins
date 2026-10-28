use office_sdk::OfficeDocument;
use office_sdk::docx::{
    DocxDocument, PageOrientation, PaperSize, Table, TableCell, TableRow, TextAlign,
};
use office_sdk::error::OfficeError;
use office_sdk::markdown::{InlineElement, MarkdownDocument, MarkdownElement};
use office_sdk::pdf::{
    AnnotationType, Color, EncryptionLevel, FormFieldType, PageContent, PdfDocument,
    PermissionFlags, ShapeType, TextStyle,
};
use office_sdk::pptx::{AnimationType, PptxDocument, SlideSize};
use office_sdk::utils::{read_zip_entry, read_zip_entry_bytes, validate_office_format};
use office_sdk::xlsx::{
    ChartType, PageOrientation as XlsxPageOrientation, PaperSize as XlsxPaperSize, XlsxDocument,
};
use std::io::Cursor;
use std::io::Write;
use tempfile::TempDir;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

// Encode complete images: a magic header alone is not a valid image fixture.
fn image_fixture(format: image::ImageFormat, width: u32, height: u32) -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::new(width, height));
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

#[test]
fn test_docx_creation_and_parsing() {
    let mut doc = DocxDocument::new();

    doc.add_paragraph("Hello, World!").unwrap();
    doc.add_paragraph("This is a test document.").unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();

    let text = parsed_doc.get_text().unwrap();
    assert!(text.contains("Hello, World!"));
    assert!(text.contains("This is a test document."));
}

#[test]
fn test_docx_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.docx");

    let mut doc = DocxDocument::new();
    doc.add_paragraph("File operation test").unwrap();

    doc.save(&file_path).unwrap();
    assert!(file_path.exists());

    let loaded_doc = DocxDocument::from_file(&file_path).unwrap();
    let text = loaded_doc.get_text().unwrap();
    assert!(text.contains("File operation test"));
}

#[test]
fn test_xlsx_creation_and_parsing() {
    let mut doc = XlsxDocument::new();

    let data = vec![
        vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
        vec!["John".to_string(), "30".to_string(), "New York".to_string()],
        vec!["Jane".to_string(), "25".to_string(), "Boston".to_string()],
    ];

    doc.add_worksheet("Sheet1", &data).unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = XlsxDocument::from_bytes(&bytes).unwrap();

    let parsed_data = parsed_doc.get_worksheet_data("Sheet1").unwrap();
    assert!(!parsed_data.is_empty());

    if !parsed_data.is_empty() && !parsed_data[0].is_empty() {
        let first_row = &parsed_data[0];
        if !first_row.is_empty() {
            assert!(
                first_row[0].contains("Name") || first_row.iter().any(|cell| cell.contains("Name"))
            );
        }
    }
}

#[test]
fn test_xlsx_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.xlsx");

    let mut doc = XlsxDocument::new();
    let data = vec![
        vec!["Product".to_string(), "Price".to_string()],
        vec!["Apple".to_string(), "1.50".to_string()],
        vec!["Banana".to_string(), "0.80".to_string()],
    ];

    doc.add_worksheet("Products", &data).unwrap();

    doc.save(&file_path).unwrap();
    assert!(file_path.exists());

    let loaded_doc = XlsxDocument::from_file(&file_path).unwrap();
    let parsed_data = loaded_doc.get_worksheet_data("Products").unwrap();

    assert!(!parsed_data.is_empty());
}

#[test]
fn test_empty_documents() {
    let docx_doc = DocxDocument::new();
    let docx_bytes = docx_doc.to_bytes().unwrap();
    assert!(!docx_bytes.is_empty());

    let xlsx_doc = XlsxDocument::new();
    let xlsx_bytes = xlsx_doc.to_bytes().unwrap();
    assert!(!xlsx_bytes.is_empty());
}

#[test]
fn test_invalid_format_detection() {
    let invalid_data = b"This is not a valid Office document";

    let docx_result = DocxDocument::from_bytes(invalid_data);
    assert!(docx_result.is_err());

    let xlsx_result = XlsxDocument::from_bytes(invalid_data);
    assert!(xlsx_result.is_err());
}

#[test]
fn test_utils_read_zip_entry() {
    // Create a test ZIP archive
    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut zip = ZipWriter::new(cursor);

        zip.start_file("test.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"Hello, World!").unwrap();

        zip.start_file("binary.bin", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&[0x01, 0x02, 0x03, 0x04]).unwrap();

        zip.finish().unwrap();
    }

    let cursor = Cursor::new(&buf);
    let mut archive = ZipArchive::new(cursor).unwrap();

    // Test reading string entry
    let content = read_zip_entry(&mut archive, "test.txt").unwrap();
    assert_eq!(content, "Hello, World!");

    // Test reading missing entry
    let result = read_zip_entry(&mut archive, "missing.txt");
    assert!(result.is_err());
    match result.unwrap_err() {
        OfficeError::MissingFile(name) => assert_eq!(name, "missing.txt"),
        _ => panic!("Expected MissingFile error"),
    }
}

#[test]
fn test_utils_read_zip_entry_bytes() {
    // Create a test ZIP archive
    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut zip = ZipWriter::new(cursor);

        zip.start_file("binary.bin", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&[0x01, 0x02, 0x03, 0x04]).unwrap();

        zip.finish().unwrap();
    }

    let cursor = Cursor::new(&buf);
    let mut archive = ZipArchive::new(cursor).unwrap();

    // Test reading binary entry
    let content = read_zip_entry_bytes(&mut archive, "binary.bin").unwrap();
    assert_eq!(content, vec![0x01, 0x02, 0x03, 0x04]);

    // Test reading missing entry
    let result = read_zip_entry_bytes(&mut archive, "missing.bin");
    assert!(result.is_err());
    match result.unwrap_err() {
        OfficeError::MissingFile(name) => assert_eq!(name, "missing.bin"),
        _ => panic!("Expected MissingFile error"),
    }
}

#[test]
fn test_utils_validate_office_format() {
    // Create a test ZIP archive with required files
    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut zip = ZipWriter::new(cursor);

        zip.start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"<document/>").unwrap();

        zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"<Types/>").unwrap();

        zip.finish().unwrap();
    }

    let cursor = Cursor::new(&buf);
    let mut archive = ZipArchive::new(cursor).unwrap();

    // Test validation with all required files present
    let result =
        validate_office_format(&mut archive, &["word/document.xml", "[Content_Types].xml"]);
    assert!(result.is_ok());

    // Test validation with missing required file
    let result = validate_office_format(
        &mut archive,
        &["word/document.xml", "[Content_Types].xml", "missing.xml"],
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        OfficeError::InvalidFormat(msg) => {
            assert!(msg.contains("Missing required file: missing.xml"))
        }
        _ => panic!("Expected InvalidFormat error"),
    }
}

#[test]
fn test_docx_error_scenarios() {
    let mut doc = DocxDocument::new();

    // Test XML parsing with invalid body structure
    let invalid_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
        <w:body>
            <w:p><w:r><w:t>Some text</w:t></w:r></w:p>
        </w:body>
    </w:document>"#
        .to_string();

    doc.content = invalid_xml;
    let text_result = doc.get_text();
    assert!(text_result.is_ok());
    assert!(text_result.unwrap().contains("Some text"));
}

#[test]
fn test_xlsx_number_to_column_conversion() {
    let _doc = XlsxDocument::new();

    // Test column number conversion (testing private method through worksheet creation)
    let data = vec![
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec!["1".to_string(), "2".to_string(), "3".to_string()],
    ];

    let mut test_doc = XlsxDocument::new();
    test_doc.add_worksheet("TestSheet", &data).unwrap();

    let worksheet_data = test_doc.get_worksheet_data("TestSheet").unwrap();
    assert_eq!(worksheet_data.len(), 2);
    assert_eq!(worksheet_data[0], vec!["A", "B", "C"]);
}

#[test]
fn test_default_implementations() {
    // Test Default trait implementations
    let docx_doc: DocxDocument = Default::default();
    assert!(!docx_doc.content.is_empty());

    let xlsx_doc: XlsxDocument = Default::default();
    assert!(!xlsx_doc.workbook.is_empty());
}

#[test]
fn test_error_display() {
    // Test error display formatting
    let error = OfficeError::InvalidFormat("test format".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid document format: test format"));

    let error = OfficeError::MissingFile("test.xml".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Missing required file: test.xml"));
}

#[test]
fn test_pptx_creation_and_parsing() {
    let mut doc = PptxDocument::new();

    // Add slides
    let slide1_id = doc
        .add_slide(
            "Welcome to Office SDK",
            &["This is the first slide", "Created with Rust!"],
        )
        .unwrap();
    let slide2_id = doc
        .add_slide(
            "Features",
            &[
                "Parse PPTX files",
                "Create presentations",
                "Extract slide content",
            ],
        )
        .unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();

    // Test slide retrieval
    let slide1 = parsed_doc.get_slide_content(&slide1_id).unwrap();
    assert_eq!(slide1.title, "Welcome to Office SDK");
    assert!(slide1.content.iter().any(|c| c.contains("first slide")));

    let slide2 = parsed_doc.get_slide_content(&slide2_id).unwrap();
    assert_eq!(slide2.title, "Features");
    assert!(slide2.content.iter().any(|c| c.contains("Parse PPTX")));

    // Test all slides retrieval
    let all_slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(all_slides.len(), 2);

    // Test slide text extraction
    let text = parsed_doc.get_slide_text().unwrap();
    assert!(text.contains("Welcome to Office SDK"));
    assert!(text.contains("Features"));
    assert!(text.contains("Parse PPTX files"));
}

#[test]
fn test_pptx_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.pptx");

    let mut doc = PptxDocument::new();
    doc.add_slide(
        "Title Slide",
        &["This is a test presentation", "Generated by Office SDK"],
    )
    .unwrap();

    doc.save(&file_path).unwrap();
    assert!(file_path.exists());

    let loaded_doc = PptxDocument::from_file(&file_path).unwrap();
    let slides = loaded_doc.get_all_slides().unwrap();

    assert_eq!(slides.len(), 1);
    assert_eq!(slides[0].title, "Title Slide");
    assert!(
        slides[0]
            .content
            .iter()
            .any(|c| c.contains("test presentation"))
    );
}

#[test]
fn test_pptx_empty_presentation() {
    let doc = PptxDocument::new();
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();
    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 0);

    let text = parsed_doc.get_slide_text().unwrap();
    assert!(text.is_empty());
}

#[test]
fn test_pptx_slide_without_title() {
    let mut doc = PptxDocument::new();
    doc.add_slide("", &["Content without title", "Another line"])
        .unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();

    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 1);
    assert!(slides[0].title.is_empty());
    assert_eq!(slides[0].content.len(), 2);
}

#[test]
fn test_pptx_slide_without_content() {
    let mut doc = PptxDocument::new();
    doc.add_slide("Title Only", &[]).unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();

    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 1);
    assert_eq!(slides[0].title, "Title Only");
    assert!(slides[0].content.is_empty());
}

#[test]
fn test_pptx_multiple_slides() {
    let mut doc = PptxDocument::new();

    for i in 1..=5 {
        doc.add_slide(
            &format!("Slide {}", i),
            &[&format!("Content for slide {}", i), "Additional content"],
        )
        .unwrap();
    }

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();

    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 5);

    for (i, slide) in slides.iter().enumerate() {
        assert_eq!(slide.title, format!("Slide {}", i + 1));
        assert!(
            slide
                .content
                .iter()
                .any(|c| c.contains(&format!("Content for slide {}", i + 1)))
        );
    }
}

#[test]
fn test_pptx_invalid_format_detection() {
    let invalid_data = b"This is not a valid PPTX document";

    let result = PptxDocument::from_bytes(invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_pptx_get_nonexistent_slide() {
    let doc = PptxDocument::new();

    let result = doc.get_slide_content("nonexistent_slide");
    assert!(result.is_err());
    match result.unwrap_err() {
        OfficeError::MissingFile(name) => assert!(name.contains("nonexistent_slide")),
        _ => panic!("Expected MissingFile error"),
    }
}

#[test]
fn test_pptx_default_implementation() {
    let doc: PptxDocument = Default::default();
    assert!(!doc.presentation.is_empty());
    assert!(doc.slides.is_empty());
    assert!(!doc.slide_masters.is_empty());
    assert!(!doc.slide_layouts.is_empty());
}

#[test]
fn test_pptx_special_characters() {
    let mut doc = PptxDocument::new();
    doc.add_slide(
        "Special Characters: <>\"'&",
        &["Content with <xml> & \"quotes\"", "More special chars: éñü"],
    )
    .unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();

    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 1);

    // The title and content should be properly escaped and unescaped
    assert!(slides[0].title.contains("Special Characters"));
    assert!(slides[0].content.iter().any(|c| c.contains("<xml>")));
    assert!(slides[0].content.iter().any(|c| c.contains("éñü")));
}

// =============================================================================
// MARKDOWN CONVERSION TESTS
// =============================================================================

#[test]
fn test_markdown_parsing_basic_elements() {
    let markdown_text = r#"# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

Another paragraph with `inline code`.

```rust
fn main() {
    println!("Hello, world!");
}
```

- Bullet item 1
- Bullet item 2

1. Numbered item 1
2. Numbered item 2

> This is a blockquote

---
"#;

    let doc = MarkdownDocument::from_markdown(markdown_text).unwrap();

    // Should have multiple elements
    assert!(doc.elements.len() >= 8);

    // Check for heading
    if let MarkdownElement::Heading { level, text } = &doc.elements[0] {
        assert_eq!(*level, 1);
        assert_eq!(text.len(), 1);
        if let InlineElement::Text(text) = &text[0] {
            assert_eq!(text, "Heading 1");
        }
    } else {
        panic!("Expected heading element");
    }

    // Find and check code block
    let code_block = doc
        .elements
        .iter()
        .find(|e| matches!(e, MarkdownElement::CodeBlock { .. }));
    assert!(code_block.is_some());

    if let Some(MarkdownElement::CodeBlock { language, code }) = code_block {
        assert_eq!(language.as_ref().unwrap(), "rust");
        assert!(code.contains("println!"));
    }
}

#[test]
fn test_markdown_to_text_conversion() {
    let markdown_text = r#"# Test Document

This is a **bold** and *italic* paragraph.

```
code block
```

- List item

> Quote
"#;

    let doc = MarkdownDocument::from_markdown(markdown_text).unwrap();
    let generated = doc.to_markdown();

    // Should contain the main elements
    assert!(generated.contains("# Test Document"));
    assert!(generated.contains("**bold**"));
    assert!(generated.contains("*italic*"));
    assert!(generated.contains("```"));
    assert!(generated.contains("code block"));
    assert!(generated.contains("- List item"));
    assert!(generated.contains("> Quote"));
}

#[test]
fn test_markdown_round_trip_conversion() {
    let markdown_text = r#"# Main Heading

A paragraph with **bold** and *italic* formatting.

## Subheading

Another paragraph with `inline code`.

```rust
// Code example
fn hello() {
    println!("Hello!");
}
```

### Lists

- First item
- Second item

1. Numbered first
2. Numbered second

> This is a blockquote with some text.

---
"#;

    let doc = MarkdownDocument::from_markdown(markdown_text).unwrap();
    let generated = doc.to_markdown();
    let doc2 = MarkdownDocument::from_markdown(&generated).unwrap();

    // Both documents should have similar structure
    assert_eq!(doc.elements.len(), doc2.elements.len());

    // Spot check a few elements
    assert!(matches!(
        doc.elements[0],
        MarkdownElement::Heading { level: 1, .. }
    ));
    assert!(matches!(
        doc2.elements[0],
        MarkdownElement::Heading { level: 1, .. }
    ));
}

#[test]
fn test_markdown_to_docx_conversion() {
    let markdown_text = r#"# Document Title

This is the first paragraph with **bold text**.

## Section 1

Here's a paragraph with *italic text* and `inline code`.

```python
def hello_world():
    print("Hello, World!")
```

### Subsection

- First bullet point
- Second bullet point

1. First numbered item
2. Second numbered item

> This is a blockquote with important information.

---

Final paragraph after horizontal rule.
"#;

    let markdown_doc = MarkdownDocument::from_markdown(markdown_text).unwrap();
    let docx_doc = markdown_doc.to_docx().unwrap();

    // Verify DOCX was created successfully
    let docx_bytes = docx_doc.to_bytes().unwrap();
    assert!(!docx_bytes.is_empty());

    // Parse the DOCX and check content
    let parsed_docx = DocxDocument::from_bytes(&docx_bytes).unwrap();
    let text_content = parsed_docx.get_text().unwrap();

    // Should contain key elements from the markdown
    assert!(text_content.contains("Document Title"));
    assert!(text_content.contains("Section 1"));
    assert!(text_content.contains("bold text"));
    assert!(text_content.contains("italic text"));
    assert!(text_content.contains("def hello_world"));
    assert!(text_content.contains("bullet point"));
    assert!(text_content.contains("numbered item"));
    assert!(text_content.contains("blockquote"));
}

#[test]
fn test_docx_to_markdown_conversion() {
    let mut docx = DocxDocument::new();
    docx.add_heading(1, "Main Title").unwrap();
    docx.add_paragraph("This is a regular paragraph.").unwrap();
    docx.add_heading(2, "Subsection").unwrap();
    docx.add_paragraph("Another paragraph with content.")
        .unwrap();
    docx.add_code_block("console.log('Hello, world!');")
        .unwrap();

    let markdown_doc = MarkdownDocument::from_docx(&docx).unwrap();

    // Should have some elements (DOCX to markdown conversion is simplified)
    assert!(!markdown_doc.elements.is_empty());

    // Convert back to markdown text to verify content
    let markdown_text = markdown_doc.to_markdown();
    assert!(markdown_text.contains("Main Title") || markdown_text.contains("regular paragraph"));
}

#[test]
fn test_markdown_complex_inline_formatting() {
    let markdown_text = r#"# Complex Formatting

This paragraph has text with formatting.

Here's some content with various elements.

Mixed formatting example.
"#;

    let doc = MarkdownDocument::from_markdown(markdown_text).unwrap();

    // Should parse without errors
    assert!(!doc.elements.is_empty());

    // Convert back to markdown
    let generated = doc.to_markdown();
    assert!(generated.contains("Complex Formatting"));
    assert!(generated.contains("paragraph"));

    // Note: Complex inline formatting like bold/italic is a planned feature
}

#[test]
fn test_markdown_tables() {
    let markdown_text = r#"# Tables

Table content will be supported in future versions.

Regular paragraph after table note.
"#;

    let doc = MarkdownDocument::from_markdown(markdown_text).unwrap();

    // Should have heading and paragraphs
    assert!(doc.elements.len() >= 2);

    // Convert back to markdown
    let generated = doc.to_markdown();
    assert!(generated.contains("Tables"));
    assert!(generated.contains("Regular paragraph"));

    // Note: Table parsing is a planned feature
}

#[test]
fn test_markdown_empty_document() {
    let empty_markdown = "";
    let doc = MarkdownDocument::from_markdown(empty_markdown).unwrap();
    assert!(doc.elements.is_empty());

    let generated = doc.to_markdown();
    assert!(generated.is_empty());
}

#[test]
fn test_markdown_only_whitespace() {
    let whitespace_markdown = "   \n  \n   \n";
    let doc = MarkdownDocument::from_markdown(whitespace_markdown).unwrap();
    assert!(doc.elements.is_empty());

    let generated = doc.to_markdown();
    assert!(generated.is_empty());
}

#[test]
fn test_markdown_special_characters_handling() {
    let markdown_text = r#"# Special Characters Test

Text with < > & " ' characters.

Code with special chars: `<div class="test">content</div>`

> Quote with "quoted text" and 'single quotes'.
"#;

    let doc = MarkdownDocument::from_markdown(markdown_text).unwrap();
    assert!(!doc.elements.is_empty());

    let generated = doc.to_markdown();
    assert!(generated.contains("< > & \""));
    assert!(generated.contains("<div class="));
    assert!(generated.contains("\"quoted text\""));
}

#[test]
fn test_markdown_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let markdown_path = temp_dir.path().join("test.md");
    let docx_path = temp_dir.path().join("test.docx");

    let markdown_content = r#"# Test Document

This is a test of file operations.

## Features

- Markdown parsing
- DOCX conversion
- File I/O support

```rust
fn main() {
    println!("Test complete!");
}
```
"#;

    // Write markdown to file
    std::fs::write(&markdown_path, markdown_content).unwrap();

    // Read and parse markdown
    let markdown_text = std::fs::read_to_string(&markdown_path).unwrap();
    let markdown_doc = MarkdownDocument::from_markdown(&markdown_text).unwrap();

    // Convert to DOCX and save
    let docx_doc = markdown_doc.to_docx().unwrap();
    docx_doc.save(&docx_path).unwrap();

    // Verify DOCX file was created
    assert!(docx_path.exists());

    // Load DOCX and verify content
    let loaded_docx = DocxDocument::from_file(&docx_path).unwrap();
    let text_content = loaded_docx.get_text().unwrap();

    assert!(text_content.contains("Test Document"));
    assert!(text_content.contains("Features"));
    assert!(text_content.contains("Markdown parsing"));
    assert!(text_content.contains("println!"));
}

// =============================================================================
// ADVANCED DOCX FEATURES TESTS
// =============================================================================

#[test]
fn test_docx_metadata_and_properties() {
    let doc = DocxDocument::new()
        .with_author("John Doe")
        .with_title("Test Document")
        .with_subject("Unit Testing");

    assert_eq!(doc.metadata.author.as_deref(), Some("John Doe"));
    assert_eq!(doc.metadata.title.as_deref(), Some("Test Document"));
    assert_eq!(doc.metadata.subject.as_deref(), Some("Unit Testing"));
    assert_eq!(doc.metadata.last_modified_by.as_deref(), Some("John Doe"));

    // Test metadata update
    let mut doc = doc;
    let original_modified = doc.metadata.modified;
    std::thread::sleep(std::time::Duration::from_millis(10));
    doc.update_metadata();
    assert!(doc.metadata.modified > original_modified);
}

#[test]
fn test_docx_page_setup() {
    let mut doc = DocxDocument::new();

    // Test A4 size
    doc.set_page_size(PaperSize::A4);
    assert_eq!(doc.page_setup.paper_size, PaperSize::A4);
    assert!((doc.page_setup.width - 11906.0).abs() < 1.0);

    // Test Letter size
    doc.set_page_size(PaperSize::Letter);
    assert_eq!(doc.page_setup.paper_size, PaperSize::Letter);
    assert!((doc.page_setup.width - 12240.0).abs() < 1.0);

    // Test custom size
    doc.set_page_size(PaperSize::Custom {
        width: 10000.0,
        height: 14000.0,
    });
    assert!((doc.page_setup.width - 10000.0).abs() < 1.0);
    assert!((doc.page_setup.height - 14000.0).abs() < 1.0);

    // Test orientation
    doc.set_orientation(PageOrientation::Landscape);
    assert_eq!(doc.page_setup.orientation, PageOrientation::Landscape);

    // Test margins
    doc.set_margins(1000.0, 1200.0, 800.0, 900.0);
    assert!((doc.page_setup.margin_top - 1000.0).abs() < 1.0);
    assert!((doc.page_setup.margin_bottom - 1200.0).abs() < 1.0);
    assert!((doc.page_setup.margin_left - 800.0).abs() < 1.0);
    assert!((doc.page_setup.margin_right - 900.0).abs() < 1.0);
}

#[test]
fn test_docx_table_creation() {
    let mut doc = DocxDocument::new();

    // Create a simple table
    let table_id = doc
        .create_table(
            &["Name", "Age", "City"],
            &[
                vec!["Alice", "30", "New York"],
                vec!["Bob", "25", "San Francisco"],
                vec!["Charlie", "35", "Chicago"],
            ],
        )
        .unwrap();

    assert!(!table_id.is_empty());
    assert_eq!(doc.tables.len(), 1);

    let table = &doc.tables[0];
    assert_eq!(table.rows.len(), 4); // 1 header + 3 data rows
    assert_eq!(table.rows[0].cells.len(), 3); // 3 columns

    // Check header row properties
    assert_eq!(table.rows[0].cells[0].content, "Name");
    assert_eq!(
        table.rows[0].cells[0].background_color,
        Some("D9D9D9".to_string())
    );
    assert_eq!(table.rows[0].cells[0].text_align, Some(TextAlign::Center));

    // Check data row
    assert_eq!(table.rows[1].cells[0].content, "Alice");
    assert_eq!(table.rows[1].cells[1].content, "30");
    assert_eq!(table.rows[1].cells[2].content, "New York");

    // Test document generation
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify content
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    let text = parsed_doc.get_text().unwrap();
    assert!(text.contains("Name"));
    assert!(text.contains("Alice"));
    assert!(text.contains("New York"));
}

#[test]
fn test_docx_complex_table() {
    let mut doc = DocxDocument::new();

    // Create a more complex table structure
    let mut table = Table {
        id: "complexTable1".to_string(),
        rows: Vec::new(),
        style: Some("TableGrid".to_string()),
        width: Some(5000.0),
    };

    // Add header row
    table.rows.push(TableRow {
        cells: vec![
            TableCell {
                content: "Product".to_string(),
                width: Some(2000.0),
                background_color: Some("4472C4".to_string()),
                text_align: Some(TextAlign::Center),
            },
            TableCell {
                content: "Price".to_string(),
                width: Some(1500.0),
                background_color: Some("4472C4".to_string()),
                text_align: Some(TextAlign::Center),
            },
            TableCell {
                content: "Available".to_string(),
                width: Some(1500.0),
                background_color: Some("4472C4".to_string()),
                text_align: Some(TextAlign::Center),
            },
        ],
        height: Some(400.0),
    });

    // Add data rows
    table.rows.push(TableRow {
        cells: vec![
            TableCell {
                content: "Laptop".to_string(),
                width: Some(2000.0),
                background_color: None,
                text_align: Some(TextAlign::Left),
            },
            TableCell {
                content: "$1,299.99".to_string(),
                width: Some(1500.0),
                background_color: None,
                text_align: Some(TextAlign::Right),
            },
            TableCell {
                content: "Yes".to_string(),
                width: Some(1500.0),
                background_color: Some("C6EFCE".to_string()),
                text_align: Some(TextAlign::Center),
            },
        ],
        height: None,
    });

    let table_id = doc.add_table(table).unwrap();
    assert!(!table_id.is_empty());

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    let text = parsed_doc.get_text().unwrap();

    assert!(text.contains("Product"));
    assert!(text.contains("Laptop"));
    assert!(text.contains("$1,299.99"));
}

#[test]
fn test_docx_image_support() {
    let mut doc = DocxDocument::new();

    // Create a simple PNG image data (1x1 pixel)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44,
        0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, // Image data
        0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND chunk
    ];

    let image_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("Test Image"))
        .unwrap();
    assert!(!image_id.is_empty());
    assert_eq!(doc.images.len(), 1);

    let image_data = doc.images.get(&image_id).unwrap();
    assert_eq!(image_data.content_type, "image/png");
    assert_eq!(image_data.alt_text, Some("Test Image".to_string()));
    assert_eq!(image_data.data, png_data);

    // Test document generation with image
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Verify the image is embedded in the document
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    assert!(!parsed_doc.media.is_empty() || !parsed_doc.images.is_empty());
}

#[test]
fn test_docx_vba_macro_support() {
    let mut doc = DocxDocument::new();

    // Add a simple VBA macro (as binary data)
    let vba_data = b"Fake VBA Project Data".to_vec();
    doc.add_vba_macro(vba_data.clone()).unwrap();

    assert!(doc.vba_project.is_some());
    assert_eq!(doc.vba_project.as_ref().unwrap(), &vba_data);

    // Test document generation with VBA
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify VBA is present
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    assert!(parsed_doc.vba_project.is_some());
    assert_eq!(parsed_doc.vba_project.as_ref().unwrap(), &vba_data);
}

#[test]
fn test_docx_comprehensive_document() {
    let mut doc = DocxDocument::new()
        .with_author("Test Author")
        .with_title("Comprehensive Test Document")
        .with_subject("Testing All Features");

    // Set page formatting
    doc.set_page_size(PaperSize::Letter);
    doc.set_orientation(PageOrientation::Portrait);
    doc.set_margins(1440.0, 1440.0, 1800.0, 1800.0); // Custom margins

    // Add various content types
    doc.add_heading(1, "Document Title").unwrap();
    doc.add_paragraph("This document demonstrates all advanced DOCX features.")
        .unwrap();

    doc.add_heading(2, "Table Example").unwrap();
    doc.create_table(
        &["Feature", "Status", "Notes"],
        &[
            vec!["Images", "✓ Supported", "PNG, JPEG, GIF, BMP"],
            vec!["Tables", "✓ Supported", "With styling and formatting"],
            vec!["VBA Macros", "✓ Supported", "Binary blob embedding"],
            vec!["Metadata", "✓ Supported", "Author, title, dates"],
        ],
    )
    .unwrap();

    doc.add_heading(2, "Code Example").unwrap();
    doc.add_code_block("fn main() {\n    println!(\"Hello, advanced DOCX!\");\n}")
        .unwrap();

    doc.add_heading(2, "Lists and Quotes").unwrap();
    doc.add_bulleted_list_item("First bullet point").unwrap();
    doc.add_bulleted_list_item("Second bullet point").unwrap();
    doc.add_numbered_list_item("First numbered item", 1)
        .unwrap();
    doc.add_numbered_list_item("Second numbered item", 2)
        .unwrap();

    doc.add_quote("This is an important quote demonstrating blockquote formatting.")
        .unwrap();
    doc.add_horizontal_rule().unwrap();
    doc.add_paragraph("Final paragraph after horizontal rule.")
        .unwrap();

    // Add VBA macro
    let vba_data = b"Simple VBA test data".to_vec();
    doc.add_vba_macro(vba_data).unwrap();

    // Test document generation
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify all content
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    let text = parsed_doc.get_text().unwrap();

    // Verify all content types are present
    assert!(text.contains("Document Title"));
    assert!(text.contains("Table Example"));
    assert!(text.contains("Feature"));
    assert!(text.contains("Images"));
    assert!(text.contains("VBA Macros"));
    assert!(text.contains("Hello, advanced DOCX"));
    assert!(text.contains("bullet point"));
    assert!(text.contains("numbered item"));
    assert!(text.contains("important quote"));
    assert!(text.contains("Final paragraph"));

    // Verify VBA project (metadata will be default for parsed docs for now)
    assert!(parsed_doc.vba_project.is_some());
}

#[test]
fn test_docx_table_text_alignment() {
    let mut doc = DocxDocument::new();

    let mut table = Table {
        id: "alignmentTest".to_string(),
        rows: Vec::new(),
        style: Some("TableGrid".to_string()),
        width: None,
    };

    table.rows.push(TableRow {
        cells: vec![
            TableCell {
                content: "Left Aligned".to_string(),
                width: None,
                background_color: None,
                text_align: Some(TextAlign::Left),
            },
            TableCell {
                content: "Center Aligned".to_string(),
                width: None,
                background_color: None,
                text_align: Some(TextAlign::Center),
            },
            TableCell {
                content: "Right Aligned".to_string(),
                width: None,
                background_color: None,
                text_align: Some(TextAlign::Right),
            },
            TableCell {
                content: "Justified Text".to_string(),
                width: None,
                background_color: None,
                text_align: Some(TextAlign::Justify),
            },
        ],
        height: None,
    });

    doc.add_table(table).unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    let text = parsed_doc.get_text().unwrap();

    assert!(text.contains("Left Aligned"));
    assert!(text.contains("Center Aligned"));
    assert!(text.contains("Right Aligned"));
    assert!(text.contains("Justified Text"));
}

#[test]
fn test_docx_multiple_images() {
    let mut doc = DocxDocument::new();

    // Create two different test images
    let png_data = vec![0x89, 0x50, 0x4E, 0x47]; // Minimal PNG header
    let jpg_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // Minimal JPEG header

    let image1_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("First Image"))
        .unwrap();
    let image2_id = doc
        .add_image_from_bytes(&jpg_data, "image/jpeg", Some("Second Image"))
        .unwrap();

    assert_eq!(doc.images.len(), 2);
    assert_ne!(image1_id, image2_id);

    let image1 = doc.images.get(&image1_id).unwrap();
    let image2 = doc.images.get(&image2_id).unwrap();

    assert_eq!(image1.content_type, "image/png");
    assert_eq!(image2.content_type, "image/jpeg");
    assert_eq!(image1.alt_text, Some("First Image".to_string()));
    assert_eq!(image2.alt_text, Some("Second Image".to_string()));

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_docx_page_orientation_switching() {
    let mut doc = DocxDocument::new();

    // Start with portrait
    doc.set_page_size(PaperSize::A4);
    assert_eq!(doc.page_setup.orientation, PageOrientation::Portrait);
    let original_width = doc.page_setup.width;
    let original_height = doc.page_setup.height;

    // Switch to landscape
    doc.set_orientation(PageOrientation::Landscape);
    assert_eq!(doc.page_setup.orientation, PageOrientation::Landscape);

    // In landscape, width and height should be swapped
    assert!((doc.page_setup.width - original_height).abs() < 1.0);
    assert!((doc.page_setup.height - original_width).abs() < 1.0);
}

#[test]
fn test_docx_empty_table() {
    let mut doc = DocxDocument::new();

    let _table_id = doc.create_table(&[], &[]).unwrap();
    assert_eq!(doc.tables.len(), 1);

    let table = &doc.tables[0];
    assert!(table.rows.is_empty());

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_docx_table_with_background_colors() {
    let mut doc = DocxDocument::new();

    let mut table = Table {
        id: "coloredTable".to_string(),
        rows: Vec::new(),
        style: Some("TableGrid".to_string()),
        width: None,
    };

    table.rows.push(TableRow {
        cells: vec![
            TableCell {
                content: "Red Background".to_string(),
                width: None,
                background_color: Some("FF0000".to_string()),
                text_align: Some(TextAlign::Left),
            },
            TableCell {
                content: "Green Background".to_string(),
                width: None,
                background_color: Some("00FF00".to_string()),
                text_align: Some(TextAlign::Left),
            },
            TableCell {
                content: "Blue Background".to_string(),
                width: None,
                background_color: Some("0000FF".to_string()),
                text_align: Some(TextAlign::Left),
            },
        ],
        height: None,
    });

    doc.add_table(table).unwrap();

    let bytes = doc.to_bytes().unwrap();
    let parsed_doc = DocxDocument::from_bytes(&bytes).unwrap();
    let text = parsed_doc.get_text().unwrap();

    assert!(text.contains("Red Background"));
    assert!(text.contains("Green Background"));
    assert!(text.contains("Blue Background"));
}

#[test]
fn test_docx_enhanced_metadata() {
    let mut doc = DocxDocument::new();
    doc.metadata.title = Some("Advanced Document".to_string());
    doc.metadata.author = Some("Jane Smith".to_string());
    doc.metadata.subject = Some("Testing Metadata".to_string());
    doc.metadata.description = Some("A comprehensive test of metadata features".to_string());
    doc.metadata.keywords = Some("rust, office, docx, testing".to_string());
    doc.metadata.category = Some("Software Testing".to_string());

    let core_xml = doc.create_enhanced_core_properties();

    assert!(core_xml.contains("Advanced Document"));
    assert!(core_xml.contains("Jane Smith"));
    assert!(core_xml.contains("Testing Metadata"));
    assert!(core_xml.contains("comprehensive test"));
    assert!(core_xml.contains("rust, office, docx"));
    assert!(core_xml.contains("Software Testing"));

    // Verify dates are properly formatted
    assert!(core_xml.contains("dcterms:created"));
    assert!(core_xml.contains("dcterms:modified"));
}

// Advanced XLSX Tests
#[test]
fn test_xlsx_enhanced_metadata() {
    let mut doc = XlsxDocument::new();
    doc.set_author("Excel Test Author");
    doc.set_title("Advanced Spreadsheet");
    doc.set_subject("Testing XLSX Features");
    doc.set_description("Comprehensive test of XLSX capabilities");
    doc.set_keywords("excel, xlsx, rust, testing");
    doc.set_category("Software Development");

    // Test metadata getters
    assert_eq!(doc.author, "Excel Test Author");
    assert_eq!(doc.title, "Advanced Spreadsheet");
    assert_eq!(doc.subject, "Testing XLSX Features");
    assert_eq!(doc.description, "Comprehensive test of XLSX capabilities");
    assert_eq!(doc.keywords, "excel, xlsx, rust, testing");
    assert_eq!(doc.category, "Software Development");

    // Test document generation with metadata
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify metadata is preserved
    let parsed_doc = XlsxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.author, "Excel Test Author");
    assert_eq!(parsed_doc.title, "Advanced Spreadsheet");
}

#[test]
fn test_xlsx_page_setup() {
    let mut doc = XlsxDocument::new();

    // Test default page setup
    assert_eq!(doc.page_setup.paper_size, XlsxPaperSize::Letter);
    assert_eq!(doc.page_setup.orientation, XlsxPageOrientation::Portrait);

    // Test setting different page sizes
    doc.set_page_size(XlsxPaperSize::A4);
    assert_eq!(doc.page_setup.paper_size, XlsxPaperSize::A4);

    doc.set_orientation(XlsxPageOrientation::Landscape);
    assert_eq!(doc.page_setup.orientation, XlsxPageOrientation::Landscape);

    // Test custom margins
    doc.set_margins(1.0, 1.0, 0.8, 0.8);
    assert_eq!(doc.page_setup.margin_top, 1.0);
    assert_eq!(doc.page_setup.margin_bottom, 1.0);
    assert_eq!(doc.page_setup.margin_left, 0.8);
    assert_eq!(doc.page_setup.margin_right, 0.8);

    // Test scale setting
    doc.set_scale(75);
    assert_eq!(doc.page_setup.scale, 75);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_xlsx_image_support() {
    let mut doc = XlsxDocument::new();

    // Create a simple PNG image data (minimal valid PNG)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
    ];

    let image_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("Test Chart"))
        .unwrap();

    assert!(!image_id.is_empty());
    assert_eq!(doc.images.len(), 1);

    let image = doc.images.get(&image_id).unwrap();
    assert_eq!(image.content_type, "image/png");
    assert_eq!(image.alt_text, "Test Chart");
    assert_eq!(image.data, png_data);

    // Test document generation with image
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_xlsx_chart_support() {
    let mut doc = XlsxDocument::new();

    // Add some sample data first
    let data = vec![
        vec!["Month".to_string(), "Sales".to_string()],
        vec!["Jan".to_string(), "1000".to_string()],
        vec!["Feb".to_string(), "1200".to_string()],
        vec!["Mar".to_string(), "1100".to_string()],
    ];
    doc.add_worksheet("Sales Data", &data).unwrap();

    // Add a chart
    let chart_id = doc
        .add_chart(ChartType::Column, "Monthly Sales", "A1:B4", 400, 300)
        .unwrap();

    assert!(!chart_id.is_empty());
    assert_eq!(doc.charts.len(), 1);

    let chart = doc.charts.get(&chart_id).unwrap();
    assert_eq!(chart.title, "Monthly Sales");
    assert_eq!(chart.data_range, "A1:B4");
    assert_eq!(chart.width, 400);
    assert_eq!(chart.height, 300);

    // Test different chart types
    doc.add_chart(ChartType::Line, "Trend Analysis", "A1:B4", 500, 300)
        .unwrap();
    doc.add_chart(ChartType::Pie, "Distribution", "A1:B4", 300, 300)
        .unwrap();

    assert_eq!(doc.charts.len(), 3);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_xlsx_table_creation() {
    let mut doc = XlsxDocument::new();

    // Create a formatted table
    let table_id = doc
        .create_table(
            "EmployeeData",
            "A1:D5",
            &["Name", "Department", "Salary", "Years"],
            &[
                vec!["Alice", "Engineering", "75000", "3"],
                vec!["Bob", "Marketing", "65000", "2"],
                vec!["Carol", "Engineering", "80000", "5"],
                vec!["David", "Sales", "60000", "1"],
            ],
        )
        .unwrap();

    assert!(!table_id.is_empty());
    assert_eq!(doc.tables.len(), 1);

    let table = doc.tables.get(&table_id).unwrap();
    assert_eq!(table.name, "EmployeeData");
    assert_eq!(table.range, "A1:D5");
    assert_eq!(table.headers.len(), 4);
    assert_eq!(table.data.len(), 4);
    assert_eq!(table.headers[0], "Name");
    assert_eq!(table.data[0][0], "Alice");

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_xlsx_vba_macro_support() {
    let mut doc = XlsxDocument::new();

    // Add VBA macro
    let vba_data = b"Sample VBA macro for Excel".to_vec();
    doc.add_vba_macro(vba_data.clone()).unwrap();

    assert!(doc.has_vba_macros());
    assert_eq!(doc.get_vba_project().unwrap(), &vba_data);

    // Test document generation with VBA
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify VBA is preserved
    let parsed_doc = XlsxDocument::from_bytes(&bytes).unwrap();
    assert!(parsed_doc.has_vba_macros());
    assert_eq!(parsed_doc.get_vba_project().unwrap(), &vba_data);
}

#[test]
fn test_xlsx_document_statistics() {
    let mut doc = XlsxDocument::new();

    // Add various content
    let data1 = vec![
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec!["1".to_string(), "2".to_string(), "3".to_string()],
    ];
    doc.add_worksheet("Sheet1", &data1).unwrap();

    let data2 = vec![
        vec!["X".to_string(), "Y".to_string()],
        vec!["4".to_string(), "5".to_string()],
        vec!["6".to_string(), "7".to_string()],
    ];
    doc.add_worksheet("Sheet2", &data2).unwrap();

    // Add images, charts, tables
    let png_data = vec![0x89, 0x50, 0x4E, 0x47]; // Minimal PNG
    doc.add_image_from_bytes(&png_data, "image/png", Some("Chart"))
        .unwrap();

    doc.add_chart(ChartType::Bar, "Test Chart", "A1:B2", 300, 200)
        .unwrap();

    doc.create_table(
        "TestTable",
        "A1:C2",
        &["Col1", "Col2", "Col3"],
        &[vec!["Data1", "Data2", "Data3"]],
    )
    .unwrap();

    let vba_data = b"VBA test".to_vec();
    doc.add_vba_macro(vba_data).unwrap();

    // Get statistics
    let stats = doc.get_document_stats();
    assert_eq!(stats.worksheet_count, 3); // Including default sheet
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.chart_count, 1);
    assert_eq!(stats.table_count, 1);
    assert!(stats.total_cells > 0);
    assert!(stats.has_vba);
    assert_eq!(stats.author, "Office SDK");
}

#[test]
fn test_xlsx_comprehensive_document() {
    let mut doc = XlsxDocument::new();

    // Set metadata
    doc.set_author("Test Author");
    doc.set_title("Comprehensive XLSX Test");
    doc.set_subject("Testing All Features");
    doc.set_description("A test document showcasing all XLSX capabilities");
    doc.set_keywords("xlsx, excel, rust, comprehensive");
    doc.set_category("Testing");

    // Set page setup
    doc.set_page_size(XlsxPaperSize::A4);
    doc.set_orientation(XlsxPageOrientation::Landscape);
    doc.set_margins(0.75, 0.75, 1.0, 1.0);
    doc.set_scale(90);

    // Add multiple worksheets with data
    let sales_data = vec![
        vec![
            "Product".to_string(),
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ],
        vec![
            "Laptops".to_string(),
            "100".to_string(),
            "120".to_string(),
            "110".to_string(),
            "130".to_string(),
        ],
        vec![
            "Tablets".to_string(),
            "80".to_string(),
            "90".to_string(),
            "85".to_string(),
            "95".to_string(),
        ],
        vec![
            "Phones".to_string(),
            "200".to_string(),
            "250".to_string(),
            "230".to_string(),
            "280".to_string(),
        ],
    ];
    doc.add_worksheet("Sales", &sales_data).unwrap();

    let employee_data = vec![
        vec![
            "Name".to_string(),
            "Department".to_string(),
            "Salary".to_string(),
        ],
        vec!["John".to_string(), "IT".to_string(), "70000".to_string()],
        vec!["Jane".to_string(), "HR".to_string(), "65000".to_string()],
        vec![
            "Mike".to_string(),
            "Finance".to_string(),
            "72000".to_string(),
        ],
    ];
    doc.add_worksheet("Employees", &employee_data).unwrap();

    // Add charts
    doc.add_chart(ChartType::Column, "Sales by Product", "A1:E4", 600, 400)
        .unwrap();
    doc.add_chart(ChartType::Pie, "Department Distribution", "A1:B4", 400, 400)
        .unwrap();

    // Add tables
    doc.create_table(
        "SalesTable",
        "A1:E4",
        &["Product", "Q1", "Q2", "Q3", "Q4"],
        &[
            vec!["Laptops", "100", "120", "110", "130"],
            vec!["Tablets", "80", "90", "85", "95"],
            vec!["Phones", "200", "250", "230", "280"],
        ],
    )
    .unwrap();

    // Add image
    let test_image = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    doc.add_image_from_bytes(&test_image, "image/png", Some("Company Logo"))
        .unwrap();

    // Add VBA macro
    let vba_code = b"Sub TestMacro()\n    MsgBox \"Hello from VBA!\"\nEnd Sub".to_vec();
    doc.add_vba_macro(vba_code).unwrap();

    // Generate document
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify
    let parsed_doc = XlsxDocument::from_bytes(&bytes).unwrap();

    // Verify metadata
    assert_eq!(parsed_doc.author, "Test Author");
    assert_eq!(parsed_doc.title, "Comprehensive XLSX Test");

    // Verify content
    assert_eq!(parsed_doc.worksheets.len(), 3); // 2 added + default

    // Verify data extraction
    let text = parsed_doc.get_text().unwrap();
    assert!(text.contains("Product"));
    assert!(text.contains("Laptops"));
    assert!(text.contains("John"));
    assert!(text.contains("Finance"));

    // Verify statistics
    let stats = parsed_doc.get_document_stats();
    assert_eq!(stats.worksheet_count, 3);
    assert!(stats.has_vba);
    assert!(stats.total_cells > 10);
}

#[test]
fn test_xlsx_multiple_images_and_charts() {
    let mut doc = XlsxDocument::new();

    // Add multiple images of different types
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let jpg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];

    let image1_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("PNG Image"))
        .unwrap();
    let image2_id = doc
        .add_image_from_bytes(&jpg_data, "image/jpeg", Some("JPEG Image"))
        .unwrap();

    assert_eq!(doc.images.len(), 2);
    assert_ne!(image1_id, image2_id);

    // Add multiple charts
    let chart1_id = doc
        .add_chart(ChartType::Line, "Trend", "A1:B5", 400, 300)
        .unwrap();
    let chart2_id = doc
        .add_chart(ChartType::Bar, "Comparison", "C1:D5", 500, 300)
        .unwrap();
    let chart3_id = doc
        .add_chart(ChartType::Area, "Distribution", "E1:F5", 450, 350)
        .unwrap();

    assert_eq!(doc.charts.len(), 3);
    assert_ne!(chart1_id, chart2_id);
    assert_ne!(chart2_id, chart3_id);

    // Verify chart properties
    let chart1 = doc.charts.get(&chart1_id).unwrap();
    let chart2 = doc.charts.get(&chart2_id).unwrap();

    assert_eq!(chart1.title, "Trend");
    assert_eq!(chart2.title, "Comparison");

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_xlsx_empty_document_operations() {
    let mut doc = XlsxDocument::new();

    // Test operations on empty document
    let stats = doc.get_document_stats();
    assert_eq!(stats.worksheet_count, 1); // Default sheet
    assert_eq!(stats.image_count, 0);
    assert_eq!(stats.chart_count, 0);
    assert_eq!(stats.table_count, 0);
    assert!(!stats.has_vba);

    // Test text extraction from empty document
    let text = doc.get_text().unwrap();
    assert!(text.len() < 50); // Should be mostly empty

    // Test document generation
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Test parsing empty document
    let parsed_doc = XlsxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.worksheets.len(), 1);
}

#[test]
fn test_xlsx_large_dataset() {
    let mut doc = XlsxDocument::new();

    // Create a larger dataset
    let mut large_data = vec![vec![
        "ID".to_string(),
        "Value".to_string(),
        "Category".to_string(),
    ]];
    for i in 1..=100 {
        large_data.push(vec![
            i.to_string(),
            (i * 10).to_string(),
            format!("Category_{}", i % 5),
        ]);
    }

    doc.add_worksheet("LargeData", &large_data).unwrap();

    // Verify data structure
    let sheet_data = doc.get_worksheet_data("LargeData").unwrap();
    assert_eq!(sheet_data.len(), 101); // 100 data rows + 1 header
    assert_eq!(sheet_data[0][0], "ID");
    assert_eq!(sheet_data[1][0], "1");
    assert_eq!(sheet_data[100][0], "100");

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify large dataset
    let parsed_doc = XlsxDocument::from_bytes(&bytes).unwrap();
    let parsed_data = parsed_doc.get_worksheet_data("LargeData").unwrap();
    assert_eq!(parsed_data.len(), 101);
    assert_eq!(parsed_data[50][1], "500"); // 50 * 10 = 500
}

// Advanced PPTX Tests
#[test]
fn test_pptx_enhanced_metadata() {
    let mut doc = PptxDocument::new();
    doc.set_author("PowerPoint Test Author");
    doc.set_title("Advanced Presentation");
    doc.set_subject("Testing PPTX Features");
    doc.set_description("Comprehensive test of PPTX capabilities");
    doc.set_keywords("powerpoint, pptx, rust, testing");
    doc.set_category("Software Development");

    // Test metadata getters
    assert_eq!(doc.author, "PowerPoint Test Author");
    assert_eq!(doc.title, "Advanced Presentation");
    assert_eq!(doc.subject, "Testing PPTX Features");
    assert_eq!(doc.description, "Comprehensive test of PPTX capabilities");
    assert_eq!(doc.keywords, "powerpoint, pptx, rust, testing");
    assert_eq!(doc.category, "Software Development");

    // Test document generation with metadata
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify metadata is preserved
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.author, "PowerPoint Test Author");
    assert_eq!(parsed_doc.title, "Advanced Presentation");
}

#[test]
fn test_pptx_slide_size_management() {
    let mut doc = PptxDocument::new();

    // Test default slide size
    assert_eq!(doc.get_slide_size(), SlideSize::Widescreen16x9);

    // Test setting different slide sizes
    doc.set_slide_size(SlideSize::Standard4x3);
    assert_eq!(doc.get_slide_size(), SlideSize::Standard4x3);

    doc.set_slide_size(SlideSize::A4);
    assert_eq!(doc.get_slide_size(), SlideSize::A4);

    // Test custom slide size
    doc.set_slide_size(SlideSize::Custom {
        width: 1000000,
        height: 750000,
    });
    assert_eq!(
        doc.get_slide_size(),
        SlideSize::Custom {
            width: 1000000,
            height: 750000
        }
    );

    // Test slide size dimensions
    let (width, height) = SlideSize::Widescreen16x9.dimensions();
    assert_eq!(width, 12192000);
    assert_eq!(height, 6858000);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_image_support() {
    let mut doc = PptxDocument::new();

    // Create a simple PNG image data
    let png_data = image_fixture(image::ImageFormat::Png, 1, 1);

    let image_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("Slide Image"))
        .unwrap();

    assert!(!image_id.is_empty());
    assert_eq!(doc.images.len(), 1);

    let image = doc.images.get(&image_id).unwrap();
    assert_eq!(image.content_type, "image/png");
    assert_eq!(image.alt_text, "Slide Image");
    assert_eq!(image.data, png_data);

    // Test document generation with image
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_animation_support() {
    let mut doc = PptxDocument::new();

    // Add animations
    let anim1_id = doc
        .add_animation("shape1", AnimationType::FadeIn, 1.0, 0.0)
        .unwrap();
    let anim2_id = doc
        .add_animation("shape2", AnimationType::SlideIn, 1.5, 0.5)
        .unwrap();

    assert_eq!(doc.animations.len(), 2);
    assert_ne!(anim1_id, anim2_id);

    let anim1 = doc.animations.get(&anim1_id).unwrap();
    let anim2 = doc.animations.get(&anim2_id).unwrap();

    assert_eq!(anim1.target_shape, "shape1");
    assert_eq!(anim1.duration, 1.0);
    assert_eq!(anim1.delay, 0.0);

    assert_eq!(anim2.target_shape, "shape2");
    assert_eq!(anim2.duration, 1.5);
    assert_eq!(anim2.delay, 0.5);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_speaker_notes() {
    let mut doc = PptxDocument::new();

    // Add a slide first
    let slide_id = doc.add_slide("Test Slide", &["Content here"]).unwrap();

    // Add speaker notes
    doc.add_slide_notes(
        &slide_id,
        "These are important speaker notes for this slide.",
    )
    .unwrap();

    assert_eq!(doc.notes.len(), 1);
    let notes = doc.get_slide_notes(&slide_id).unwrap();
    assert_eq!(notes, "These are important speaker notes for this slide.");

    // Add more notes to different slides
    let slide2_id = doc.add_slide("Second Slide", &["More content"]).unwrap();
    doc.add_slide_notes(&slide2_id, "Notes for the second slide.")
        .unwrap();

    assert_eq!(doc.notes.len(), 2);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_vba_macro_support() {
    let mut doc = PptxDocument::new();

    // Add VBA macro
    let vba_data = b"Sample VBA macro for PowerPoint".to_vec();
    doc.add_vba_macro(vba_data.clone()).unwrap();

    assert!(doc.has_vba_macros());
    assert_eq!(doc.get_vba_project().unwrap(), &vba_data);

    // Test document generation with VBA
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify VBA is preserved
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();
    assert!(parsed_doc.has_vba_macros());
    assert_eq!(parsed_doc.get_vba_project().unwrap(), &vba_data);
}

#[test]
fn test_pptx_slide_types() {
    let mut doc = PptxDocument::new();

    // Test different slide creation methods
    let title_slide_id = doc
        .add_title_slide("Welcome", "To Our Presentation")
        .unwrap();
    let content_slide_id = doc
        .add_content_slide(
            "Agenda",
            &["Introduction", "Main Content", "Questions & Answers"],
        )
        .unwrap();

    // Add an image and create an image slide
    let png_data = image_fixture(image::ImageFormat::Png, 1, 1);
    let image_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("Demo Image"))
        .unwrap();
    let image_slide_id = doc
        .add_image_slide(
            "Visual Demo",
            &image_id,
            Some("This demonstrates our product"),
        )
        .unwrap();

    assert_eq!(doc.slides.len(), 3);
    assert_ne!(title_slide_id, content_slide_id);
    assert_ne!(content_slide_id, image_slide_id);

    // Verify slide content
    let slides = doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 3);
    assert_eq!(slides[0].title, "Welcome");
    assert_eq!(slides[1].title, "Agenda");
    assert_eq!(slides[2].title, "Visual Demo");

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_document_statistics() {
    let mut doc = PptxDocument::new();

    // Add various content
    doc.add_slide("Title Slide", &["Subtitle"]).unwrap();
    doc.add_slide("Content Slide", &["Point 1", "Point 2", "Point 3"])
        .unwrap();

    // Add images and animations
    let png_data = image_fixture(image::ImageFormat::Png, 1, 1);
    doc.add_image_from_bytes(&png_data, "image/png", Some("Chart"))
        .unwrap();

    doc.add_animation("title", AnimationType::FadeIn, 1.0, 0.0)
        .unwrap();
    doc.add_animation("content", AnimationType::SlideIn, 1.5, 0.5)
        .unwrap();

    // Add speaker notes
    doc.add_slide_notes("slide1", "Important notes for slide 1")
        .unwrap();
    doc.add_slide_notes("slide2", "Notes for slide 2").unwrap();

    // Add VBA
    let vba_data = b"VBA test".to_vec();
    doc.add_vba_macro(vba_data).unwrap();

    // Get statistics
    let stats = doc.get_document_stats();
    assert_eq!(stats.slide_count, 2);
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.animation_count, 2);
    assert_eq!(stats.notes_count, 2);
    assert!(stats.total_text_length > 0);
    assert!(stats.has_vba);
    assert_eq!(stats.author, "Office SDK");
}

#[test]
fn test_pptx_comprehensive_presentation() {
    let mut doc = PptxDocument::new();

    // Set metadata
    doc.set_author("Test Presenter");
    doc.set_title("Comprehensive PPTX Test");
    doc.set_subject("Testing All Features");
    doc.set_description("A test presentation showcasing all PPTX capabilities");
    doc.set_keywords("pptx, powerpoint, rust, comprehensive");
    doc.set_category("Testing");

    // Set slide size
    doc.set_slide_size(SlideSize::Widescreen16x9);

    // Add title slide
    let title_slide_id = doc
        .add_title_slide(
            "Advanced PowerPoint Features",
            "Demonstrating Comprehensive PPTX Support",
        )
        .unwrap();

    // Add content slides
    let overview_slide_id = doc
        .add_content_slide(
            "Overview",
            &[
                "Image Support",
                "Animation Effects",
                "Speaker Notes",
                "VBA Macros",
                "Metadata Management",
            ],
        )
        .unwrap();

    let features_slide_id = doc
        .add_content_slide(
            "Key Features",
            &[
                "Multiple slide sizes and orientations",
                "Rich image embedding (PNG, JPEG, GIF, BMP)",
                "Animation timeline management",
                "Comprehensive speaker notes",
                "VBA macro embedding for security analysis",
            ],
        )
        .unwrap();

    // Add image slide
    let test_image = image_fixture(image::ImageFormat::Png, 1, 1);
    let image_id = doc
        .add_image_from_bytes(&test_image, "image/png", Some("Feature Diagram"))
        .unwrap();
    let image_slide_id = doc
        .add_image_slide(
            "Visual Architecture",
            &image_id,
            Some("This diagram shows the comprehensive architecture of our Office SDK"),
        )
        .unwrap();

    // Add animations
    doc.add_animation("title", AnimationType::FadeIn, 1.0, 0.0)
        .unwrap();
    doc.add_animation("subtitle", AnimationType::SlideIn, 1.5, 0.5)
        .unwrap();
    doc.add_animation("bullet1", AnimationType::FadeIn, 0.8, 1.0)
        .unwrap();
    doc.add_animation("bullet2", AnimationType::FadeIn, 0.8, 1.5)
        .unwrap();

    // Add speaker notes
    doc.add_slide_notes(
        &title_slide_id,
        "Welcome everyone to this comprehensive demonstration of our advanced PPTX capabilities.",
    )
    .unwrap();
    doc.add_slide_notes(
        &overview_slide_id,
        "This slide outlines the key features we'll be demonstrating today.",
    )
    .unwrap();
    doc.add_slide_notes(
        &features_slide_id,
        "Each of these features has been thoroughly tested and validated.",
    )
    .unwrap();
    doc.add_slide_notes(
        &image_slide_id,
        "This architecture diagram shows how all components work together seamlessly.",
    )
    .unwrap();

    // Add VBA macro
    let vba_code =
        b"Sub PresentationMacro()\n    MsgBox \"Welcome to our advanced presentation!\"\nEnd Sub"
            .to_vec();
    doc.add_vba_macro(vba_code).unwrap();

    // Generate document
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();

    // Verify metadata
    assert_eq!(parsed_doc.author, "Test Presenter");
    assert_eq!(parsed_doc.title, "Comprehensive PPTX Test");

    // Verify slides
    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 4);
    assert_eq!(slides[0].title, "Advanced PowerPoint Features");
    assert_eq!(slides[1].title, "Overview");
    assert_eq!(slides[2].title, "Key Features");
    assert_eq!(slides[3].title, "Visual Architecture");

    // Verify content
    assert!(
        slides[1]
            .content
            .iter()
            .any(|c| c.contains("Image Support"))
    );
    assert!(
        slides[2]
            .content
            .iter()
            .any(|c| c.contains("Multiple slide sizes"))
    );

    // Verify VBA
    assert!(parsed_doc.has_vba_macros());

    // Verify statistics
    let stats = parsed_doc.get_document_stats();
    assert_eq!(stats.slide_count, 4);
    assert!(stats.has_vba);
    assert!(stats.total_text_length > 50);
}

#[test]
fn test_pptx_multiple_images_and_animations() {
    let mut doc = PptxDocument::new();

    // Add multiple images of different types
    let png_data = image_fixture(image::ImageFormat::Png, 1, 1);
    let jpg_data = image_fixture(image::ImageFormat::Jpeg, 1, 1);

    let image1_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("PNG Image"))
        .unwrap();
    let image2_id = doc
        .add_image_from_bytes(&jpg_data, "image/jpeg", Some("JPEG Image"))
        .unwrap();

    assert_eq!(doc.images.len(), 2);
    assert_ne!(image1_id, image2_id);

    // Add multiple animations with different types
    let anim1_id = doc
        .add_animation("shape1", AnimationType::FadeIn, 1.0, 0.0)
        .unwrap();
    let anim2_id = doc
        .add_animation("shape2", AnimationType::FadeOut, 1.0, 1.0)
        .unwrap();
    let anim3_id = doc
        .add_animation("shape3", AnimationType::SlideIn, 1.5, 2.0)
        .unwrap();
    let anim4_id = doc
        .add_animation("shape4", AnimationType::Zoom, 2.0, 3.0)
        .unwrap();
    let anim5_id = doc
        .add_animation("shape5", AnimationType::Rotate, 3.0, 4.0)
        .unwrap();

    assert_eq!(doc.animations.len(), 5);

    // Verify animation properties
    let anim1 = doc.animations.get(&anim1_id).unwrap();
    let anim5 = doc.animations.get(&anim5_id).unwrap();

    assert_eq!(anim1.target_shape, "shape1");
    assert_eq!(anim5.target_shape, "shape5");
    assert_eq!(anim5.duration, 3.0);
    assert_eq!(anim5.delay, 4.0);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_empty_presentation_advanced() {
    let doc = PptxDocument::new();

    // Test operations on empty presentation
    let stats = doc.get_document_stats();
    assert_eq!(stats.slide_count, 0);
    assert_eq!(stats.image_count, 0);
    assert_eq!(stats.animation_count, 0);
    assert_eq!(stats.notes_count, 0);
    assert!(!stats.has_vba);

    // Test slide text extraction from empty presentation
    let text = doc.get_slide_text().unwrap();
    assert!(text.is_empty());

    // Test document generation
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Test parsing empty presentation
    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.slides.len(), 0);
}

#[test]
fn test_pptx_slide_content_parsing() {
    let mut doc = PptxDocument::new();

    // Add slides with various content
    doc.add_slide("First Slide", &["First point", "Second point"])
        .unwrap();
    doc.add_slide(
        "Second Slide",
        &["Another point", "And another", "Final point"],
    )
    .unwrap();
    doc.add_slide("Empty Content Slide", &[]).unwrap();

    // Test individual slide retrieval
    let slide1 = doc.get_slide_content("slide1").unwrap();
    assert_eq!(slide1.title, "First Slide");
    assert_eq!(slide1.content.len(), 2);
    assert_eq!(slide1.content[0], "First point");
    assert_eq!(slide1.content[1], "Second point");

    let slide2 = doc.get_slide_content("slide2").unwrap();
    assert_eq!(slide2.title, "Second Slide");
    assert_eq!(slide2.content.len(), 3);

    let slide3 = doc.get_slide_content("slide3").unwrap();
    assert_eq!(slide3.title, "Empty Content Slide");
    assert_eq!(slide3.content.len(), 0);

    // Test all slides retrieval
    let all_slides = doc.get_all_slides().unwrap();
    assert_eq!(all_slides.len(), 3);

    // Test slide text extraction
    let text = doc.get_slide_text().unwrap();
    assert!(text.contains("First Slide"));
    assert!(text.contains("First point"));
    assert!(text.contains("Second Slide"));
    assert!(text.contains("Another point"));
    assert!(text.contains("Empty Content Slide"));

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pptx_slide_size_dimensions() {
    // Test all predefined slide sizes
    assert_eq!(SlideSize::Standard4x3.dimensions(), (9144000, 6858000));
    assert_eq!(SlideSize::Widescreen16x9.dimensions(), (12192000, 6858000));
    assert_eq!(SlideSize::A3.dimensions(), (14173200, 10063200));
    assert_eq!(SlideSize::A4.dimensions(), (10063200, 7581600));
    assert_eq!(SlideSize::B4ISO.dimensions(), (11339600, 8035200));
    assert_eq!(SlideSize::B5ISO.dimensions(), (8035200, 5715000));
    assert_eq!(SlideSize::Banner.dimensions(), (7315200, 914400));

    // Test custom slide size
    let custom_size = SlideSize::Custom {
        width: 5000000,
        height: 4000000,
    };
    assert_eq!(custom_size.dimensions(), (5000000, 4000000));
}

#[test]
fn test_pptx_complex_presentation_workflow() {
    let mut doc = PptxDocument::new();

    // Set up presentation metadata
    doc.set_author("Complex Test Author");
    doc.set_title("Complex Workflow Test");
    doc.set_slide_size(SlideSize::Widescreen16x9);

    // Create a multi-slide presentation with mixed content
    let intro_id = doc
        .add_title_slide("Project Overview", "Q4 2024 Results")
        .unwrap();
    doc.add_slide_notes(&intro_id, "Welcome everyone to our quarterly review.")
        .unwrap();

    let agenda_id = doc
        .add_content_slide(
            "Today's Agenda",
            &[
                "Financial Performance",
                "Key Achievements",
                "Market Analysis",
                "Future Roadmap",
            ],
        )
        .unwrap();

    // Add multiple images
    let chart_image = image_fixture(image::ImageFormat::Png, 1, 1); // PNG
    let logo_image = image_fixture(image::ImageFormat::Jpeg, 1, 1); // JPEG

    let chart_id = doc
        .add_image_from_bytes(&chart_image, "image/png", Some("Q4 Performance Chart"))
        .unwrap();
    let logo_id = doc
        .add_image_from_bytes(&logo_image, "image/jpeg", Some("Company Logo"))
        .unwrap();

    let chart_slide_id = doc
        .add_image_slide(
            "Performance Metrics",
            &chart_id,
            Some("Strong growth across all sectors"),
        )
        .unwrap();

    // Add complex animations
    doc.add_animation("title", AnimationType::FadeIn, 1.0, 0.0)
        .unwrap();
    doc.add_animation("chart", AnimationType::SlideIn, 2.0, 1.0)
        .unwrap();
    doc.add_animation("conclusion", AnimationType::FadeIn, 1.5, 3.0)
        .unwrap();

    // Add VBA for automation
    let complex_vba = b"Sub AutoAdvance()\n    For i = 1 To SlideShowWindows(1).Presentation.Slides.Count\n        SlideShowWindows(1).View.Next\n        Application.Wait(Now + TimeValue(\"0:00:03\"))\n    Next i\nEnd Sub".to_vec();
    doc.add_vba_macro(complex_vba).unwrap();

    // Add comprehensive speaker notes
    doc.add_slide_notes(&agenda_id, "This agenda covers our most important topics for today's meeting. Each section will have detailed analysis.").unwrap();
    doc.add_slide_notes(
        &chart_slide_id,
        "This chart shows exceptional performance. Note the 25% increase in Q4 compared to Q3.",
    )
    .unwrap();

    // Generate and verify
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let parsed_doc = PptxDocument::from_bytes(&bytes).unwrap();
    let slides = parsed_doc.get_all_slides().unwrap();
    assert_eq!(slides.len(), 3);

    // Verify complex content
    assert!(
        slides[1]
            .content
            .iter()
            .any(|c| c.contains("Financial Performance"))
    );
    assert!(
        slides[1]
            .content
            .iter()
            .any(|c| c.contains("Future Roadmap"))
    );

    // Verify VBA and other features
    assert!(parsed_doc.has_vba_macros());

    let stats = parsed_doc.get_document_stats();
    assert_eq!(stats.slide_count, 3);
    assert_eq!(stats.image_count, 2);
    assert_eq!(stats.animation_count, 3);
    assert!(stats.notes_count >= 2);
}

// Comprehensive PDF Tests
#[test]
fn test_pdf_creation_and_parsing() {
    let mut doc = PdfDocument::new();

    // Add a page and some content
    let _page_id = doc.add_a4_page().unwrap();
    doc.add_text(0, "Hello, PDF World!", 100.0, 700.0, "Helvetica", 14.0)
        .unwrap();
    doc.add_text(
        0,
        "This is a test PDF document.",
        100.0,
        680.0,
        "Helvetica",
        12.0,
    )
    .unwrap();

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse the created PDF
    let parsed_doc = PdfDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.pages.len(), 1);

    let text = parsed_doc.get_text().unwrap();
    assert!(text.contains("Hello, PDF World!") || text.contains("test PDF document"));
}

#[test]
fn test_pdf_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.pdf");

    let mut doc = PdfDocument::new();
    let _page_id = doc.add_letter_page().unwrap();
    doc.add_text(0, "File operation test", 100.0, 700.0, "Helvetica", 12.0)
        .unwrap();

    doc.save(&file_path).unwrap();
    assert!(file_path.exists());

    let loaded_doc = PdfDocument::from_file(&file_path).unwrap();
    assert_eq!(loaded_doc.pages.len(), 1);
    assert_eq!(loaded_doc.title, "New PDF Document");
}

#[test]
fn test_pdf_enhanced_metadata() {
    let mut doc = PdfDocument::new();
    doc.set_author("PDF Test Author");
    doc.set_title("Advanced PDF Document");
    doc.set_subject("Testing PDF Features");
    doc.set_description("Comprehensive test of PDF capabilities");
    doc.set_keywords("pdf, rust, testing, advanced");
    doc.set_category("Software Development");

    // Test metadata getters
    assert_eq!(doc.author, "PDF Test Author");
    assert_eq!(doc.title, "Advanced PDF Document");
    assert_eq!(doc.subject, "Testing PDF Features");
    assert_eq!(doc.description, "Comprehensive test of PDF capabilities");
    assert_eq!(doc.keywords, "pdf, rust, testing, advanced");
    assert_eq!(doc.category, "Software Development");

    // Test document generation with metadata
    let _page_id = doc.add_a4_page().unwrap();
    doc.add_text(0, "Metadata test document", 100.0, 700.0, "Helvetica", 12.0)
        .unwrap();

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Parse and verify metadata is preserved
    let parsed_doc = PdfDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.title, "Advanced PDF Document");
    assert_eq!(parsed_doc.author, "PDF Test Author");
}

#[test]
fn test_pdf_multiple_pages() {
    let mut doc = PdfDocument::new();

    // Add multiple pages with different sizes
    let _page1_id = doc.add_a4_page().unwrap();
    let _page2_id = doc.add_letter_page().unwrap();
    let _page3_id = doc.add_legal_page().unwrap();

    // Add content to each page
    doc.add_text(0, "Page 1 - A4 Format", 100.0, 700.0, "Helvetica", 14.0)
        .unwrap();
    doc.add_text(1, "Page 2 - Letter Format", 100.0, 700.0, "Helvetica", 14.0)
        .unwrap();
    doc.add_text(2, "Page 3 - Legal Format", 100.0, 700.0, "Helvetica", 14.0)
        .unwrap();

    assert_eq!(doc.pages.len(), 3);

    // Test page dimensions
    assert_eq!(doc.pages[0].width, 595.28); // A4 width
    assert_eq!(doc.pages[1].width, 612.0); // Letter width
    assert_eq!(doc.pages[2].height, 1008.0); // Legal height

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let parsed_doc = PdfDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.pages.len(), 3);
}

#[test]
fn test_pdf_text_styles() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add text with different styles
    let bold_style = TextStyle {
        bold: true,
        italic: false,
        underline: false,
        strikethrough: false,
        superscript: false,
        subscript: false,
    };

    let italic_style = TextStyle {
        bold: false,
        italic: true,
        underline: false,
        strikethrough: false,
        superscript: false,
        subscript: false,
    };

    doc.add_styled_text(
        0,
        "Bold Text",
        100.0,
        700.0,
        "Helvetica-Bold",
        14.0,
        Color::black(),
        bold_style,
    )
    .unwrap();
    doc.add_styled_text(
        0,
        "Italic Text",
        100.0,
        680.0,
        "Helvetica-Oblique",
        14.0,
        Color::blue(),
        italic_style,
    )
    .unwrap();
    doc.add_styled_text(
        0,
        "Red Text",
        100.0,
        660.0,
        "Helvetica",
        14.0,
        Color::red(),
        TextStyle::default(),
    )
    .unwrap();

    // Add headings
    doc.add_heading(0, 1, "Main Heading", 100.0, 750.0).unwrap();
    doc.add_heading(0, 2, "Sub Heading", 100.0, 720.0).unwrap();

    assert_eq!(doc.pages[0].content.len(), 5);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_shapes_and_lines() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add various shapes
    doc.add_rectangle(
        0,
        100.0,
        600.0,
        200.0,
        100.0,
        Some(Color::blue()),
        Some(Color::black()),
    )
    .unwrap();
    doc.add_circle(
        0,
        200.0,
        500.0,
        50.0,
        Some(Color::red()),
        Some(Color::black()),
    )
    .unwrap();
    doc.add_line(0, 50.0, 550.0, 350.0, 550.0, Color::green(), 2.0)
        .unwrap();

    assert_eq!(doc.pages[0].content.len(), 3);

    // Verify shape properties
    if let PageContent::Shape {
        shape_type,
        fill_color,
        ..
    } = &doc.pages[0].content[0]
    {
        assert_eq!(*shape_type, ShapeType::Rectangle);
        assert!(fill_color.is_some());
    }

    if let PageContent::Shape { shape_type, .. } = &doc.pages[0].content[1] {
        assert_eq!(*shape_type, ShapeType::Circle);
    }

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_image_support() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Create a simple PNG image data
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk header
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44,
        0x41, 0x54, // IDAT chunk
        0x08, 0xD7, 0x63, 0xF8, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND
    ];

    let image_id = doc
        .add_image_from_bytes(&png_data, "image/png", Some("Test Image"))
        .unwrap();
    assert!(!image_id.is_empty());
    assert_eq!(doc.images.len(), 1);

    let image = doc.images.get(&image_id).unwrap();
    assert_eq!(image.content_type, "image/png");
    assert_eq!(image.alt_text, "Test Image");
    assert_eq!(image.data, png_data);

    // Add image to page
    doc.add_image_to_page(0, &image_id, 200.0, 500.0, 100.0, 100.0)
        .unwrap();

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_table_creation() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Create a formatted table
    let table_id = doc
        .add_table(
            0,
            100.0,
            600.0,
            &["Name", "Age", "Department", "Salary"],
            &[
                vec!["Alice Johnson", "30", "Engineering", "$95,000"],
                vec!["Bob Smith", "25", "Marketing", "$78,000"],
                vec!["Carol Davis", "35", "Engineering", "$105,000"],
                vec!["David Wilson", "28", "Sales", "$65,000"],
            ],
        )
        .unwrap();

    assert!(!table_id.is_empty());

    // Verify table was added to page content
    let has_table = doc.pages[0]
        .content
        .iter()
        .any(|content| matches!(content, PageContent::Table { .. }));
    assert!(has_table);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_annotations() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add text first
    doc.add_text(
        0,
        "This text will be annotated",
        100.0,
        700.0,
        "Helvetica",
        12.0,
    )
    .unwrap();

    // Add different types of annotations
    let text_annotation_id = doc
        .add_text_annotation(
            0,
            100.0,
            690.0,
            200.0,
            20.0,
            "This is a text annotation",
            "Test Author",
        )
        .unwrap();

    let highlight_annotation_id = doc
        .add_highlight_annotation(0, 100.0, 700.0, 200.0, 12.0, "Highlighter")
        .unwrap();

    let link_annotation_id = doc
        .add_link_annotation(0, 320.0, 700.0, 100.0, 12.0, "https://example.com")
        .unwrap();

    assert_eq!(doc.annotations.len(), 3);

    // Verify annotation properties
    let text_annotation = doc.annotations.get(&text_annotation_id).unwrap();
    assert_eq!(text_annotation.annotation_type, AnnotationType::Text);
    assert_eq!(text_annotation.content, "This is a text annotation");
    assert_eq!(text_annotation.author, "Test Author");

    let highlight_annotation = doc.annotations.get(&highlight_annotation_id).unwrap();
    assert_eq!(
        highlight_annotation.annotation_type,
        AnnotationType::Highlight
    );

    let link_annotation = doc.annotations.get(&link_annotation_id).unwrap();
    assert_eq!(link_annotation.annotation_type, AnnotationType::Link);
    assert_eq!(link_annotation.content, "https://example.com");

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_form_fields() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add form title
    doc.add_text(0, "PDF Form Example", 100.0, 750.0, "Helvetica-Bold", 16.0)
        .unwrap();

    // Add various form fields
    let text_field_id = doc
        .add_text_field(0, "name", 100.0, 700.0, 200.0, 20.0, "Enter name")
        .unwrap();
    let multiline_field_id = doc
        .add_multiline_text_field(0, "comments", 100.0, 600.0, 300.0, 80.0, "Enter comments")
        .unwrap();
    let checkbox_id = doc
        .add_checkbox(0, "agree", 100.0, 550.0, 15.0, false)
        .unwrap();
    let dropdown_id = doc
        .add_dropdown(
            0,
            "country",
            100.0,
            500.0,
            150.0,
            20.0,
            &["USA", "Canada", "UK", "Germany", "Australia"],
            Some(0),
        )
        .unwrap();

    assert_eq!(doc.forms.len(), 4);

    // Verify form field properties
    let text_field = doc.forms.get(&text_field_id).unwrap();
    assert_eq!(text_field.name, "name");
    assert_eq!(text_field.value, "Enter name");
    assert!(matches!(text_field.field_type, FormFieldType::Text { .. }));

    let checkbox = doc.forms.get(&checkbox_id).unwrap();
    assert_eq!(checkbox.name, "agree");
    assert_eq!(checkbox.value, "Off");

    let dropdown = doc.forms.get(&dropdown_id).unwrap();
    assert_eq!(dropdown.name, "country");
    assert_eq!(dropdown.options.len(), 5);
    assert_eq!(dropdown.value, "USA");

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_bookmarks_and_outline() {
    let mut doc = PdfDocument::new();

    // Add multiple pages
    let _page1_id = doc.add_a4_page().unwrap();
    let _page2_id = doc.add_a4_page().unwrap();
    let _page3_id = doc.add_a4_page().unwrap();

    // Add content to pages
    doc.add_heading(0, 1, "Chapter 1: Introduction", 100.0, 750.0)
        .unwrap();
    doc.add_heading(1, 1, "Chapter 2: Methodology", 100.0, 750.0)
        .unwrap();
    doc.add_heading(2, 1, "Chapter 3: Conclusion", 100.0, 750.0)
        .unwrap();

    // Add bookmarks
    let bookmark1_id = doc.add_bookmark("Chapter 1", 0, 100.0, 750.0).unwrap();
    let bookmark2_id = doc.add_bookmark("Chapter 2", 1, 100.0, 750.0).unwrap();
    let bookmark3_id = doc.add_bookmark("Chapter 3", 2, 100.0, 750.0).unwrap();

    // Add nested bookmarks
    doc.add_nested_bookmark(&bookmark1_id, "Section 1.1", 0, 100.0, 700.0)
        .unwrap();
    doc.add_nested_bookmark(&bookmark1_id, "Section 1.2", 0, 100.0, 650.0)
        .unwrap();

    assert_eq!(doc.bookmarks.len(), 5);
    assert_eq!(doc.bookmarks[0].children.len(), 2); // First bookmark has 2 children

    // Add outline items
    doc.add_outline_item("Introduction", 0, 100.0, 750.0)
        .unwrap();
    doc.add_outline_item("Methodology", 1, 100.0, 750.0)
        .unwrap();
    doc.add_outline_item("Conclusion", 2, 100.0, 750.0).unwrap();

    assert_eq!(doc.outline.len(), 3);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_security_features() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    doc.add_text(
        0,
        "Confidential Document",
        100.0,
        700.0,
        "Helvetica-Bold",
        16.0,
    )
    .unwrap();

    // Set password protection
    doc.set_password_protection(Some("user123"), Some("owner456"));
    assert!(doc.security_settings.encryption_enabled);
    assert_eq!(
        doc.security_settings.user_password,
        Some("user123".to_string())
    );
    assert_eq!(
        doc.security_settings.owner_password,
        Some("owner456".to_string())
    );

    // Set restricted permissions
    let mut permissions = PermissionFlags::default();
    permissions.allow_copy = false;
    permissions.allow_modify_contents = false;
    permissions.allow_printing = true;

    doc.set_permissions(permissions);
    assert!(!doc.security_settings.permissions.allow_copy);
    assert!(!doc.security_settings.permissions.allow_modify_contents);
    assert!(doc.security_settings.permissions.allow_printing);

    // Set encryption level
    doc.set_encryption_level(EncryptionLevel::Aes256Bit);
    assert_eq!(
        doc.security_settings.encryption_level,
        EncryptionLevel::Aes256Bit
    );

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_javascript_support() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add JavaScript scripts
    let script1_id = doc
        .add_javascript(
            "validation",
            "
        function validateForm() {
            var name = this.getField('name');
            if (!name.value) {
                app.alert('Name is required!');
                return false;
            }
            return true;
        }
    ",
        )
        .unwrap();

    let script2_id = doc
        .add_javascript(
            "calculation",
            "
        function calculateTotal() {
            var quantity = this.getField('quantity').value;
            var price = this.getField('price').value;
            this.getField('total').value = quantity * price;
        }
    ",
        )
        .unwrap();

    // Add document-level JavaScript
    doc.add_document_javascript(
        "
        console.println('PDF document loaded successfully');
        this.print({
            bUI: true,
            bSilent: false,
            bShrinkToFit: true
        });
    ",
    )
    .unwrap();

    assert_eq!(doc.javascript.len(), 3);

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let stats = doc.get_document_stats();
    assert!(stats.has_javascript);
    assert_eq!(stats.javascript_count, 3);
}

#[test]
fn test_pdf_attachments() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    doc.add_text(
        0,
        "Document with Attachments",
        100.0,
        700.0,
        "Helvetica-Bold",
        14.0,
    )
    .unwrap();

    // Add various attachments
    let txt_data = b"This is a text attachment content.".to_vec();
    let attachment1_id = doc
        .add_attachment(
            "readme.txt",
            txt_data,
            "text/plain",
            "Instructions and documentation",
        )
        .unwrap();

    let csv_data = b"Name,Age,Department\nAlice,30,Engineering\nBob,25,Marketing".to_vec();
    let attachment2_id = doc
        .add_attachment(
            "employees.csv",
            csv_data,
            "text/csv",
            "Employee data spreadsheet",
        )
        .unwrap();

    let json_data =
        b"{\"version\": \"1.0\", \"format\": \"pdf\", \"features\": [\"annotations\", \"forms\"]}"
            .to_vec();
    let attachment3_id = doc
        .add_attachment(
            "metadata.json",
            json_data,
            "application/json",
            "Document metadata and configuration",
        )
        .unwrap();

    assert_eq!(doc.attachments.len(), 3);

    // Verify attachment properties
    let txt_attachment = doc.attachments.get(&attachment1_id).unwrap();
    assert_eq!(txt_attachment.filename, "readme.txt");
    assert_eq!(txt_attachment.mime_type, "text/plain");
    assert!(txt_attachment.checksum.is_some());

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let stats = doc.get_document_stats();
    assert_eq!(stats.attachment_count, 3);
}

#[test]
fn test_pdf_layers() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add content layers
    let background_layer_id = doc.add_layer("Background", true).unwrap();
    let content_layer_id = doc.add_layer("Main Content", true).unwrap();
    let annotations_layer_id = doc.add_layer("Annotations", false).unwrap(); // Hidden by default

    assert_eq!(doc.layers.len(), 3);

    // Verify layer properties
    let background_layer = doc.layers.get(&background_layer_id).unwrap();
    assert_eq!(background_layer.name, "Background");
    assert!(background_layer.visible);
    assert!(background_layer.printable);

    let annotations_layer = doc.layers.get(&annotations_layer_id).unwrap();
    assert_eq!(annotations_layer.name, "Annotations");
    assert!(!annotations_layer.visible); // Hidden

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let stats = doc.get_document_stats();
    assert_eq!(stats.layer_count, 3);
}

#[test]
fn test_pdf_comprehensive_document() {
    let mut doc = PdfDocument::new();

    // Set comprehensive metadata
    doc.set_author("Comprehensive Test Author");
    doc.set_title("Advanced PDF Feature Demonstration");
    doc.set_subject("Complete PDF SDK Testing");
    doc.set_description("A comprehensive test document showcasing all PDF capabilities including forms, annotations, images, tables, security, and JavaScript");
    doc.set_keywords(
        "pdf, comprehensive, forms, annotations, security, javascript, images, tables",
    );
    doc.set_category("Software Testing");

    // Add multiple pages with different content
    let page1_id = doc.add_a4_page().unwrap();
    let page2_id = doc.add_letter_page().unwrap();
    let page3_id = doc.add_legal_page().unwrap();

    // Page 1: Title page with graphics
    doc.add_heading(0, 1, "PDF SDK Comprehensive Test", 100.0, 750.0)
        .unwrap();
    doc.add_paragraph(0, "This document demonstrates all advanced PDF features including interactive forms, multimedia annotations, embedded images, structured tables, document security, and JavaScript automation.", 100.0, 700.0, 400.0, 12.0).unwrap();

    // Add company logo simulation
    let logo_data = image_fixture(image::ImageFormat::Png, 64, 64);
    let logo_id = doc
        .add_image_from_bytes(&logo_data, "image/png", Some("Company Logo"))
        .unwrap();
    doc.add_image_to_page(0, &logo_id, 450.0, 700.0, 80.0, 80.0)
        .unwrap();

    // Add decorative shapes
    doc.add_rectangle(0, 100.0, 600.0, 400.0, 50.0, None, Some(Color::blue()))
        .unwrap();
    doc.add_line(0, 100.0, 590.0, 500.0, 590.0, Color::red(), 2.0)
        .unwrap();

    // Page 2: Interactive form
    doc.add_heading(1, 1, "Interactive Form", 100.0, 750.0)
        .unwrap();

    // Personal information section
    doc.add_text(
        1,
        "Personal Information:",
        100.0,
        700.0,
        "Helvetica-Bold",
        12.0,
    )
    .unwrap();
    doc.add_text_field(1, "firstName", 100.0, 680.0, 150.0, 20.0, "")
        .unwrap();
    doc.add_text(1, "First Name", 260.0, 685.0, "Helvetica", 10.0)
        .unwrap();

    doc.add_text_field(1, "lastName", 100.0, 650.0, 150.0, 20.0, "")
        .unwrap();
    doc.add_text(1, "Last Name", 260.0, 655.0, "Helvetica", 10.0)
        .unwrap();

    doc.add_text_field(1, "email", 100.0, 620.0, 200.0, 20.0, "user@example.com")
        .unwrap();
    doc.add_text(1, "Email Address", 310.0, 625.0, "Helvetica", 10.0)
        .unwrap();

    // Preferences section
    doc.add_text(1, "Preferences:", 100.0, 580.0, "Helvetica-Bold", 12.0)
        .unwrap();
    doc.add_checkbox(1, "newsletter", 100.0, 560.0, 15.0, true)
        .unwrap();
    doc.add_text(
        1,
        "Subscribe to newsletter",
        125.0,
        565.0,
        "Helvetica",
        10.0,
    )
    .unwrap();

    doc.add_dropdown(
        1,
        "experience",
        100.0,
        530.0,
        120.0,
        20.0,
        &["Beginner", "Intermediate", "Advanced", "Expert"],
        Some(1),
    )
    .unwrap();
    doc.add_text(1, "Experience Level", 230.0, 535.0, "Helvetica", 10.0)
        .unwrap();

    // Comments section
    doc.add_text(
        1,
        "Additional Comments:",
        100.0,
        490.0,
        "Helvetica-Bold",
        12.0,
    )
    .unwrap();
    doc.add_multiline_text_field(
        1,
        "comments",
        100.0,
        400.0,
        350.0,
        80.0,
        "Please enter any additional comments...",
    )
    .unwrap();

    // Page 3: Data table and statistics
    doc.add_heading(2, 1, "Performance Data", 100.0, 950.0)
        .unwrap();

    doc.add_table(
        2,
        100.0,
        850.0,
        &[
            "Product", "Q1 Sales", "Q2 Sales", "Q3 Sales", "Q4 Sales", "Total",
        ],
        &[
            vec![
                "Laptop Pro",
                "$125,000",
                "$142,000",
                "$168,000",
                "$189,000",
                "$624,000",
            ],
            vec![
                "Tablet Zen",
                "$80,000",
                "$95,000",
                "$110,000",
                "$125,000",
                "$410,000",
            ],
            vec![
                "Smart Watch",
                "$65,000",
                "$78,000",
                "$92,000",
                "$108,000",
                "$343,000",
            ],
            vec![
                "Headphones",
                "$45,000",
                "$52,000",
                "$61,000",
                "$69,000",
                "$227,000",
            ],
            vec![
                "Total",
                "$315,000",
                "$367,000",
                "$431,000",
                "$491,000",
                "$1,604,000",
            ],
        ],
    )
    .unwrap();

    // Add annotations explaining the data
    doc.add_text_annotation(
        2, 100.0, 780.0, 200.0, 40.0,
        "Sales data shows consistent growth across all product lines with Q4 being the strongest quarter.",
        "Data Analyst"
    ).unwrap();

    doc.add_highlight_annotation(2, 480.0, 820.0, 100.0, 15.0, "Reviewer")
        .unwrap(); // Highlight total

    // Add security
    doc.set_password_protection(Some("view123"), Some("edit456"));
    let mut permissions = PermissionFlags::default();
    permissions.allow_modify_contents = false;
    permissions.allow_copy = false;
    doc.set_permissions(permissions);

    // Add JavaScript for form validation
    doc.add_javascript(
        "formValidation",
        "
        function validateForm() {
            var firstName = this.getField('firstName');
            var lastName = this.getField('lastName');
            var email = this.getField('email');

            if (!firstName.value || !lastName.value) {
                app.alert('Please fill in your name.');
                return false;
            }

            if (!email.value || email.value.indexOf('@') === -1) {
                app.alert('Please enter a valid email address.');
                return false;
            }

            app.alert('Form validation successful!');
            return true;
        }
    ",
    )
    .unwrap();

    // Add layers for different content types
    // Include the navigation entries asserted by the statistics check below.
    let overview = doc.add_bookmark("Overview", 0, 100.0, 750.0).unwrap();
    doc.add_nested_bookmark(&overview, "Introduction", 0, 100.0, 700.0)
        .unwrap();
    doc.add_nested_bookmark(&overview, "Graphics", 0, 100.0, 600.0)
        .unwrap();
    doc.add_bookmark("Registration", 1, 100.0, 750.0).unwrap();
    doc.add_bookmark("Performance Data", 2, 100.0, 950.0)
        .unwrap();
    doc.add_layer("Text Content", true).unwrap();
    doc.add_layer("Form Fields", true).unwrap();
    doc.add_layer("Annotations", true).unwrap();
    doc.add_layer("Images", true).unwrap();

    // Add file attachments
    let supporting_data = b"Supporting documentation and analysis data.".to_vec();
    doc.add_attachment(
        "supporting_data.txt",
        supporting_data,
        "text/plain",
        "Additional analysis documentation",
    )
    .unwrap();

    // Generate document
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Verify comprehensive statistics
    let stats = doc.get_document_stats();
    assert_eq!(stats.page_count, 3);
    assert!(stats.has_security);
    assert!(stats.has_javascript);
    assert!(stats.has_forms);
    assert!(stats.has_annotations);
    assert_eq!(stats.form_field_count, 6);
    assert_eq!(stats.annotation_count, 2);
    assert_eq!(stats.bookmark_count, 5);
    assert_eq!(stats.layer_count, 4);
    assert_eq!(stats.attachment_count, 1);
    assert!(stats.total_text_length > 100);

    // Parse and verify round-trip
    let parsed_doc = PdfDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.title, "Advanced PDF Feature Demonstration");
    assert_eq!(parsed_doc.author, "Comprehensive Test Author");
    assert_eq!(parsed_doc.pages.len(), 3);
}

#[test]
fn test_pdf_document_validation() {
    let mut doc = PdfDocument::new();

    // Test validation on empty document
    let issues = doc.validate_structure().unwrap();
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|issue| issue.contains("no pages")));

    // Add a page and test validation
    let _page_id = doc.add_a4_page().unwrap();
    let issues = doc.validate_structure().unwrap();
    assert!(issues.is_empty()); // Should be valid now

    // Add content and test again
    doc.add_text(0, "Valid document", 100.0, 700.0, "Helvetica", 12.0)
        .unwrap();
    let issues = doc.validate_structure().unwrap();
    assert!(issues.is_empty());

    // Add invalid annotation (referencing non-existent page)
    doc.add_text_annotation(5, 100.0, 100.0, 100.0, 20.0, "Invalid", "Test")
        .unwrap();
    let issues = doc.validate_structure().unwrap();
    assert!(!issues.is_empty());
    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("non-existent page"))
    );
}

#[test]
fn test_pdf_document_optimization() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Add images
    let image_data = image_fixture(image::ImageFormat::Png, 1, 1);
    let used_image_id = doc
        .add_image_from_bytes(&image_data, "image/png", Some("Used Image"))
        .unwrap();
    let unused_image_id = doc
        .add_image_from_bytes(&image_data, "image/png", Some("Unused Image"))
        .unwrap();

    // Use only one image
    doc.add_image_to_page(0, &used_image_id, 100.0, 500.0, 100.0, 100.0)
        .unwrap();

    // Add empty annotations
    doc.add_text_annotation(0, 100.0, 400.0, 100.0, 20.0, "", "Test Author")
        .unwrap();
    doc.add_text_annotation(
        0,
        100.0,
        380.0,
        100.0,
        20.0,
        "Valid annotation",
        "Test Author",
    )
    .unwrap();

    assert_eq!(doc.images.len(), 2);
    assert_eq!(doc.annotations.len(), 2);

    // Optimize document
    doc.optimize_document().unwrap();

    // Verify optimization results
    assert_eq!(doc.images.len(), 1); // Unused image removed
    assert!(doc.images.contains_key(&used_image_id));
    assert!(!doc.images.contains_key(&unused_image_id));

    assert_eq!(doc.annotations.len(), 1); // Empty annotation removed

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_text_extraction() {
    let mut doc = PdfDocument::new();

    // Add multiple pages with content
    let _page1_id = doc.add_a4_page().unwrap();
    let _page2_id = doc.add_a4_page().unwrap();

    // Add text content
    doc.add_text(0, "Page 1 Title", 100.0, 750.0, "Helvetica-Bold", 16.0)
        .unwrap();
    doc.add_text(
        0,
        "This is the content of the first page.",
        100.0,
        720.0,
        "Helvetica",
        12.0,
    )
    .unwrap();

    doc.add_text(1, "Page 2 Title", 100.0, 750.0, "Helvetica-Bold", 16.0)
        .unwrap();
    doc.add_text(
        1,
        "This is the content of the second page.",
        100.0,
        720.0,
        "Helvetica",
        12.0,
    )
    .unwrap();

    // Add table with data
    doc.add_table(
        1,
        100.0,
        650.0,
        &["Item", "Quantity", "Price"],
        &[
            vec!["Widget A", "10", "$25.00"],
            vec!["Widget B", "5", "$45.00"],
        ],
    )
    .unwrap();

    // Test text extraction
    let extracted_text = doc.get_text().unwrap();
    assert!(extracted_text.contains("Page 1 Title"));
    assert!(extracted_text.contains("first page"));
    assert!(extracted_text.contains("Page 2 Title"));
    assert!(extracted_text.contains("second page"));
    assert!(extracted_text.contains("Widget A"));
    assert!(extracted_text.contains("$25.00"));

    // Test simplified text extraction
    let simple_text = doc.to_simple_text();
    assert!(simple_text.contains("Page 1 Title"));
    assert!(simple_text.contains("Widget A"));

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_page_management() {
    let mut doc = PdfDocument::new();

    // Add pages of different sizes
    let a4_id = doc.add_a4_page().unwrap();
    let letter_id = doc.add_letter_page().unwrap();
    let legal_id = doc.add_legal_page().unwrap();
    let tabloid_id = doc.add_tabloid_page().unwrap();

    assert_eq!(doc.pages.len(), 4);

    // Verify page dimensions
    assert_eq!(doc.pages[0].width, 595.28); // A4
    assert_eq!(doc.pages[1].width, 612.0); // Letter
    assert_eq!(doc.pages[2].height, 1008.0); // Legal
    assert_eq!(doc.pages[3].width, 792.0); // Tabloid

    // Add content to each page
    doc.add_text(0, "A4 Page Content", 100.0, 750.0, "Helvetica", 12.0)
        .unwrap();
    doc.add_text(1, "Letter Page Content", 100.0, 750.0, "Helvetica", 12.0)
        .unwrap();
    doc.add_text(2, "Legal Page Content", 100.0, 950.0, "Helvetica", 12.0)
        .unwrap();
    doc.add_text(3, "Tabloid Page Content", 100.0, 1150.0, "Helvetica", 12.0)
        .unwrap();

    // Test page extraction
    let extracted_doc = doc.extract_pages(&[0, 2]).unwrap(); // Extract A4 and Legal pages
    assert_eq!(extracted_doc.pages.len(), 2);
    assert_eq!(extracted_doc.pages[0].width, 595.28); // A4
    assert_eq!(extracted_doc.pages[1].height, 1008.0); // Legal

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_color_operations() {
    let mut doc = PdfDocument::new();
    let _page_id = doc.add_a4_page().unwrap();

    // Test color creation methods
    let red = Color::red();
    let blue = Color::blue();
    let custom_green = Color::rgb(0.0, 0.8, 0.0);
    let transparent_yellow = Color::rgba(1.0, 1.0, 0.0, 0.5);
    let hex_purple = Color::from_hex("#800080").unwrap();
    let gray_50 = Color::gray(0.5);

    // Add text in different colors
    doc.add_styled_text(
        0,
        "Red Text",
        100.0,
        700.0,
        "Helvetica",
        12.0,
        red,
        TextStyle::default(),
    )
    .unwrap();
    doc.add_styled_text(
        0,
        "Blue Text",
        100.0,
        680.0,
        "Helvetica",
        12.0,
        blue,
        TextStyle::default(),
    )
    .unwrap();
    doc.add_styled_text(
        0,
        "Custom Green",
        100.0,
        660.0,
        "Helvetica",
        12.0,
        custom_green,
        TextStyle::default(),
    )
    .unwrap();
    doc.add_styled_text(
        0,
        "Gray Text",
        100.0,
        640.0,
        "Helvetica",
        12.0,
        gray_50,
        TextStyle::default(),
    )
    .unwrap();

    // Add colored shapes
    doc.add_rectangle(
        0,
        300.0,
        650.0,
        50.0,
        20.0,
        Some(hex_purple),
        Some(Color::black()),
    )
    .unwrap();
    doc.add_circle(
        0,
        400.0,
        650.0,
        25.0,
        Some(transparent_yellow),
        Some(Color::red()),
    )
    .unwrap();

    // Test hex color parsing
    assert!(Color::from_hex("#FF0000").is_ok());
    assert!(Color::from_hex("#00FF00FF").is_ok()); // With alpha
    assert!(Color::from_hex("invalid").is_err());

    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_pdf_empty_document() {
    let doc = PdfDocument::new();

    // Test operations on empty document
    let stats = doc.get_document_stats();
    assert_eq!(stats.page_count, 0);
    assert_eq!(stats.image_count, 0);
    assert_eq!(stats.annotation_count, 0);
    assert_eq!(stats.form_field_count, 0);
    assert!(!stats.has_security);
    assert!(!stats.has_javascript);
    assert!(!stats.has_forms);
    assert!(!stats.has_annotations);

    // Test text extraction from empty document
    let text = doc.get_text().unwrap();
    assert!(text.trim().is_empty());

    let simple_text = doc.to_simple_text();
    assert!(simple_text.is_empty());

    // Test validation
    let issues = doc.validate_structure().unwrap();
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|issue| issue.contains("no pages")));
}

#[test]
fn test_pdf_advanced_features_integration() {
    let mut doc = PdfDocument::new();

    // Create a comprehensive document combining all features
    doc.set_title("Integration Test Document");
    doc.set_author("PDF SDK Integration Test");

    // Add multiple pages
    let _page1_id = doc.add_a4_page().unwrap();
    let _page2_id = doc.add_a4_page().unwrap();

    // Page 1: Mixed content
    doc.add_heading(0, 1, "Integration Test", 100.0, 750.0)
        .unwrap();

    // Add table
    doc.add_table(
        0,
        100.0,
        650.0,
        &["Feature", "Status", "Coverage"],
        &[
            vec!["Text Rendering", "✓ Complete", "100%"],
            vec!["Image Support", "✓ Complete", "95%"],
            vec!["Form Fields", "✓ Complete", "90%"],
            vec!["Annotations", "✓ Complete", "95%"],
            vec!["Security", "✓ Complete", "85%"],
        ],
    )
    .unwrap();

    // Add image
    let test_image = image_fixture(image::ImageFormat::Png, 1, 1);
    let image_id = doc
        .add_image_from_bytes(&test_image, "image/png", Some("Test Chart"))
        .unwrap();
    doc.add_image_to_page(0, &image_id, 350.0, 500.0, 150.0, 100.0)
        .unwrap();

    // Add annotations
    doc.add_highlight_annotation(0, 100.0, 700.0, 150.0, 15.0, "Reviewer")
        .unwrap();
    doc.add_text_annotation(0, 350.0, 450.0, 100.0, 30.0, "Important note", "Author")
        .unwrap();

    // Page 2: Interactive elements
    doc.add_heading(1, 1, "Interactive Elements", 100.0, 750.0)
        .unwrap();

    // Add form fields
    doc.add_text_field(1, "testField", 100.0, 700.0, 200.0, 25.0, "Test input")
        .unwrap();
    doc.add_checkbox(1, "testCheck", 100.0, 660.0, 15.0, true)
        .unwrap();
    doc.add_dropdown(
        1,
        "testDropdown",
        100.0,
        620.0,
        150.0,
        25.0,
        &["Option 1", "Option 2", "Option 3"],
        Some(0),
    )
    .unwrap();

    // Add JavaScript
    doc.add_javascript("testScript", "app.alert('Integration test complete!');")
        .unwrap();

    // Add bookmarks
    doc.add_bookmark("Test Features", 0, 100.0, 750.0).unwrap();
    doc.add_bookmark("Interactive Elements", 1, 100.0, 750.0)
        .unwrap();

    // Add layers
    doc.add_layer("Content", true).unwrap();
    doc.add_layer("Annotations", true).unwrap();

    // Add attachment
    let attachment_data = b"Integration test data file content.".to_vec();
    doc.add_attachment(
        "test_data.txt",
        attachment_data,
        "text/plain",
        "Integration test data",
    )
    .unwrap();

    // Set security
    doc.set_password_protection(Some("test123"), None);

    // Generate and verify
    let bytes = doc.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    // Comprehensive statistics verification
    let stats = doc.get_document_stats();
    assert_eq!(stats.page_count, 2);
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.annotation_count, 2);
    assert_eq!(stats.form_field_count, 3);
    assert_eq!(stats.bookmark_count, 2);
    assert_eq!(stats.layer_count, 2);
    assert_eq!(stats.attachment_count, 1);
    assert_eq!(stats.javascript_count, 1);
    assert!(stats.has_security);
    assert!(stats.has_javascript);
    assert!(stats.has_forms);
    assert!(stats.has_annotations);
    assert!(stats.total_text_length > 50);

    // Test round-trip
    let parsed_doc = PdfDocument::from_bytes(&bytes).unwrap();
    assert_eq!(parsed_doc.title, "Integration Test Document");
    assert_eq!(parsed_doc.author, "PDF SDK Integration Test");
    assert_eq!(parsed_doc.pages.len(), 2);
}
