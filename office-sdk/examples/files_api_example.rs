use office_sdk::Result;
use office_sdk::files::{CreateRequest, DocumentFormat, FileInput, FileMetadata, FilesAPI};

fn main() -> Result<()> {
    println!("=== Office SDK Files API Demo ===\n");

    let files_api = FilesAPI::new();

    demo_basic_document_creation(&files_api)?;
    demo_markdown_conversion(&files_api)?;
    demo_template_usage(&files_api)?;
    demo_batch_operations(&files_api)?;
    demo_document_conversion(&files_api)?;

    Ok(())
}

fn demo_basic_document_creation(api: &FilesAPI) -> Result<()> {
    println!("1. Basic Document Creation");
    println!("==========================");

    let content = "This is a simple document created with the Files API.";
    let mut metadata = FileMetadata::default();
    metadata.title = Some("Sample Document".to_string());
    metadata.author = Some("Files API Demo".to_string());

    let docx_result =
        api.create_document_with_metadata(DocumentFormat::Docx, content, metadata.clone())?;

    println!(
        "✓ Created DOCX: {} ({} bytes)",
        docx_result.filename,
        docx_result.size()
    );
    docx_result.save_to_file("output/sample_document.docx")?;

    let pdf_result = api.create_document_with_metadata(DocumentFormat::Pdf, content, metadata)?;

    println!(
        "✓ Created PDF: {} ({} bytes)",
        pdf_result.filename,
        pdf_result.size()
    );
    pdf_result.save_to_file("output/sample_document.pdf")?;

    println!();
    Ok(())
}

fn demo_markdown_conversion(api: &FilesAPI) -> Result<()> {
    println!("2. Markdown Conversion");
    println!("======================");

    let markdown_content = r#"# Project Documentation

## Overview
This project demonstrates the Files API capabilities.

## Features
- Document creation from markdown
- Multiple format support
- Template system
- Batch operations

### Code Example
```rust
let api = FilesAPI::new();
let result = api.create_document(DocumentFormat::Docx, content)?;
```

## Conclusion
The Files API provides a powerful interface for document manipulation."#;

    let mut metadata = FileMetadata::default();
    metadata.title = Some("Project Documentation".to_string());
    metadata.author = Some("Documentation Team".to_string());
    metadata.keywords = vec!["API".to_string(), "Documentation".to_string()];

    let docx_result = api.create_document_with_metadata(
        DocumentFormat::Docx,
        markdown_content,
        metadata.clone(),
    )?;

    println!(
        "✓ Converted Markdown to DOCX: {} ({} bytes)",
        docx_result.filename,
        docx_result.size()
    );
    docx_result.save_to_file("output/markdown_converted.docx")?;

    let pptx_result =
        api.create_document_with_metadata(DocumentFormat::Pptx, markdown_content, metadata)?;

    println!(
        "✓ Converted Markdown to PPTX: {} ({} bytes)",
        pptx_result.filename,
        pptx_result.size()
    );
    pptx_result.save_to_file("output/markdown_converted.pptx")?;

    println!();
    Ok(())
}

fn demo_template_usage(api: &FilesAPI) -> Result<()> {
    println!("3. Template Usage");
    println!("=================");

    let business_content = r#"Executive Summary

Our quarterly results show significant growth across all sectors.

Key Achievements:
- 25% increase in revenue
- 15% growth in customer base
- Successful product launch
- Market expansion into 3 new regions

Financial Performance:
- Revenue: $2.5M (up from $2.0M)
- Profit Margin: 18%
- Operating Expenses: $1.8M

Strategic Initiatives:
- Digital transformation project
- Sustainability program launch
- Partnership with key vendors
- Team expansion in R&D

Outlook for next quarter remains positive with continued growth expected."#;

    let mut metadata = FileMetadata::default();
    metadata.title = Some("Quarterly Business Report".to_string());
    metadata.author = Some("Executive Team".to_string());

    let business_docx =
        api.builders
            .docx
            .with_template("business", business_content, metadata.clone())?;
    println!(
        "✓ Created business document: {} ({} bytes)",
        business_docx.filename,
        business_docx.size()
    );
    business_docx.save_to_file("output/business_report.docx")?;

    let csv_data = r#"Product,Q1 Sales,Q2 Sales,Q3 Sales,Q4 Sales
Widget A,15000,18000,22000,25000
Widget B,8000,9500,11000,12500
Widget C,12000,14000,16000,18000
Widget D,5000,6200,7500,8900"#;

    let mut csv_metadata = FileMetadata::default();
    csv_metadata.title = Some("Sales Data".to_string());

    let financial_xlsx = api
        .builders
        .xlsx
        .with_template("financial", csv_data, csv_metadata)?;
    println!(
        "✓ Created financial spreadsheet: {} ({} bytes)",
        financial_xlsx.filename,
        financial_xlsx.size()
    );
    financial_xlsx.save_to_file("output/sales_data.xlsx")?;

    println!();
    Ok(())
}

