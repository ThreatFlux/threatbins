use office_sdk::OfficeDocument;
use office_sdk::pptx::PptxDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PptxDocument::new();

    // Add title slide
    doc.add_slide(
        "Welcome to Office SDK",
        &[
            "A Rust library for Office document processing",
            "Supports DOCX, XLSX, and PPTX formats",
        ],
    )?;

    // Add feature overview slide
    doc.add_slide(
        "Key Features",
        &[
            "✓ Create and parse Office documents",
            "✓ Memory-efficient ZIP-based processing",
            "✓ Type-safe APIs with comprehensive error handling",
            "✓ Built with Rust 1.89.0",
        ],
    )?;

    // Add code example slide
    doc.add_slide(
        "Example Usage",
        &[
            "let mut doc = PptxDocument::new();",
            "doc.add_slide(\"Title\", &[\"Content\"]);",
            "doc.save(\"presentation.pptx\")?;",
        ],
    )?;

    // Add supported formats slide
    doc.add_slide(
        "Supported Formats",
        &[
            "DOCX - Word documents",
            "XLSX - Excel spreadsheets",
            "PPTX - PowerPoint presentations",
            "Full OpenXML compatibility",
        ],
    )?;

    // Save the presentation
    doc.save("example.pptx")?;
    println!("PPTX presentation created successfully: example.pptx");

    // Load and analyze the presentation
    let loaded_doc = PptxDocument::from_file("example.pptx")?;
    let slides = loaded_doc.get_all_slides()?;

    println!("\nPresentation contains {} slides:", slides.len());
    for (i, slide) in slides.iter().enumerate() {
        println!("\nSlide {}: {}", i + 1, slide.title);
        for (j, content) in slide.content.iter().enumerate() {
            println!("  {}. {}", j + 1, content);
        }
    }

    // Extract all text content
    let all_text = loaded_doc.get_slide_text()?;
    println!("\n--- Complete Presentation Text ---");
    println!("{}", all_text);

    Ok(())
}
