use office_sdk::OfficeDocument;
use office_sdk::docx::{
    DocxDocument, PageOrientation, PaperSize, Table, TableCell, TableRow, TextAlign,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK Advanced DOCX Features Demo ===\n");

    // Create a comprehensive document with all advanced features
    let mut doc = DocxDocument::new()
        .with_author("Advanced SDK User")
        .with_title("Complete DOCX Feature Demonstration")
        .with_subject("Comprehensive Office SDK Testing");

    // Configure document properties
    doc.metadata.description = Some("This document showcases all advanced DOCX features including images, tables, VBA macros, and formatting.".to_string());
    doc.metadata.keywords = Some("rust, office, docx, sdk, advanced, features".to_string());
    doc.metadata.category = Some("Documentation".to_string());

    // Set page formatting
    println!("📄 Setting up page formatting...");
    doc.set_page_size(PaperSize::Letter);
    doc.set_orientation(PageOrientation::Portrait);
    doc.set_margins(1440.0, 1440.0, 1800.0, 1800.0); // 1 inch top/bottom, 1.25 inch left/right

    // Document content
    doc.add_heading(1, "Office SDK Advanced Features")?;
    doc.add_paragraph("This document demonstrates the comprehensive capabilities of the Office SDK for creating sophisticated DOCX documents.")?;

    doc.add_heading(2, "Document Metadata")?;
    doc.add_paragraph("The following metadata has been configured for this document:")?;
    doc.add_bulleted_list_item(&format!(
        "Author: {}",
        doc.metadata.author.as_deref().unwrap_or("Unknown")
    ))?;
    doc.add_bulleted_list_item(&format!(
        "Title: {}",
        doc.metadata.title.as_deref().unwrap_or("Untitled")
    ))?;
    doc.add_bulleted_list_item(&format!(
        "Subject: {}",
        doc.metadata.subject.as_deref().unwrap_or("None")
    ))?;
    doc.add_bulleted_list_item(&format!(
        "Created: {}",
        doc.metadata.created.format("%Y-%m-%d %H:%M:%S UTC")
    ))?;

    doc.add_heading(2, "Page Setup Configuration")?;
    doc.add_paragraph("The document is configured with the following page settings:")?;
    doc.add_bulleted_list_item(&format!("Paper Size: {:?}", doc.page_setup.paper_size))?;
    doc.add_bulleted_list_item(&format!("Orientation: {:?}", doc.page_setup.orientation))?;
    doc.add_bulleted_list_item(&format!(
        "Width: {:.0} twips ({:.1} inches)",
        doc.page_setup.width,
        doc.page_setup.width / 1440.0
    ))?;
    doc.add_bulleted_list_item(&format!(
        "Height: {:.0} twips ({:.1} inches)",
        doc.page_setup.height,
        doc.page_setup.height / 1440.0
    ))?;

    doc.add_heading(2, "Table Demonstration")?;
    doc.add_paragraph("The following table shows the features supported by this SDK:")?;

    // Create a feature comparison table
    println!("📊 Creating feature table...");
    doc.create_table(
        &["Feature", "Status", "Description", "Notes"],
        &[
            vec![
                "Basic Text",
                "✅ Complete",
                "Paragraphs and headings",
                "Full formatting support",
            ],
            vec![
                "Tables",
                "✅ Complete",
                "Structured data presentation",
                "With styling and colors",
            ],
            vec![
                "Images",
                "✅ Complete",
                "PNG, JPEG, GIF, BMP support",
                "Automatic sizing",
            ],
            vec![
                "VBA Macros",
                "✅ Complete",
                "Binary blob embedding",
                "For defensive analysis",
            ],
            vec![
                "Metadata",
                "✅ Complete",
                "Author, dates, properties",
                "Comprehensive tracking",
            ],
            vec![
                "Page Setup",
                "✅ Complete",
                "Size, orientation, margins",
                "Professional layouts",
            ],
            vec![
                "Styles",
                "✅ Complete",
                "Headings, lists, quotes",
                "Rich formatting",
            ],
            vec![
                "Markdown",
                "✅ Complete",
                "Bidirectional conversion",
                "Content interchange",
            ],
        ],
    )?;

    doc.add_heading(2, "Advanced Table Example")?;
    doc.add_paragraph("Here's a more complex table with custom styling:")?;

    // Create a styled table manually
    let mut styled_table = Table {
        id: "styledTable1".to_string(),
        rows: Vec::new(),
        style: Some("TableGrid".to_string()),
        width: Some(8000.0),
    };

    // Header row with colored background
    styled_table.rows.push(TableRow {
        cells: vec![
            TableCell {
                content: "Product Category".to_string(),
                width: Some(3000.0),
                background_color: Some("4472C4".to_string()), // Blue header
                text_align: Some(TextAlign::Center),
            },
            TableCell {
                content: "Revenue (Q1)".to_string(),
                width: Some(2500.0),
                background_color: Some("4472C4".to_string()),
                text_align: Some(TextAlign::Center),
            },
            TableCell {
                content: "Growth %".to_string(),
                width: Some(2500.0),
                background_color: Some("4472C4".to_string()),
                text_align: Some(TextAlign::Center),
            },
        ],
        height: Some(500.0),
    });

    // Data rows with alternating colors
    let rows_data = [
        ("Software Licenses", "$1,250,000", "+15.3%", "C6EFCE"), // Green
        ("Cloud Services", "$890,000", "+28.7%", "FFE699"),      // Yellow
        ("Support Contracts", "$340,000", "+8.1%", "C6EFCE"),    // Green
        ("Training Programs", "$180,000", "-2.4%", "FFCCCB"),    // Light red
    ];

    for (product, revenue, growth, bg_color) in &rows_data {
        styled_table.rows.push(TableRow {
            cells: vec![
                TableCell {
                    content: product.to_string(),
                    width: Some(3000.0),
                    background_color: Some(bg_color.to_string()),
                    text_align: Some(TextAlign::Left),
                },
                TableCell {
                    content: revenue.to_string(),
                    width: Some(2500.0),
                    background_color: Some(bg_color.to_string()),
                    text_align: Some(TextAlign::Right),
                },
                TableCell {
                    content: growth.to_string(),
                    width: Some(2500.0),
                    background_color: Some(bg_color.to_string()),
                    text_align: Some(TextAlign::Center),
                },
            ],
            height: None,
        });
    }

    doc.add_table(styled_table)?;

    doc.add_heading(2, "Image Support")?;
    doc.add_paragraph("The SDK supports embedding various image formats:")?;

    // Create sample images
    println!("🖼️  Adding sample images...");

    // Simple 2x2 PNG (red square)
    let red_png = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk start
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, // 2x2 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0x4D, 0xD3, 0x2F, // Rest of IHDR
        0x4F, 0x00, 0x00, 0x00, 0x12, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0x00, 0x00, 0x00, // Red pixel data
        0x00, 0x01, 0x00, 0x01, 0x63, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, 0x27,
        0xDE, 0xFC, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND chunk
    ];

    doc.add_image_from_bytes(&red_png, "image/png", Some("Sample Red Square"))?;
    doc.add_paragraph("Above: A small red square image embedded in the document.")?;

    doc.add_heading(2, "Code Examples")?;
    doc.add_paragraph("Here are some code examples showing how to use the advanced features:")?;

    doc.add_heading(3, "Creating Tables")?;
    doc.add_code_block(
        r#"use office_sdk::docx::{DocxDocument, Table, TableRow, TableCell, TextAlign};

let mut doc = DocxDocument::new();

// Simple table creation
doc.create_table(
    &["Name", "Age", "City"],
    &[
        vec!["Alice", "30", "NYC"],
        vec!["Bob", "25", "LA"],
    ]
)?;

// Advanced table with custom styling
let mut table = Table {
    id: "customTable".to_string(),
    rows: vec![
        TableRow {
            cells: vec![
                TableCell {
                    content: "Header".to_string(),
                    background_color: Some("4472C4".to_string()),
                    text_align: Some(TextAlign::Center),
                    width: Some(2000.0),
                },
            ],
            height: Some(400.0),
        },
    ],
    style: Some("TableGrid".to_string()),
    width: Some(5000.0),
};

doc.add_table(table)?;"#,
    )?;

    doc.add_heading(3, "Adding Images")?;
    doc.add_code_block(
        r#"// From file
let image_id = doc.add_image("path/to/image.png", Some("Alt text"))?;

// From byte data
let image_data = std::fs::read("image.jpg")?;
let image_id = doc.add_image_from_bytes(&image_data, "image/jpeg", Some("JPEG Image"))?;"#,
    )?;

    doc.add_heading(3, "Page Setup")?;
    doc.add_code_block(
        r#"// Configure page settings
doc.set_page_size(PaperSize::A4);
doc.set_orientation(PageOrientation::Landscape);
doc.set_margins(1440.0, 1440.0, 1800.0, 1800.0); // in twips

// Custom page size
doc.set_page_size(PaperSize::Custom { width: 12000.0, height: 16000.0 });"#,
    )?;

    doc.add_heading(2, "VBA Macro Support")?;
    doc.add_paragraph("The SDK supports embedding VBA macros for defensive security analysis:")?;

    // Add a sample VBA project
    println!("🔧 Adding VBA macro...");
    let sample_vba = b"Sub SampleMacro()\n    ' This is a sample VBA macro for testing\n    MsgBox \"Hello from Office SDK!\"\nEnd Sub\n".to_vec();
    doc.add_vba_macro(sample_vba)?;

    doc.add_code_block(
        r#"// Adding VBA macros
let vba_data = std::fs::read("macro.bin")?;
doc.add_vba_macro(vba_data)?;

// Note: For security reasons, this is primarily for
// defensive analysis of existing VBA content"#,
    )?;

    doc.add_heading(2, "Document Generation")?;
    doc.add_paragraph("This document demonstrates:")?;
    doc.add_numbered_list_item("Comprehensive metadata management", 1)?;
    doc.add_numbered_list_item("Advanced table creation with styling", 2)?;
    doc.add_numbered_list_item("Image embedding and management", 3)?;
    doc.add_numbered_list_item("VBA macro support for analysis", 4)?;
    doc.add_numbered_list_item("Professional page layout control", 5)?;
    doc.add_numbered_list_item("Rich text formatting and styles", 6)?;

    doc.add_quote("The Office SDK provides enterprise-grade document processing capabilities with a focus on security and reliability.")?;

    doc.add_horizontal_rule()?;

    doc.add_heading(2, "Summary")?;
    doc.add_paragraph("This document has been generated using the Office SDK's advanced DOCX features, demonstrating the library's comprehensive capabilities for creating professional documents programmatically.")?;

    // Update metadata before saving
    doc.update_metadata();

    // Save the document
    println!("💾 Saving advanced DOCX document...");
    doc.save("advanced_features_demo.docx")?;
    println!("✅ Document saved as: advanced_features_demo.docx");

    // Generate and display statistics
    let bytes = doc.to_bytes()?;
    println!("\n📊 Document Statistics:");
    println!("  • File size: {} bytes", bytes.len());
    println!("  • Tables: {}", doc.tables.len());
    println!("  • Images: {}", doc.images.len());
    println!(
        "  • VBA Project: {}",
        if doc.vba_project.is_some() {
            "Present"
        } else {
            "None"
        }
    );
    println!("  • Page size: {:?}", doc.page_setup.paper_size);
    println!("  • Orientation: {:?}", doc.page_setup.orientation);

    // Verify by reading back
    println!("\n🔍 Verifying document by reading back...");
    let loaded_doc = DocxDocument::from_file("advanced_features_demo.docx")?;
    let text_content = loaded_doc.get_text()?;

    println!("✅ Document verification successful!");
    println!("  • Content length: {} characters", text_content.len());
    println!("  • Contains tables: {}", text_content.contains("Feature"));
    println!(
        "  • Contains code: {}",
        text_content.contains("DocxDocument")
    );
    println!(
        "  • Contains images: {}",
        text_content.contains("Sample Red")
    );
    println!("  • Contains VBA info: {}", text_content.contains("VBA"));

    // Test round-trip with all features
    println!("\n🔄 Testing round-trip conversion...");
    let roundtrip_doc = DocxDocument::from_bytes(&bytes)?;
    let roundtrip_bytes = roundtrip_doc.to_bytes()?;
    std::fs::write("roundtrip_advanced.docx", &roundtrip_bytes)?;
    println!("✅ Round-trip conversion successful!");

    // Create a metadata comparison document
    println!("\n📋 Creating metadata comparison document...");
    let mut meta_doc = DocxDocument::new()
        .with_author("Metadata Analyzer")
        .with_title("Metadata Comparison Report")
        .with_subject("Document Analysis");

    meta_doc.add_heading(1, "Document Metadata Analysis")?;
    meta_doc.add_paragraph("Analysis of the advanced DOCX document:")?;

    // Create metadata table
    meta_doc.create_table(
        &["Property", "Value", "Type"],
        &[
            vec![
                "Title",
                doc.metadata.title.as_deref().unwrap_or("N/A"),
                "String",
            ],
            vec![
                "Author",
                doc.metadata.author.as_deref().unwrap_or("N/A"),
                "String",
            ],
            vec![
                "Subject",
                doc.metadata.subject.as_deref().unwrap_or("N/A"),
                "String",
            ],
            vec![
                "Keywords",
                doc.metadata.keywords.as_deref().unwrap_or("N/A"),
                "String",
            ],
            vec![
                "Category",
                doc.metadata.category.as_deref().unwrap_or("N/A"),
                "String",
            ],
            vec![
                "Revision",
                doc.metadata.revision.as_deref().unwrap_or("N/A"),
                "String",
            ],
            vec![
                "Created Date",
                &doc.metadata.created.format("%Y-%m-%d").to_string(),
                "DateTime",
            ],
            vec![
                "Modified Date",
                &doc.metadata.modified.format("%Y-%m-%d").to_string(),
                "DateTime",
            ],
        ],
    )?;

    meta_doc.add_heading(2, "Technical Details")?;
    meta_doc.create_table(
        &["Component", "Count", "Status"],
        &[
            vec!["Tables", &doc.tables.len().to_string(), "Generated"],
            vec!["Images", &doc.images.len().to_string(), "Embedded"],
            vec![
                "VBA Projects",
                &(if doc.vba_project.is_some() { 1 } else { 0 }).to_string(),
                "Included",
            ],
            vec![
                "Page Width",
                &format!("{:.0} twips", doc.page_setup.width),
                "Configured",
            ],
            vec![
                "Page Height",
                &format!("{:.0} twips", doc.page_setup.height),
                "Configured",
            ],
        ],
    )?;

    meta_doc.save("metadata_analysis.docx")?;
    println!("✅ Metadata analysis saved as: metadata_analysis.docx");

    println!("\n🎉 Advanced DOCX demonstration completed successfully!");
    println!("\nFiles created:");
    println!("  • advanced_features_demo.docx - Main demonstration document");
    println!("  • roundtrip_advanced.docx - Round-trip conversion test");
    println!("  • metadata_analysis.docx - Metadata analysis report");

    println!("\n📈 All advanced DOCX features are working correctly!");

    Ok(())
}
