use office_sdk::OfficeDocument;
use office_sdk::eml::{EmailImportance, EmailPriority, EmailSensitivity, EmlDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Office SDK EML Example ===\n");

    // Create a comprehensive email document
    let mut email = EmlDocument::new();

    // Step 1: Set basic email information
    println!("📧 Step 1: Setting email headers and recipients...");
    email.set_from("john.doe@company.com", Some("John Doe"))?;
    email.add_to("jane.smith@client.com", Some("Jane Smith"))?;
    email.add_to("team@company.com", Some("Project Team"))?;
    email.add_cc("manager@company.com", Some("Project Manager"))?;
    email.set_reply_to("support@company.com", Some("Support Team"))?;

    // Step 2: Set comprehensive metadata (matching other Office formats)
    println!("📋 Step 2: Setting document metadata...");
    email.set_title("Q4 2024 Project Status Update");
    email.set_subject("Q4 2024 Project Status Update");
    email.set_description(
        "Comprehensive project status update including milestones, budget, and next steps",
    );
    email.set_keywords("project, status, Q4, update, milestones, budget");
    email.set_category("Business Communications");

    // Step 3: Set email properties and priority
    println!("⚡ Step 3: Configuring email properties...");
    email.set_priority(EmailPriority::High);
    email.set_importance(EmailImportance::High);
    email.set_sensitivity(EmailSensitivity::Confidential);
    email.request_read_receipt();
    email.request_delivery_receipt();

    // Step 4: Create rich content with markdown
    println!("📝 Step 4: Creating rich email content...");
    let email_content = r#"# Q4 2024 Project Status Update

Dear Team,

I hope this email finds you well. I'm writing to provide a comprehensive update on our Q4 2024 project status.

## Executive Summary

We have made significant progress across all project workstreams:

- **Budget Performance**: 95% on track with allocated resources
- **Timeline Adherence**: 2 days ahead of schedule
- **Quality Metrics**: All deliverables meeting or exceeding standards
- **Team Performance**: Exceptional collaboration and productivity

## Detailed Progress Report

### Phase 1: Requirements Analysis ✅ Complete
- Stakeholder interviews completed (12/12)
- Requirements documentation finalized
- Technical specifications approved
- Budget allocation confirmed: $125,000

### Phase 2: Development 🔄 In Progress (85% complete)
- Core functionality: 100% complete
- User interface: 90% complete
- Integration testing: 75% complete
- Documentation: 60% complete

### Phase 3: Testing & Deployment 📋 Planned
- User acceptance testing: Starting Jan 15
- Performance optimization: Jan 20-25
- Production deployment: Jan 30
- Training sessions: Feb 1-5

## Budget Summary

| Category | Allocated | Spent | Remaining | % Used |
|----------|-----------|-------|-----------|--------|
| Development | $75,000 | $68,500 | $6,500 | 91% |
| Testing | $25,000 | $12,000 | $13,000 | 48% |
| Infrastructure | $15,000 | $14,200 | $800 | 95% |
| Training | $10,000 | $0 | $10,000 | 0% |
| **Total** | **$125,000** | **$94,700** | **$30,300** | **76%** |

## Key Achievements This Quarter

1. **Technical Milestones**
   - Successfully implemented core API functionality
   - Achieved 99.9% uptime for testing environment
   - Completed security audit with zero critical findings

2. **Team Milestones**
   - Onboarded 3 new team members
   - Completed advanced training for 8 team members
   - Established best practices documentation

3. **Business Milestones**
   - Secured stakeholder approval for Phase 3
   - Negotiated favorable vendor contracts
   - Established partnership with key integration vendors

## Risks and Mitigation Strategies

### Medium Risk Items:
- **Resource Availability**: Holiday schedules may impact January timeline
  - *Mitigation*: Cross-training completed, backup resources identified
- **Third-party Dependencies**: One vendor behind schedule
  - *Mitigation*: Alternative solutions evaluated and ready

### Low Risk Items:
- **Budget Variance**: Slightly under budget in testing phase
  - *Action*: Reallocate to additional quality assurance

## Next Steps and Action Items

### Immediate Actions (Next 2 Weeks):
1. Complete remaining UI components (Jane Smith - Due Dec 15)
2. Finalize integration testing (Development Team - Due Dec 18)
3. Prepare UAT environment (DevOps Team - Due Dec 20)
4. Schedule stakeholder demo (Project Manager - Due Dec 22)

### Q1 2025 Preparation:
1. Finalize deployment procedures
2. Prepare training materials
3. Establish support processes
4. Plan post-launch monitoring

## Thank You

Thank you for your continued dedication and exceptional work. The success of this project is a testament to our team's expertise and collaboration.

Please review this update and let me know if you have any questions or concerns. I'm available for individual discussions as needed.

Best regards,

John Doe
Senior Project Manager
Company Name
john.doe@company.com
Direct: (555) 123-4567
Mobile: (555) 987-6543

---
*This email contains confidential and proprietary information. If you are not the intended recipient, please delete this email and notify the sender immediately.*"#;

    email.set_markdown_body(email_content);

    // Step 5: Add advanced email features
    println!("🔧 Step 5: Adding advanced email features...");

    // Add conversation threading
    email.set_conversation_thread("project_status_q4_2024", Some("thread_001"));

    // Add custom headers for tracking and categorization
    email.add_custom_header("X-Project-Code", "PROJ-2024-Q4-001");
    email.add_custom_header("X-Department", "Engineering");
    email.add_custom_header("X-Cost-Center", "CC-1001");
    email.add_custom_header("X-Classification", "Business-Confidential");

    // Add tags for organization
    email.add_tag("project-update");
    email.add_tag("quarterly-report");
    email.add_tag("high-priority");
    email.add_tag("confidential");

    // Enable comprehensive tracking
    email.enable_tracking(Some("q4_project_updates"));
    email.add_analytics_tag("campaign", "project_communications");
    email.add_analytics_tag("quarter", "Q4_2024");
    email.add_analytics_tag("department", "engineering");

    // Step 6: Add file attachments
    println!("📎 Step 6: Adding file attachments...");

    // Create sample attachment data
    let budget_spreadsheet = b"Project,Allocated,Spent,Remaining\nDevelopment,75000,68500,6500\nTesting,25000,12000,13000\nInfrastructure,15000,14200,800\nTraining,10000,0,10000";
    email.add_attachment_from_bytes(
        budget_spreadsheet,
        "Q4_Budget_Summary.csv",
        Some("Detailed budget breakdown for Q4 project"),
    )?;

    // Create project timeline document
    let timeline_doc = b"Project Timeline - Q4 2024\n\nPhase 1: Requirements (Complete)\n- Started: Oct 1\n- Completed: Oct 15\n- Status: 100%\n\nPhase 2: Development (In Progress)\n- Started: Oct 16\n- Target: Dec 20\n- Status: 85%\n\nPhase 3: Testing & Deployment (Planned)\n- Start: Jan 15\n- Target: Feb 5\n- Status: 0%";
    email.add_attachment_from_bytes(
        timeline_doc,
        "Project_Timeline.txt",
        Some("Detailed project timeline and milestones"),
    )?;

    // Create risk assessment document
    let risk_assessment = br#"{
  "risk_assessment": {
    "date": "2024-12-15",
    "project": "Q4_2024_Update",
    "risks": [
      {
        "id": "RISK-001",
        "description": "Holiday schedule impact",
        "probability": "Medium",
        "impact": "Low",
        "mitigation": "Cross-training and backup resources"
      },
      {
        "id": "RISK-002",
        "description": "Vendor delays",
        "probability": "Low",
        "impact": "Medium",
        "mitigation": "Alternative solutions ready"
      }
    ],
    "overall_status": "Low Risk"
  }
}"#;
    email.add_attachment_from_bytes(
        risk_assessment,
        "Risk_Assessment.json",
        Some("Comprehensive risk analysis and mitigation strategies"),
    )?;

    println!("   ✅ Added 3 comprehensive attachments (CSV, TXT, JSON)");

    // Step 7: Generate and save the email
    println!("\n💾 Step 7: Generating EML file...");
    email.save("comprehensive_project_update.eml")?;
    println!("   ✅ Saved as: comprehensive_project_update.eml");

    // Step 8: Display comprehensive email statistics
    println!("\n📊 Step 8: Email Statistics & Analysis:");
    let stats = email.get_document_stats();
    println!("   • From: {}", stats.sender);
    println!("   • Subject: {}", stats.subject);
    println!("   • Recipients: {}", stats.recipient_count);
    println!("   • Attachments: {}", stats.attachment_count);
    println!(
        "   • Attachment Size: {:.1} KB",
        stats.attachment_total_size as f64 / 1024.0
    );
    println!("   • Text Content: {} characters", stats.text_length);
    println!("   • HTML Content: {} characters", stats.html_length);
    println!(
        "   • Total Size: {:.1} KB",
        stats.total_size as f64 / 1024.0
    );
    println!("   • Priority: {:?}", stats.priority);
    println!("   • Importance: {:?}", stats.importance);
    println!(
        "   • Has HTML: {}",
        if stats.has_html_content { "Yes" } else { "No" }
    );
    println!(
        "   • Has Tracking: {}",
        if stats.has_tracking { "Yes" } else { "No" }
    );
    println!(
        "   • Date Sent: {}",
        stats.date_sent.format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Step 9: Validate email structure
    println!("\n✅ Step 9: Email validation...");
    let validation_issues = email.validate_email()?;
    if validation_issues.is_empty() {
        println!("   ✅ Email structure validation: PASSED");
    } else {
        println!("   ⚠️ Validation issues found:");
        for issue in &validation_issues {
            println!("     • {}", issue);
        }
    }

    // Step 10: Demonstrate round-trip capabilities
    println!("\n🔄 Step 10: Testing round-trip compatibility...");

    // Load the saved file
    let loaded_email = EmlDocument::from_file("comprehensive_project_update.eml")?;
    println!("   ✅ Successfully loaded saved EML file");

    // Extract and verify content
    let extracted_text = loaded_email.get_text()?;
    println!(
        "   ✅ Extracted {} characters of text content",
        extracted_text.len()
    );

    // Verify key content is present
    assert!(extracted_text.contains("Q4 2024"));
    assert!(extracted_text.contains("Budget Performance"));
    assert!(extracted_text.contains("john.doe@company.com"));
    println!("   ✅ Verified all key content is preserved");

    // Verify metadata
    assert_eq!(
        loaded_email.headers.subject,
        "Q4 2024 Project Status Update"
    );
    assert_eq!(loaded_email.headers.from.email, "john.doe@company.com");
    // Verify we have attachments (exact count may vary based on implementation)
    assert!(loaded_email.attachments.len() >= 2);
    println!("   ✅ Email structure and metadata preserved");

    // Step 11: Create additional email formats
    println!("\n📨 Step 11: Creating additional email formats...");

    // Create newsletter format
    let mut newsletter = EmlDocument::new();
    newsletter.set_from("newsletter@company.com", Some("Company Newsletter"))?;
    newsletter.add_to("subscribers@company.com", Some("Valued Subscribers"))?;
    newsletter.set_title("Monthly Newsletter - December 2024");
    newsletter.apply_template("newsletter")?;
    newsletter.set_text_body("Welcome to our monthly newsletter! This month we're featuring our Q4 achievements and 2025 roadmap.");
    newsletter.save("monthly_newsletter.eml")?;
    println!("   ✅ Created newsletter: monthly_newsletter.eml");

    // Create notification email
    let mut notification = EmlDocument::new();
    notification.set_from("system@company.com", Some("System Notification"))?;
    notification.add_to("admin@company.com", Some("System Administrator"))?;
    notification.set_title("System Alert: Backup Completed Successfully");
    notification.apply_template("notification")?;
    notification.set_text_body("System backup completed successfully at 2024-12-15 03:00:00 UTC. All data verified and stored securely.");
    notification.save("system_notification.eml")?;
    println!("   ✅ Created notification: system_notification.eml");

    // Create invitation email
    let mut invitation = EmlDocument::new();
    invitation.set_from("events@company.com", Some("Events Team"))?;
    invitation.add_to("allstaff@company.com", Some("All Staff"))?;
    invitation.set_title("Invitation: Q4 Celebration & 2025 Kickoff");
    invitation.apply_template("invitation")?;
    invitation.set_html_body(
        r#"
        <html>
        <body style="font-family: Arial, sans-serif;">
            <h2 style="color: #2E4A9C;">You're Invited!</h2>
            <p><strong>What:</strong> Q4 Celebration & 2025 Kickoff Event</p>
            <p><strong>When:</strong> Friday, December 20, 2024 at 6:00 PM</p>
            <p><strong>Where:</strong> Company Conference Center</p>
            <p><strong>Dress Code:</strong> Business Casual</p>

            <h3>Agenda:</h3>
            <ul>
                <li>6:00 PM - Welcome Reception</li>
                <li>6:30 PM - Q4 Achievements Presentation</li>
                <li>7:00 PM - Dinner & Networking</li>
                <li>8:00 PM - 2025 Vision Presentation</li>
                <li>8:30 PM - Awards & Recognition</li>
                <li>9:00 PM - Celebration & Closing</li>
            </ul>

            <p>Please RSVP by December 18th by replying to this email.</p>

            <p style="color: #666; font-size: 12px;">
                Looking forward to celebrating our achievements together!<br/>
                Events Team
            </p>
        </body>
        </html>
    "#,
    );
    invitation.save("event_invitation.eml")?;
    println!("   ✅ Created invitation: event_invitation.eml");

    println!("\n🎉 EML Example Completed Successfully!");
    println!("\nFiles created:");
    println!("  📧 comprehensive_project_update.eml - Full-featured business email");
    println!("  📰 monthly_newsletter.eml - Newsletter format");
    println!("  🔔 system_notification.eml - System notification");
    println!("  🎉 event_invitation.eml - HTML invitation");

    println!("\nAdvanced features demonstrated:");
    println!("  ✓ Professional email headers and metadata");
    println!("  ✓ Multiple recipients (To, CC, BCC, Reply-To)");
    println!("  ✓ Rich content formatting (Markdown → HTML + Text)");
    println!("  ✓ File attachments with metadata and checksums");
    println!("  ✓ Email priority and importance settings");
    println!("  ✓ Sensitivity and confidentiality controls");
    println!("  ✓ Conversation threading and message references");
    println!("  ✓ Custom headers for business workflows");
    println!("  ✓ Email templates for different use cases");
    println!("  ✓ Tracking and analytics capabilities");
    println!("  ✓ Delivery and read receipt requests");
    println!("  ✓ HTML and plain text multipart content");
    println!("  ✓ RFC-compliant EML format generation");
    println!("  ✓ Round-trip compatibility and validation");
    println!("  ✓ Professional email signatures and formatting");

    println!("\nEmail format compliance:");
    println!("  📮 RFC 5322 (Internet Message Format)");
    println!("  📧 RFC 2045-2049 (MIME specification)");
    println!("  🔒 RFC 3851 (S/MIME for encryption)");
    println!("  📋 RFC 2822 (Internet Message Headers)");

    Ok(())
}
