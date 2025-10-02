use office_sdk::OfficeDocument;
use office_sdk::docx::DocxDocument;
use office_sdk::markdown::MarkdownDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK Markdown ↔ DOCX Conversion Demo ===\n");

    // Sample markdown content with various elements
    let markdown_content = r#"# Office SDK Documentation

Welcome to the Office SDK - a comprehensive Rust library for handling Office document formats.

## Core Features

The SDK supports multiple document formats:

- DOCX (Word documents)
- XLSX (Excel spreadsheets)
- PPTX (PowerPoint presentations)
- Markdown conversion (NEW!)

### Code Examples

Here's how to create a DOCX document:

```rust
use office_sdk::docx::DocxDocument;
use office_sdk::OfficeDocument;

let mut doc = DocxDocument::new();
doc.add_paragraph("Hello, World!");
doc.save("document.docx")?;
```

## Markdown Conversion

### From Markdown to DOCX

The library can convert markdown documents to DOCX format:

```rust
let markdown_doc = MarkdownDocument::from_markdown(markdown_text)?;
let docx_doc = markdown_doc.to_docx()?;
docx_doc.save("output.docx")?;
```

### From DOCX to Markdown

Conversely, you can extract markdown from DOCX documents:

```rust
let docx = DocxDocument::from_file("input.docx")?;
let markdown = MarkdownDocument::from_docx(&docx)?;
let markdown_text = markdown.to_markdown();
```

## Advanced Features

> The SDK provides comprehensive error handling and type safety.

Key benefits:

1. Memory-efficient ZIP processing
2. Full OpenXML compatibility
3. Extensive test coverage (97%+)
4. Clean, idiomatic Rust APIs

---

Ready to get started? Check out the examples and documentation!
"#;

    println!("📝 Original Markdown Content:");
    println!("{}", markdown_content);
    println!("\n{}\n", "=".repeat(60));

    // Step 1: Parse the markdown
    println!("🔍 Step 1: Parsing Markdown...");
    let markdown_doc = MarkdownDocument::from_markdown(markdown_content)?;
    println!(
        "✅ Parsed {} markdown elements",
        markdown_doc.elements.len()
    );

    // Step 2: Convert markdown to DOCX
    println!("\n📄 Step 2: Converting to DOCX...");
    let docx_doc = markdown_doc.to_docx()?;
    docx_doc.save("markdown_example.docx")?;
    println!("✅ Created DOCX file: markdown_example.docx");

    // Step 3: Load the DOCX and verify content
    println!("\n🔍 Step 3: Verifying DOCX content...");
    let loaded_docx = DocxDocument::from_file("markdown_example.docx")?;
    let docx_text = loaded_docx.get_text()?;
    println!("✅ DOCX contains {} characters of text", docx_text.len());

    // Step 4: Convert DOCX back to markdown
    println!("\n🔄 Step 4: Converting DOCX back to Markdown...");
    let markdown_from_docx = MarkdownDocument::from_docx(&loaded_docx)?;
    let regenerated_markdown = markdown_from_docx.to_markdown();
    println!(
        "✅ Generated {} characters of markdown",
        regenerated_markdown.len()
    );

    // Step 5: Save the regenerated markdown
    std::fs::write("regenerated_example.md", &regenerated_markdown)?;
    println!("✅ Saved regenerated markdown: regenerated_example.md");

    // Step 6: Show the regenerated markdown
    println!("\n📝 Regenerated Markdown Content:");
    println!("{}", regenerated_markdown);

    // Step 7: Round-trip test
    println!("\n🔄 Step 7: Round-trip conversion test...");
    let markdown_doc2 = MarkdownDocument::from_markdown(&regenerated_markdown)?;
    let docx_doc2 = markdown_doc2.to_docx()?;
    docx_doc2.save("roundtrip_example.docx")?;
    println!("✅ Round-trip conversion successful!");

    // Step 8: Create a comparison document
    println!("\n📊 Step 8: Creating comparison document...");
    let mut comparison_doc = DocxDocument::new();
    comparison_doc.add_heading(1, "Conversion Demo Results")?;
    comparison_doc
        .add_paragraph("This document demonstrates the markdown conversion capabilities.")?;
    comparison_doc.add_heading(2, "Original Elements Detected")?;
    comparison_doc.add_paragraph(&format!(
        "Total markdown elements: {}",
        markdown_doc.elements.len()
    ))?;
    comparison_doc.add_heading(2, "Key Features Demonstrated")?;
    comparison_doc.add_bulleted_list_item("Heading parsing (H1, H2, H3)")?;
    comparison_doc.add_bulleted_list_item("Paragraph content")?;
    comparison_doc.add_bulleted_list_item("Code blocks with language detection")?;
    comparison_doc.add_bulleted_list_item("Ordered and unordered lists")?;
    comparison_doc.add_bulleted_list_item("Blockquotes")?;
    comparison_doc.add_bulleted_list_item("Horizontal rules")?;
    comparison_doc.add_quote("Bidirectional conversion between Markdown and DOCX formats")?;
    comparison_doc.add_horizontal_rule()?;
    comparison_doc.add_paragraph("Conversion completed successfully!")?;

    comparison_doc.save("conversion_comparison.docx")?;
    println!("✅ Created comparison document: conversion_comparison.docx");

    println!("\n🎉 Demo completed successfully!");
    println!("\nFiles created:");
    println!("  - markdown_example.docx (original markdown → DOCX)");
    println!("  - regenerated_example.md (DOCX → markdown)");
    println!("  - roundtrip_example.docx (markdown → DOCX → markdown → DOCX)");
    println!("  - conversion_comparison.docx (demonstration summary)");

    Ok(())
}