fn demo_batch_operations(api: &FilesAPI) -> Result<()> {
    println!("4. Batch Operations");
    println!("===================");

    let requests = vec![
        CreateRequest {
            format: DocumentFormat::Docx,
            content: "This is document 1 content.".to_string(),
            filename: Some("doc1.docx".to_string()),
            metadata: Some({
                let mut meta = FileMetadata::default();
                meta.title = Some("Document 1".to_string());
                meta
            }),
            template: None,
        },
        CreateRequest {
            format: DocumentFormat::Pdf,
            content: "This is document 2 content with more detailed information.".to_string(),
            filename: Some("doc2.pdf".to_string()),
            metadata: Some({
                let mut meta = FileMetadata::default();
                meta.title = Some("Document 2".to_string());
                meta
            }),
            template: None,
        },
        CreateRequest {
            format: DocumentFormat::Xlsx,
            content: "Item,Quantity,Price\nApples,10,$2.50\nOranges,15,$3.00".to_string(),
            filename: Some("inventory.xlsx".to_string()),
            metadata: Some({
                let mut meta = FileMetadata::default();
                meta.title = Some("Inventory".to_string());
                meta
            }),
            template: Some("inventory".to_string()),
        },
    ];

    let results = api.batch_create(requests)?;

    println!("✓ Created {} documents in batch:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}: {} ({} bytes)", i + 1, result.filename, result.size());
        result.save_to_file(&format!("output/{}", result.filename))?;
    }

    println!();
    Ok(())
}

fn demo_document_conversion(api: &FilesAPI) -> Result<()> {
    println!("5. Document Conversion");
    println!("======================");

    let original_content = r#"# Meeting Notes

**Date:** 2024-01-15
**Attendees:** Alice, Bob, Carol

## Agenda Items

1. Project status update
2. Budget review
3. Next quarter planning

## Action Items

- [ ] Alice to prepare budget report
- [ ] Bob to schedule client meeting
- [ ] Carol to update project timeline

## Next Meeting

Scheduled for January 29th, same time."#;

    println!("Converting text content to multiple formats...");

    let text_input = FileInput::Text(original_content.to_string());

    let formats_to_convert = vec![
        DocumentFormat::Docx,
        DocumentFormat::Pdf,
        DocumentFormat::Xlsx,
        DocumentFormat::Markdown,
    ];

    for format in &formats_to_convert {
        let result = api.convert_document(text_input.clone(), *format)?;
        println!(
            "✓ Converted to {}: {} ({} bytes)",
            format.file_extension().to_uppercase(),
            result.filename,
            result.size()
        );
        result.save_to_file(&format!("output/meeting_notes.{}", format.file_extension()))?;
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_files_api_basic_functionality() {
        let api = FilesAPI::new();
        let content = "Test document content";
        let metadata = FileMetadata::default();

        let result = api.create_document_with_metadata(DocumentFormat::Docx, content, metadata);

        assert!(result.is_ok());
        let file_result = result.unwrap();
        assert!(!file_result.content.is_empty());
        assert!(file_result.filename.ends_with(".docx"));
    }

    #[test]
    fn test_supported_formats() {
        let formats = FilesAPI::get_supported_formats();
        assert!(formats.contains(&DocumentFormat::Docx));
        assert!(formats.contains(&DocumentFormat::Pdf));
        assert!(formats.contains(&DocumentFormat::Xlsx));
    }

    #[test]
    fn test_batch_creation() {
        let api = FilesAPI::new();
        let requests = vec![
            CreateRequest {
                format: DocumentFormat::Docx,
                content: "Test 1".to_string(),
                filename: None,
                metadata: None,
                template: None,
            },
            CreateRequest {
                format: DocumentFormat::Pdf,
                content: "Test 2".to_string(),
                filename: None,
                metadata: None,
                template: None,
            },
        ];

        let results = api.batch_create(requests);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 2);
    }
}
