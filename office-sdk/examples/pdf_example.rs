use office_sdk::OfficeDocument;
use office_sdk::pdf::{Color, PdfDocument, TextStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK PDF Example ===\n");

    // Create a new PDF document
    let mut doc = PdfDocument::new();

    // Set document metadata
    doc.set_title("Simple PDF Example");
    doc.set_author("Office SDK");
    doc.set_subject("Basic PDF Creation");
    doc.set_keywords("pdf, example, rust, office-sdk");

    // Step 1: Add pages
    println!("📄 Adding pages...");
    let page1_id = doc.add_a4_page()?;
    let page2_id = doc.add_letter_page()?;

    // Step 2: Add text content
    println!("📝 Adding text content...");

    // Page 1: Title and introduction
    doc.add_heading(0, 1, "Welcome to PDF SDK", 100.0, 750.0)?;
    doc.add_text(
        0,
        "This is a simple example of PDF creation using the Office SDK.",
        100.0,
        720.0,
        "Helvetica",
        12.0,
    )?;

    // Add some styled text
    let bold_style = TextStyle {
        bold: true,
        italic: false,
        underline: false,
        strikethrough: false,
        superscript: false,
        subscript: false,
    };

    doc.add_styled_text(
        0,
        "Key Features:",
        100.0,
        680.0,
        "Helvetica-Bold",
        14.0,
        Color::blue(),
        bold_style,
    )?;
    doc.add_text(
        0,
        "• Text rendering with multiple fonts and styles",
        120.0,
        660.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_text(
        0,
        "• Image embedding and positioning",
        120.0,
        645.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_text(
        0,
        "• Interactive forms and annotations",
        120.0,
        630.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_text(
        0,
        "• Document security and encryption",
        120.0,
        615.0,
        "Helvetica",
        11.0,
    )?;

    // Step 3: Add shapes and graphics
    println!("🎨 Adding graphics...");
    doc.add_rectangle(0, 100.0, 550.0, 350.0, 3.0, None, Some(Color::gray(0.7)))?;
    doc.add_circle(
        0,
        450.0,
        580.0,
        30.0,
        Some(Color::yellow()),
        Some(Color::red()),
    )?;
    doc.add_line(0, 100.0, 570.0, 400.0, 570.0, Color::blue(), 2.0)?;

    // Page 2: Table and form example
    doc.add_heading(1, 1, "Data and Interaction", 100.0, 750.0)?;

    // Add a simple table
    doc.add_table(
        1,
        100.0,
        650.0,
        &["Product", "Q4 Sales", "Growth"],
        &[
            vec!["Laptop Pro", "$125K", "+15%"],
            vec!["Tablet", "$89K", "+32%"],
            vec!["Phone", "$156K", "+8%"],
        ],
    )?;

    // Add form fields
    doc.add_text(1, "Feedback Form:", 100.0, 500.0, "Helvetica-Bold", 12.0)?;
    doc.add_text_field(1, "name", 100.0, 470.0, 200.0, 20.0, "Your name")?;
    doc.add_text(1, "Name", 310.0, 475.0, "Helvetica", 10.0)?;

    doc.add_checkbox(1, "subscribe", 100.0, 440.0, 15.0, false)?;
    doc.add_text(1, "Subscribe to updates", 125.0, 445.0, "Helvetica", 10.0)?;

    // Step 4: Add bookmarks
    println!("🔖 Adding navigation...");
    doc.add_bookmark("Introduction", 0, 100.0, 750.0)?;
    doc.add_bookmark("Data & Forms", 1, 100.0, 750.0)?;

    // Step 5: Generate document
    println!("\n💾 Generating PDF...");
    doc.save("simple_example.pdf")?;

    // Display statistics
    let stats = doc.get_document_stats();
    println!("✅ PDF created successfully!");
    println!("   • Pages: {}", stats.page_count);
    println!("   • Images: {}", stats.image_count);
    println!("   • Form fields: {}", stats.form_field_count);
    println!("   • Bookmarks: {}", stats.bookmark_count);
    println!("   • File: simple_example.pdf");

    Ok(())
}
