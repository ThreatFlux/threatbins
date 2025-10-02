use office_sdk::OfficeDocument;
use office_sdk::xlsx::{ChartType, PageOrientation, PaperSize, XlsxDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK Advanced XLSX Demo ===\n");

    // Create a new advanced XLSX document
    let mut doc = XlsxDocument::new();

    // Step 1: Set comprehensive metadata
    println!("📊 Step 1: Setting document metadata...");
    doc.set_author("Office SDK Demo");
    doc.set_title("Q4 2024 Financial Report");
    doc.set_subject("Quarterly Business Analysis");
    doc.set_description("Comprehensive financial and operational analysis for Q4 2024");
    doc.set_keywords("financial, quarterly, analysis, excel, data");
    doc.set_category("Business Reports");

    // Step 2: Configure page setup
    println!("📄 Step 2: Configuring page setup...");
    doc.set_page_size(PaperSize::A4);
    doc.set_orientation(PageOrientation::Landscape);
    doc.set_margins(0.75, 0.75, 1.0, 1.0);
    doc.set_scale(85);

    // Step 3: Create comprehensive worksheets
    println!("📈 Step 3: Creating worksheets with data...");

    // Financial Summary worksheet
    let financial_data = vec![
        vec![
            "Metric".to_string(),
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
            "Total".to_string(),
        ],
        vec![
            "Revenue ($M)".to_string(),
            "12.5".to_string(),
            "14.2".to_string(),
            "16.8".to_string(),
            "18.9".to_string(),
            "62.4".to_string(),
        ],
        vec![
            "Expenses ($M)".to_string(),
            "8.3".to_string(),
            "9.1".to_string(),
            "10.2".to_string(),
            "11.4".to_string(),
            "39.0".to_string(),
        ],
        vec![
            "Net Income ($M)".to_string(),
            "4.2".to_string(),
            "5.1".to_string(),
            "6.6".to_string(),
            "7.5".to_string(),
            "23.4".to_string(),
        ],
        vec![
            "Profit Margin (%)".to_string(),
            "33.6".to_string(),
            "35.9".to_string(),
            "39.3".to_string(),
            "39.7".to_string(),
            "37.5".to_string(),
        ],
    ];
    doc.add_worksheet("Financial Summary", &financial_data)?;

    // Step 4: Add charts for data visualization
    println!("📊 Step 4: Adding charts and visualizations...");

    // Revenue trend chart
    doc.add_chart(
        ChartType::Column,
        "Quarterly Revenue Trend",
        "A1:E2",
        600,
        400,
    )?;

    // Step 5: Add images
    println!("🖼️ Step 5: Adding images...");

    // Create a simple company logo (minimal PNG data)
    let logo_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk header
        0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, // 32x32 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0xFC, 0x18, 0xED, 0xA3,
    ];

    let _logo_id = doc.add_image_from_bytes(&logo_data, "image/png", Some("Company Logo"))?;

    // Step 6: Add VBA macro for automation
    println!("🔧 Step 6: Adding VBA macros...");

    let vba_macro = b"Sub GenerateReport()\n    MsgBox \"Report generated successfully!\", vbInformation\nEnd Sub".to_vec();
    doc.add_vba_macro(vba_macro)?;

    // Step 7: Generate and save the document
    println!("\n💾 Step 7: Generating Excel file...");
    doc.save("advanced_financial_report.xlsx")?;
    println!("   ✅ Saved as: advanced_financial_report.xlsx");

    // Step 8: Display document statistics
    println!("\n📊 Step 8: Document Statistics:");
    let stats = doc.get_document_stats();
    println!("   • Worksheets: {}", stats.worksheet_count);
    println!("   • Images: {}", stats.image_count);
    println!("   • Charts: {}", stats.chart_count);
    println!("   • Tables: {}", stats.table_count);
    println!("   • Total cells: {}", stats.total_cells);
    println!(
        "   • VBA macros: {}",
        if stats.has_vba { "Yes" } else { "No" }
    );
    println!("   • Author: {}", stats.author);

    println!("\n🎉 Advanced XLSX Demo Completed Successfully!");

    Ok(())
}
