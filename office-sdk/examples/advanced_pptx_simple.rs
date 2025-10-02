use office_sdk::OfficeDocument;
use office_sdk::pptx::{AnimationType, PptxDocument, SlideSize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK Advanced PPTX Demo ===\n");

    // Create a new advanced PPTX presentation
    let mut doc = PptxDocument::new();

    // Step 1: Set comprehensive metadata
    println!("🎯 Step 1: Setting presentation metadata...");
    doc.set_author("Office SDK Presentation Team");
    doc.set_title("Q4 2024 Business Review");
    doc.set_subject("Quarterly Performance and Strategy");
    doc.set_description("Comprehensive quarterly business review showcasing performance metrics");
    doc.set_keywords("business, quarterly, review, performance, strategy, powerpoint");
    doc.set_category("Business Presentations");

    // Step 2: Configure slide format
    println!("📐 Step 2: Configuring slide format...");
    doc.set_slide_size(SlideSize::Widescreen16x9);

    // Step 3: Create title slide
    println!("🎬 Step 3: Creating title slide...");
    let title_slide_id = doc.add_title_slide(
        "Q4 2024 Business Review",
        "Excellence in Performance • Innovation in Strategy",
    )?;
    doc.add_slide_notes(
        &title_slide_id,
        "Welcome everyone to our comprehensive Q4 2024 business review.",
    )?;

    // Step 4: Add executive summary slide
    println!("📊 Step 4: Creating executive summary...");
    let _executive_slide_id = doc.add_content_slide(
        "Executive Summary",
        &[
            "🎯 Record-breaking revenue: $62.4M (+45% YoY)",
            "💰 Net profit margin: 37.5% (industry-leading)",
            "👥 Team growth: 6 key strategic hires",
            "🚀 Product launches: 3 major innovations",
            "🌍 Market expansion: 2 new international markets",
        ],
    )?;

    // Step 5: Add images
    println!("🖼️ Step 5: Adding visual elements...");

    // Create company logo
    let logo_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk header
        0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x40, // 64x64 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0x25, 0x0B, 0xE6, 0x89,
    ];

    let _logo_id = doc.add_image_from_bytes(&logo_data, "image/png", Some("Company Logo"))?;

    // Step 6: Add animations
    println!("✨ Step 6: Adding slide animations...");

    doc.add_animation("title", AnimationType::FadeIn, 1.5, 0.0)?;
    doc.add_animation("subtitle", AnimationType::SlideIn, 2.0, 1.0)?;

    // Step 7: Add VBA automation
    println!("🔧 Step 7: Adding VBA automation...");

    let vba_macros =
        b"Sub StartPresentation()\n    MsgBox \"Presentation started!\", vbInformation\nEnd Sub"
            .to_vec();
    doc.add_vba_macro(vba_macros)?;

    // Step 8: Generate and save the presentation
    println!("\n💾 Step 8: Generating PowerPoint file...");
    doc.save("q4_business_review.pptx")?;
    println!("   ✅ Saved as: q4_business_review.pptx");

    // Step 9: Display presentation statistics
    println!("\n📊 Step 9: Presentation Statistics:");
    let stats = doc.get_document_stats();
    println!("   • Total slides: {}", stats.slide_count);
    println!("   • Images: {}", stats.image_count);
    println!("   • Animations: {}", stats.animation_count);
    println!("   • Speaker notes: {}", stats.notes_count);
    println!(
        "   • VBA macros: {}",
        if stats.has_vba { "Yes" } else { "No" }
    );
    println!("   • Author: {}", stats.author);

    println!("\n🎉 Advanced PPTX Demo Completed Successfully!");

    Ok(())
}
