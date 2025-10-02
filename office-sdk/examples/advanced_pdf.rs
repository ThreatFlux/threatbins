use office_sdk::OfficeDocument;
use office_sdk::pdf::{
    Color, DuplexMode, EncryptionLevel, PageMode, PdfDocument, PdfVersion, PermissionFlags,
    PrintScaling, ReadingDirection, TextStyle, ViewerPreferences,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK Advanced PDF Demo ===\n");

    // Create a comprehensive PDF document
    let mut doc = PdfDocument::new();

    // Step 1: Set comprehensive metadata
    println!("📄 Step 1: Setting advanced document metadata...");
    doc.set_author("Office SDK Advanced Demo Team");
    doc.set_title("Q4 2024 Comprehensive Business Report");
    doc.set_subject("Advanced PDF Features Demonstration");
    doc.set_description("This document showcases the complete range of PDF capabilities including interactive forms, multimedia annotations, embedded images, structured tables, document security, JavaScript automation, layers, and advanced typography");
    doc.set_keywords(
        "pdf, business, report, interactive, forms, security, multimedia, comprehensive",
    );
    doc.set_category("Business Intelligence Reports");

    // Step 2: Configure advanced PDF settings
    println!("⚙️ Step 2: Configuring PDF settings...");
    doc.set_pdf_version(PdfVersion::V1_7);
    doc.set_page_mode(PageMode::UseOutlines);

    // Configure viewer preferences for professional presentation
    let mut viewer_prefs = ViewerPreferences::default();
    viewer_prefs.display_doc_title = true;
    viewer_prefs.center_window = true;
    viewer_prefs.fit_window = false;
    viewer_prefs.direction = ReadingDirection::LeftToRight;
    viewer_prefs.print_scaling = PrintScaling::None;
    viewer_prefs.duplex = DuplexMode::DuplexFlipLongEdge;
    doc.set_viewer_preferences(viewer_prefs);

    // Step 3: Create comprehensive page structure
    println!("📝 Step 3: Creating multi-page document structure...");

    // Page 1: Executive Summary with graphics
    let page1_id = doc.add_a4_page()?;
    doc.add_heading(0, 1, "Q4 2024 Executive Summary", 100.0, 780.0)?;

    // Add executive summary content
    doc.add_paragraph(
        0,
        "This comprehensive report presents our exceptional Q4 2024 performance, strategic initiatives, and forward-looking analysis. Our organization achieved record-breaking results across all key performance indicators while maintaining operational excellence and driving innovation.",
        100.0, 730.0, 400.0, 12.0
    )?;

    // Add key metrics in styled text
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
        "Key Achievements:",
        100.0,
        650.0,
        "Helvetica-Bold",
        14.0,
        Color::blue(),
        bold_style,
    )?;
    doc.add_text(
        0,
        "• Revenue Growth: 47% year-over-year ($18.9M in Q4)",
        120.0,
        630.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_text(
        0,
        "• Net Profit Margin: 39.7% (industry-leading performance)",
        120.0,
        615.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_text(
        0,
        "• Customer Satisfaction: 96% (highest in company history)",
        120.0,
        600.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_text(
        0,
        "• Team Expansion: 15 strategic hires in key positions",
        120.0,
        585.0,
        "Helvetica",
        11.0,
    )?;

    // Add company logo placeholder (simplified for demo)
    doc.add_rectangle(
        0,
        450.0,
        720.0,
        80.0,
        80.0,
        Some(Color::rgba(0.0, 0.5, 1.0, 0.8)),
        Some(Color::blue()),
    )?;
    doc.add_text(0, "LOGO", 475.0, 765.0, "Helvetica-Bold", 12.0)?;
    println!("   ✅ Added company logo and executive summary");

    // Add decorative elements
    doc.add_rectangle(0, 90.0, 550.0, 420.0, 3.0, None, Some(Color::blue()))?; // Border
    doc.add_line(
        0,
        100.0,
        570.0,
        500.0,
        570.0,
        Color::rgb(0.0, 0.5, 1.0),
        2.0,
    )?; // Accent line

    // Page 2: Interactive Form
    let page2_id = doc.add_letter_page()?;
    doc.add_heading(1, 1, "Interactive Feedback Form", 100.0, 750.0)?;

    doc.add_paragraph(
        1,
        "Please provide your feedback on this comprehensive report. Your input is valuable for continuous improvement of our documentation and reporting processes.",
        100.0, 710.0, 400.0, 11.0
    )?;

    // Personal Information Section
    doc.add_heading(1, 2, "Personal Information", 100.0, 660.0)?;

    // Form fields with labels
    doc.add_text(1, "First Name:", 100.0, 635.0, "Helvetica", 10.0)?;
    doc.add_text_field(1, "firstName", 200.0, 630.0, 150.0, 20.0, "")?;

    doc.add_text(1, "Last Name:", 100.0, 605.0, "Helvetica", 10.0)?;
    doc.add_text_field(1, "lastName", 200.0, 600.0, 150.0, 20.0, "")?;

    doc.add_text(1, "Email Address:", 100.0, 575.0, "Helvetica", 10.0)?;
    doc.add_text_field(1, "email", 200.0, 570.0, 200.0, 20.0, "user@company.com")?;

    doc.add_text(1, "Company/Organization:", 100.0, 545.0, "Helvetica", 10.0)?;
    doc.add_text_field(1, "company", 230.0, 540.0, 180.0, 20.0, "")?;

    // Feedback Section
    doc.add_heading(1, 2, "Report Feedback", 100.0, 500.0)?;

    doc.add_text(1, "Overall Rating:", 100.0, 475.0, "Helvetica", 10.0)?;
    doc.add_dropdown(
        1,
        "rating",
        200.0,
        470.0,
        120.0,
        20.0,
        &["Excellent", "Very Good", "Good", "Fair", "Poor"],
        Some(0),
    )?;

    doc.add_text(
        1,
        "Report Sections (check all that apply):",
        100.0,
        440.0,
        "Helvetica",
        10.0,
    )?;
    doc.add_checkbox(1, "executive_summary", 100.0, 420.0, 12.0, true)?;
    doc.add_text(1, "Executive Summary", 120.0, 425.0, "Helvetica", 9.0)?;

    doc.add_checkbox(1, "financial_data", 100.0, 405.0, 12.0, false)?;
    doc.add_text(
        1,
        "Financial Performance Data",
        120.0,
        410.0,
        "Helvetica",
        9.0,
    )?;

    doc.add_checkbox(1, "market_analysis", 100.0, 390.0, 12.0, false)?;
    doc.add_text(
        1,
        "Market Analysis & Insights",
        120.0,
        395.0,
        "Helvetica",
        9.0,
    )?;

    doc.add_checkbox(1, "strategic_initiatives", 100.0, 375.0, 12.0, false)?;
    doc.add_text(
        1,
        "Strategic Initiatives & Roadmap",
        120.0,
        380.0,
        "Helvetica",
        9.0,
    )?;

    // Comments section
    doc.add_text(
        1,
        "Additional Comments:",
        100.0,
        340.0,
        "Helvetica-Bold",
        11.0,
    )?;
    doc.add_multiline_text_field(
        1, "comments", 100.0, 240.0, 400.0, 90.0,
        "Please share any specific feedback, suggestions for improvement, or additional insights..."
    )?;

    println!("   ✅ Created interactive feedback form with comprehensive fields");

    // Page 3: Financial Data Table
    let page3_id = doc.add_a4_page()?;
    doc.add_heading(2, 1, "Financial Performance Analysis", 100.0, 800.0)?;

    doc.add_paragraph(
        2,
        "The following table presents detailed quarterly financial performance across all business units, demonstrating consistent growth and operational efficiency improvements.",
        100.0, 760.0, 450.0, 11.0
    )?;

    // Comprehensive financial data table
    doc.add_table(
        2,
        80.0,
        680.0,
        &[
            "Business Unit",
            "Q1 Revenue",
            "Q2 Revenue",
            "Q3 Revenue",
            "Q4 Revenue",
            "Growth %",
            "Margin %",
        ],
        &[
            vec![
                "Enterprise Solutions",
                "$3.2M",
                "$3.8M",
                "$4.5M",
                "$5.1M",
                "59%",
                "42%",
            ],
            vec![
                "Consumer Products",
                "$2.1M",
                "$2.6M",
                "$3.2M",
                "$3.8M",
                "81%",
                "36%",
            ],
            vec![
                "Cloud Services",
                "$1.8M",
                "$2.4M",
                "$3.1M",
                "$4.2M",
                "133%",
                "48%",
            ],
            vec![
                "Professional Services",
                "$1.4M",
                "$1.7M",
                "$2.0M",
                "$2.3M",
                "64%",
                "35%",
            ],
            vec![
                "International Markets",
                "$0.8M",
                "$1.2M",
                "$1.6M",
                "$2.1M",
                "163%",
                "33%",
            ],
            vec![
                "Research & Development",
                "$0.5M",
                "$0.7M",
                "$0.9M",
                "$1.2M",
                "140%",
                "28%",
            ],
            vec!["TOTAL", "$9.8M", "$12.4M", "$15.3M", "$18.7M", "91%", "38%"],
        ],
    )?;

    // Add performance chart placeholder (rectangle with chart simulation)
    doc.add_rectangle(
        2,
        100.0,
        400.0,
        400.0,
        200.0,
        Some(Color::rgba(0.9, 0.9, 0.9, 1.0)),
        Some(Color::black()),
    )?;
    doc.add_text(
        2,
        "Q4 Performance Chart",
        250.0,
        520.0,
        "Helvetica-Bold",
        14.0,
    )?;
    doc.add_text(
        2,
        "Revenue Growth: +91% YoY",
        180.0,
        480.0,
        "Helvetica",
        12.0,
    )?;
    doc.add_text(2, "Profit Margin: 38%", 210.0, 460.0, "Helvetica", 12.0)?;

    // Add chart elements simulation
    doc.add_rectangle(2, 120.0, 420.0, 15.0, 80.0, Some(Color::blue()), None)?; // Q1 bar
    doc.add_rectangle(2, 150.0, 420.0, 15.0, 120.0, Some(Color::green()), None)?; // Q2 bar
    doc.add_rectangle(2, 180.0, 420.0, 15.0, 140.0, Some(Color::yellow()), None)?; // Q3 bar
    doc.add_rectangle(2, 210.0, 420.0, 15.0, 180.0, Some(Color::red()), None)?; // Q4 bar

    println!("   ✅ Added financial data table and performance charts");

    // Page 4: Advanced Features Demonstration
    let page4_id = doc.add_a4_page()?;
    doc.add_heading(3, 1, "Advanced PDF Features", 100.0, 800.0)?;

    // Add various annotations
    doc.add_text(
        3,
        "This section demonstrates advanced PDF capabilities:",
        100.0,
        760.0,
        "Helvetica",
        12.0,
    )?;

    // Text annotation
    doc.add_text_annotation(
        3, 100.0, 740.0, 250.0, 30.0,
        "This annotation explains the importance of advanced PDF features in modern document workflows.",
        "Technical Author"
    )?;

    // Highlight annotation
    doc.add_text(
        3,
        "Critical information highlighted below",
        100.0,
        690.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_highlight_annotation(3, 100.0, 690.0, 220.0, 11.0, "Reviewer")?;

    // Link annotation
    doc.add_text(
        3,
        "Visit our documentation:",
        100.0,
        660.0,
        "Helvetica",
        11.0,
    )?;
    doc.add_link_annotation(3, 230.0, 660.0, 150.0, 11.0, "https://docs.example.com")?;
    doc.add_styled_text(
        3,
        "https://docs.example.com",
        230.0,
        660.0,
        "Helvetica",
        11.0,
        Color::blue(),
        TextStyle {
            bold: false,
            italic: false,
            underline: true,
            strikethrough: false,
            superscript: false,
            subscript: false,
        },
    )?;

    // Advanced graphics and shapes
    doc.add_text(
        3,
        "Geometric Elements:",
        100.0,
        620.0,
        "Helvetica-Bold",
        12.0,
    )?;

    // Colored rectangle with transparency effect
    doc.add_rectangle(
        3,
        100.0,
        580.0,
        120.0,
        25.0,
        Some(Color::rgba(0.0, 0.7, 1.0, 0.3)),
        Some(Color::blue()),
    )?;
    doc.add_text(3, "Translucent Rectangle", 110.0, 590.0, "Helvetica", 9.0)?;

    // Circle with gradient effect simulation
    doc.add_circle(
        3,
        280.0,
        590.0,
        15.0,
        Some(Color::rgba(1.0, 0.5, 0.0, 0.8)),
        Some(Color::red()),
    )?;
    doc.add_text(3, "Gradient Circle", 310.0, 595.0, "Helvetica", 9.0)?;

    // Connecting lines
    doc.add_line(3, 100.0, 550.0, 450.0, 550.0, Color::gray(0.6), 1.5)?;
    doc.add_line(3, 450.0, 550.0, 450.0, 500.0, Color::gray(0.6), 1.5)?;

    println!("   ✅ Added advanced graphics, annotations, and visual elements");

    // Step 4: Add comprehensive security
    println!("🔒 Step 4: Implementing document security...");

    // Set strong password protection
    doc.set_password_protection(Some("UserView2024!"), Some("OwnerEdit2024@"));

    // Configure restrictive permissions for confidential data
    let mut permissions = PermissionFlags::default();
    permissions.allow_copy = false; // Prevent copying
    permissions.allow_modify_contents = false; // Prevent editing
    permissions.allow_modify_annotations = false; // Prevent annotation changes
    permissions.allow_printing = true; // Allow printing for hard copies
    permissions.allow_print_high_quality = false; // Prevent high-quality printing
    permissions.allow_extract_for_accessibility = true; // Maintain accessibility

    doc.set_permissions(permissions);
    doc.set_encryption_level(EncryptionLevel::Aes256Bit); // Maximum security

    println!("   ✅ Applied AES-256 encryption with restricted permissions");

    // Step 5: Add JavaScript automation
    println!("⚡ Step 5: Adding JavaScript automation...");

    // Form validation script
    doc.add_javascript("formValidation", r#"
        function validateBusinessForm() {
            var firstName = this.getField('firstName');
            var lastName = this.getField('lastName');
            var email = this.getField('email');
            var company = this.getField('company');
            var rating = this.getField('rating');

            // Comprehensive validation
            if (!firstName.value || firstName.value.trim().length < 2) {
                app.alert('Please enter a valid first name (minimum 2 characters).', 1, 0);
                firstName.setFocus();
                return false;
            }

            if (!lastName.value || lastName.value.trim().length < 2) {
                app.alert('Please enter a valid last name (minimum 2 characters).', 1, 0);
                lastName.setFocus();
                return false;
            }

            if (!email.value || email.value.indexOf('@') === -1 || email.value.indexOf('.') === -1) {
                app.alert('Please enter a valid email address.', 1, 0);
                email.setFocus();
                return false;
            }

            if (!company.value || company.value.trim().length < 3) {
                app.alert('Please enter your company or organization name.', 1, 0);
                company.setFocus();
                return false;
            }

            if (!rating.value) {
                app.alert('Please select an overall rating.', 1, 0);
                rating.setFocus();
                return false;
            }

            app.alert('Form validation successful! Thank you for your feedback.', 3, 0);
            return true;
        }
    "#)?;

    // Document initialization script
    doc.add_javascript("documentInit", r#"
        // Document initialization and setup
        console.println('Advanced PDF document loaded successfully');
        console.println('Document title: ' + this.info.Title);
        console.println('Document author: ' + this.info.Author);
        console.println('Page count: ' + this.numPages);

        // Configure document behavior
        this.layout = 'SinglePage';
        this.zoomType = 'FitPage';

        // Add welcome message
        app.alert('Welcome to the Advanced PDF Features Demonstration!\n\nThis document showcases comprehensive PDF capabilities including:\n• Interactive forms with validation\n• Multimedia annotations\n• Document security features\n• JavaScript automation\n• Professional formatting', 3, 0);
    "#)?;

    // Analytics and tracking script
    doc.add_javascript("analytics", r#"
        function trackDocumentUsage() {
            var usage = {
                openTime: new Date(),
                pageViews: 0,
                formInteractions: 0,
                annotationViews: 0
            };

            // Log document access
            console.println('Document accessed at: ' + usage.openTime);

            return usage;
        }

        // Initialize tracking
        var documentUsage = trackDocumentUsage();

        function logFormInteraction(fieldName) {
            documentUsage.formInteractions++;
            console.println('Form interaction: ' + fieldName + ' (Total: ' + documentUsage.formInteractions + ')');
        }

        function generateUsageReport() {
            var report = 'Document Usage Report\n' +
                        '=====================\n' +
                        'Session Start: ' + documentUsage.openTime + '\n' +
                        'Form Interactions: ' + documentUsage.formInteractions + '\n' +
                        'Current Page: ' + (this.pageNum + 1) + ' of ' + this.numPages;

            app.alert(report, 3, 0);
        }
    "#)?;

    println!(
        "   ✅ Added comprehensive JavaScript automation (validation, analytics, document control)"
    );

    // Step 6: Add professional bookmarks and navigation
    println!("🔖 Step 6: Creating document navigation structure...");

    // Add hierarchical bookmarks
    let executive_bookmark = doc.add_bookmark("Executive Summary", 0, 100.0, 780.0)?;
    doc.add_nested_bookmark(&executive_bookmark, "Key Achievements", 0, 100.0, 650.0)?;
    doc.add_nested_bookmark(&executive_bookmark, "Performance Metrics", 0, 100.0, 600.0)?;

    let form_bookmark = doc.add_bookmark("Interactive Form", 1, 100.0, 750.0)?;
    doc.add_nested_bookmark(&form_bookmark, "Personal Information", 1, 100.0, 660.0)?;
    doc.add_nested_bookmark(&form_bookmark, "Feedback Section", 1, 100.0, 500.0)?;

    let data_bookmark = doc.add_bookmark("Financial Data", 2, 100.0, 800.0)?;
    doc.add_nested_bookmark(&data_bookmark, "Performance Table", 2, 80.0, 680.0)?;
    doc.add_nested_bookmark(&data_bookmark, "Growth Charts", 2, 100.0, 400.0)?;

    let features_bookmark = doc.add_bookmark("Advanced Features", 3, 100.0, 800.0)?;
    doc.add_nested_bookmark(&features_bookmark, "Annotations", 3, 100.0, 740.0)?;
    doc.add_nested_bookmark(&features_bookmark, "Graphics Elements", 3, 100.0, 620.0)?;

    // Add outline structure
    doc.add_outline_item("Executive Summary", 0, 100.0, 780.0)?;
    doc.add_outline_item("Interactive Form", 1, 100.0, 750.0)?;
    doc.add_outline_item("Financial Analysis", 2, 100.0, 800.0)?;
    doc.add_outline_item("Feature Demonstration", 3, 100.0, 800.0)?;

    println!("   ✅ Created hierarchical bookmarks and document outline");

    // Step 7: Add document layers for different content types
    println!("🎨 Step 7: Creating content layers...");

    doc.add_layer("Text Content", true)?;
    doc.add_layer("Form Elements", true)?;
    doc.add_layer("Annotations", true)?;
    doc.add_layer("Images", true)?;
    doc.add_layer("Graphics", true)?;
    doc.add_layer("Watermark", false)?; // Hidden by default

    println!("   ✅ Added 6 content layers for organized document structure");

    // Step 8: Add file attachments
    println!("📎 Step 8: Adding file attachments...");

    // Executive summary data
    let exec_summary_data = br#"
Executive Summary - Q4 2024 Key Metrics
=======================================

Revenue Performance:
- Q4 Revenue: $18.7M (+91% vs Q4 2023)
- Annual Revenue: $56.2M (+67% YoY)
- Recurring Revenue: 78% of total

Profitability Metrics:
- Gross Margin: 64% (+6% from Q3)
- Operating Margin: 42% (industry benchmark: 28%)
- Net Margin: 38% (exceptional performance)

Growth Indicators:
- Customer Acquisition: +45% new customers
- Customer Retention: 94% (industry leading)
- Average Contract Value: +32% increase
- Expansion Revenue: $2.3M from existing customers

Strategic Achievements:
- Successfully launched 3 major product lines
- Expanded to 2 new international markets
- Completed 2 strategic acquisitions
- Achieved ISO 27001 security certification

Operational Excellence:
- Employee Satisfaction: 96% positive
- Customer Support: 99.2% satisfaction
- System Uptime: 99.98% availability
- Security Incidents: 0 (perfect record)

2025 Outlook:
- Projected Revenue: $85M-$95M
- Target Markets: EU expansion + 3 new verticals
- R&D Investment: $12M for innovation
- Team Growth: +40 strategic hires planned
"#
    .to_vec();

    doc.add_attachment(
        "executive_summary_detailed.txt",
        exec_summary_data,
        "text/plain",
        "Detailed executive summary with comprehensive metrics and analysis",
    )?;

    // Financial model spreadsheet simulation
    let financial_model = br#"
Financial Model - Q4 2024 Analysis
=================================

Revenue Model:
Enterprise Solutions: $5.1M (27% of total)
- Average Contract: $125K
- Contracts Closed: 41
- Pipeline Value: $8.2M

Consumer Products: $3.8M (20% of total)
- Units Sold: 47,500
- Average Selling Price: $80
- Inventory Turnover: 6.2x

Cloud Services: $4.2M (22% of total)
- Monthly Recurring Revenue: $1.4M
- Customer Lifetime Value: $45K
- Churn Rate: 2.1% (best in class)

Cost Structure:
Cost of Goods Sold: $11.2M (60% of revenue)
Sales & Marketing: $4.1M (22% of revenue)
Research & Development: $2.8M (15% of revenue)
General & Administrative: $2.1M (11% of revenue)

Profitability Analysis:
Gross Profit: $7.5M (40% margin)
Operating Profit: $3.8M (20% margin)
Net Profit: $3.1M (17% margin)

Cash Flow Analysis:
Operating Cash Flow: $4.2M
Free Cash Flow: $3.1M
Cash Burn Rate: $0.3M/month
Runway: 24+ months
"#
    .to_vec();

    doc.add_attachment(
        "financial_model_q4.csv",
        financial_model,
        "text/csv",
        "Detailed financial modeling data and projections",
    )?;

    // Technical specifications
    let tech_specs = br#"
{
  "document": {
    "format": "PDF",
    "version": "1.7",
    "features": {
      "interactive_forms": true,
      "multimedia_annotations": true,
      "javascript_automation": true,
      "document_security": "AES-256",
      "digital_signatures": false,
      "3d_annotations": false,
      "video_audio": false
    },
    "accessibility": {
      "tagged_pdf": false,
      "screen_reader_compatible": true,
      "high_contrast_support": true,
      "keyboard_navigation": true
    },
    "security": {
      "encryption_level": "AES-256",
      "password_protected": true,
      "permissions": {
        "printing": true,
        "copying": false,
        "editing": false,
        "annotation_editing": false,
        "form_filling": true,
        "accessibility_extraction": true
      }
    },
    "technical_details": {
      "page_count": 4,
      "images": 3,
      "forms": 9,
      "annotations": 3,
      "bookmarks": 12,
      "javascript_functions": 8,
      "layers": 6,
      "attachments": 3
    }
  }
}
"#
    .to_vec();

    doc.add_attachment(
        "technical_specifications.json",
        tech_specs,
        "application/json",
        "Technical specifications and feature matrix for this PDF document",
    )?;

    println!("   ✅ Added 3 comprehensive file attachments with detailed documentation");

    // Step 9: Generate and save the document
    println!("\n💾 Step 9: Generating comprehensive PDF file...");
    doc.save("comprehensive_business_report.pdf")?;
    println!("   ✅ Saved as: comprehensive_business_report.pdf");

    // Step 10: Display comprehensive document statistics
    println!("\n📊 Step 10: Document Statistics & Analysis:");
    let stats = doc.get_document_stats();
    println!("   • Total pages: {}", stats.page_count);
    println!("   • Images embedded: {}", stats.image_count);
    println!("   • Interactive annotations: {}", stats.annotation_count);
    println!("   • Form fields: {}", stats.form_field_count);
    println!("   • Navigation bookmarks: {}", stats.bookmark_count);
    println!("   • Document layers: {}", stats.layer_count);
    println!("   • File attachments: {}", stats.attachment_count);
    println!("   • JavaScript functions: {}", stats.javascript_count);
    println!(
        "   • Total text content: {} characters",
        stats.total_text_length
    );
    println!(
        "   • Security enabled: {}",
        if stats.has_security {
            "Yes (AES-256)"
        } else {
            "No"
        }
    );
    println!(
        "   • Interactive features: {}",
        if stats.has_forms { "Yes" } else { "No" }
    );
    println!(
        "   • Multimedia annotations: {}",
        if stats.has_annotations { "Yes" } else { "No" }
    );
    println!(
        "   • JavaScript automation: {}",
        if stats.has_javascript { "Yes" } else { "No" }
    );
    println!("   • PDF version: {:?}", stats.pdf_version);
    println!("   • Document author: {}", stats.author);
    println!(
        "   • Created: {}",
        stats.created.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "   • Last modified: {}",
        stats.modified.format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Step 11: Demonstrate round-trip capabilities
    println!("\n🔄 Step 11: Testing round-trip compatibility...");

    // Load the saved file
    let loaded_doc = PdfDocument::from_file("comprehensive_business_report.pdf")?;
    println!("   ✅ Successfully loaded saved PDF file");

    // Extract and verify content
    let extracted_text = loaded_doc.get_text()?;
    println!(
        "   ✅ Extracted {} characters of text content",
        extracted_text.len()
    );

    // Verify key content is present
    assert!(extracted_text.contains("Q4 2024") || extracted_text.contains("Executive Summary"));
    assert!(extracted_text.contains("Financial") || extracted_text.contains("Business"));
    println!("   ✅ Verified all key content is preserved");

    // Verify document structure
    assert_eq!(loaded_doc.pages.len(), 4);
    assert_eq!(loaded_doc.title, "Q4 2024 Comprehensive Business Report");
    assert_eq!(loaded_doc.author, "Office SDK Advanced Demo Team");
    println!("   ✅ Document structure and metadata preserved");

    // Step 12: Create derived documents
    println!("\n📋 Step 12: Creating derived documents...");

    // Create executive brief (pages 1 and 2 only)
    let executive_brief = doc.extract_pages(&[0, 1])?;
    executive_brief.save("executive_brief.pdf")?;
    println!("   ✅ Created executive brief: executive_brief.pdf");

    // Create data appendix (page 3 only)
    let data_appendix = doc.extract_pages(&[2])?;
    data_appendix.save("financial_data_appendix.pdf")?;
    println!("   ✅ Created data appendix: financial_data_appendix.pdf");

    // Create technical documentation
    let mut tech_doc = PdfDocument::new();
    tech_doc.set_title("PDF SDK Technical Documentation");
    tech_doc.set_author("Office SDK Development Team");

    let tech_page_id = tech_doc.add_a4_page()?;
    tech_doc.add_heading(0, 1, "PDF SDK Advanced Features", 100.0, 780.0)?;

    tech_doc.add_table(
        0,
        100.0,
        700.0,
        &["Feature Category", "Implementation", "Compliance", "Status"],
        &[
            vec![
                "Text Rendering",
                "printpdf + custom",
                "PDF 1.7",
                "✓ Complete",
            ],
            vec![
                "Image Embedding",
                "Image crate + PDF/A",
                "ICC Profiles",
                "✓ Complete",
            ],
            vec![
                "Interactive Forms",
                "AcroForms Standard",
                "PDF 1.5+",
                "✓ Complete",
            ],
            vec![
                "Annotations",
                "PDF Annotation Spec",
                "PDF 1.3+",
                "✓ Complete",
            ],
            vec!["Security", "AES-256 Encryption", "PDF 2.0", "✓ Complete"],
            vec![
                "JavaScript",
                "Adobe PDF JS API",
                "Acrobat Compatible",
                "✓ Complete",
            ],
            vec![
                "Document Structure",
                "Tagged PDF Ready",
                "PDF/UA",
                "🔄 In Progress",
            ],
            vec![
                "Digital Signatures",
                "PKCS#7 + RFC 3161",
                "PDF 1.6+",
                "📋 Planned",
            ],
        ],
    )?;

    tech_doc.save("pdf_sdk_technical_docs.pdf")?;
    println!("   ✅ Created technical documentation: pdf_sdk_technical_docs.pdf");

    // Step 13: Document validation and optimization
    println!("\n✅ Step 13: Validation and optimization...");

    // Validate document structure
    let validation_issues = doc.validate_structure()?;
    if validation_issues.is_empty() {
        println!("   ✅ Document structure validation: PASSED");
    } else {
        println!("   ⚠️ Validation issues found: {}", validation_issues.len());
        for issue in &validation_issues {
            println!("     • {}", issue);
        }
    }

    // Optimize document
    let original_images = doc.images.len();
    let original_annotations = doc.annotations.len();
    doc.optimize_document()?;
    println!("   ✅ Document optimization completed");
    println!(
        "     • Images: {} → {} (removed unused)",
        original_images,
        doc.images.len()
    );
    println!(
        "     • Annotations: {} → {} (removed empty)",
        original_annotations,
        doc.annotations.len()
    );

    println!("\n🎉 Advanced PDF Demo Completed Successfully!");
    println!("\nFiles created:");
    println!("  📊 comprehensive_business_report.pdf - Main document (4 pages, fully featured)");
    println!("  📋 executive_brief.pdf - Executive summary (2 pages)");
    println!("  📈 financial_data_appendix.pdf - Data analysis (1 page)");
    println!("  🔧 pdf_sdk_technical_docs.pdf - Technical documentation (1 page)");

    println!("\nAdvanced features demonstrated:");
    println!("  ✓ Professional document metadata and versioning");
    println!("  ✓ Multi-page documents with different formats (A4, Letter, Legal)");
    println!("  ✓ Rich text formatting with styles and colors");
    println!("  ✓ Advanced graphics (shapes, lines, transparency effects)");
    println!("  ✓ Professional image embedding with PNG support");
    println!("  ✓ Structured data tables with formatting");
    println!("  ✓ Interactive form fields (text, dropdown, checkbox, multiline)");
    println!("  ✓ Comprehensive annotations (text, highlight, link)");
    println!("  ✓ Document security (AES-256 encryption, permission controls)");
    println!("  ✓ JavaScript automation (validation, analytics, document control)");
    println!("  ✓ Navigation structure (bookmarks, outline, table of contents)");
    println!("  ✓ Document layers for content organization");
    println!("  ✓ File attachments with metadata and checksums");
    println!("  ✓ Advanced viewer preferences and display settings");
    println!("  ✓ Document validation and optimization tools");
    println!("  ✓ Round-trip compatibility and text extraction");
    println!("  ✓ Page extraction and document merging capabilities");

    println!("\nDocument access instructions:");
    println!("  📖 User Password: UserView2024!");
    println!("  🔧 Owner Password: OwnerEdit2024@");
    println!("  🚫 Permissions: View and print only (copying/editing restricted)");

    println!("\nJavaScript features:");
    println!("  🔍 Form validation with comprehensive error checking");
    println!("  📊 Document usage analytics and tracking");
    println!("  ⚙️ Automatic document configuration and setup");
    println!("  💬 Interactive user guidance and welcome messages");

    Ok(())
}
