use crate::OfficeDocument;
use crate::error::{OfficeError, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use mail_parser::{MessageParser, MimeHeaders};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Comprehensive EML (Email) document with advanced features matching other Office formats
#[derive(Debug, Clone)]
pub struct EmlDocument {
    // Core email structure
    pub headers: EmailHeaders,
    pub body: EmailBody,
    pub attachments: HashMap<String, AttachmentInfo>,

    // Enhanced metadata (matching DOCX/XLSX/PPTX/PDF)
    pub author: String,           // From sender
    pub title: String,            // Subject
    pub subject: String,          // Email subject
    pub keywords: String,         // Keywords from headers or content
    pub description: String,      // Email body preview
    pub category: String,         // Email category/folder
    pub created: DateTime<Utc>,   // Date sent
    pub modified: DateTime<Utc>,  // Last modified
    pub last_modified_by: String, // Last modifier
    pub revision: u32,            // Revision number

    // Email-specific properties
    pub message_id: String,
    pub conversation_id: Option<String>,
    pub thread_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub priority: EmailPriority,
    pub sensitivity: EmailSensitivity,
    pub importance: EmailImportance,
    pub delivery_receipt: bool,
    pub read_receipt: bool,

    // Advanced email features
    pub encryption: Option<EncryptionInfo>,
    pub digital_signature: Option<SignatureInfo>,
    pub tags: Vec<String>,
    pub flags: EmailFlags,
    pub custom_headers: HashMap<String, String>,

    // Content organization
    pub alternative_views: Vec<AlternativeView>,
    pub embedded_objects: HashMap<String, EmbeddedObject>,
    pub related_messages: Vec<String>,

    // Tracking and analytics
    pub tracking_info: Option<TrackingInfo>,
    pub delivery_status: DeliveryStatus,
    pub read_status: ReadStatus,
}

#[derive(Debug, Clone)]
pub struct EmailHeaders {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub reply_to: Vec<EmailAddress>,
    pub sender: Option<EmailAddress>,
    pub return_path: Option<String>,
    pub date: DateTime<Utc>,
    pub subject: String,
    pub message_id: String,
    pub mime_version: String,
    pub content_type: ContentType,
    pub content_transfer_encoding: String,
    pub x_mailer: Option<String>,
    pub x_priority: Option<String>,
    pub list_unsubscribe: Option<String>,
    pub auto_submitted: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmailBody {
    pub text_content: Option<String>,
    pub html_content: Option<String>,
    pub rtf_content: Option<String>,
    pub markdown_content: Option<String>,
    pub content_type: ContentType,
    pub charset: String,
    pub content_transfer_encoding: String,
    pub content_disposition: Option<String>,
    pub content_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachmentInfo {
    pub id: String,
    pub filename: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub content_disposition: String,
    pub content_transfer_encoding: String,
    pub content_id: Option<String>,
    pub size: usize,
    pub checksum: Option<String>,
    pub is_inline: bool,
    pub description: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ContentType {
    pub media_type: String,
    pub media_subtype: String,
    pub parameters: HashMap<String, String>,
    pub boundary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmailPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmailSensitivity {
    Normal,
    Personal,
    Private,
    Confidential,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmailImportance {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub key_id: Option<String>,
    pub is_signed: bool,
    pub is_encrypted: bool,
    pub certificate_thumbprint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub algorithm: String,
    pub certificate: Option<Vec<u8>>,
    pub is_valid: bool,
    pub signer_email: String,
    pub signing_time: DateTime<Utc>,
    pub certificate_chain: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct EmailFlags {
    pub seen: bool,
    pub answered: bool,
    pub flagged: bool,
    pub deleted: bool,
    pub draft: bool,
    pub recent: bool,
    pub junk: bool,
    pub forwarded: bool,
}

#[derive(Debug, Clone)]
pub struct AlternativeView {
    pub content_type: ContentType,
    pub content: String,
    pub charset: String,
    pub transfer_encoding: String,
}

#[derive(Debug, Clone)]
pub struct EmbeddedObject {
    pub id: String,
    pub content_id: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub filename: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrackingInfo {
    pub tracking_pixel: Option<String>,
    pub open_tracking: bool,
    pub click_tracking: bool,
    pub bounce_tracking: bool,
    pub delivery_tracking: bool,
    pub campaign_id: Option<String>,
    pub analytics_tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Bounced,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadStatus {
    Unread,
    Read,
    Replied,
    Forwarded,
}

#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub total_size: usize,
    pub attachment_count: usize,
    pub attachment_total_size: usize,
    pub recipient_count: usize,
    pub text_length: usize,
    pub html_length: usize,
    pub has_attachments: bool,
    pub has_html_content: bool,
    pub has_encryption: bool,
    pub has_digital_signature: bool,
    pub has_tracking: bool,
    pub sender: String,
    pub subject: String,
    pub date_sent: DateTime<Utc>,
    pub priority: EmailPriority,
    pub importance: EmailImportance,
}

// Default implementations
impl Default for EmlDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EmailHeaders {
    fn default() -> Self {
        EmailHeaders {
            from: EmailAddress::new("sender@example.com", None),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: Vec::new(),
            sender: None,
            return_path: None,
            date: Utc::now(),
            subject: "New Message".to_string(),
            message_id: format!("<{}@example.com>", Uuid::new_v4()),
            mime_version: "1.0".to_string(),
            content_type: ContentType::default(),
            content_transfer_encoding: "7bit".to_string(),
            x_mailer: Some("Office SDK".to_string()),
            x_priority: None,
            list_unsubscribe: None,
            auto_submitted: None,
        }
    }
}

impl Default for EmailBody {
    fn default() -> Self {
        EmailBody {
            text_content: None,
            html_content: None,
            rtf_content: None,
            markdown_content: None,
            content_type: ContentType::text_plain(),
            charset: "UTF-8".to_string(),
            content_transfer_encoding: "7bit".to_string(),
            content_disposition: None,
            content_id: None,
        }
    }
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::text_plain()
    }
}

impl Default for EmailFlags {
    fn default() -> Self {
        EmailFlags {
            seen: false,
            answered: false,
            flagged: false,
            deleted: false,
            draft: false,
            recent: true,
            junk: false,
            forwarded: false,
        }
    }
}

impl ContentType {
    pub fn new(media_type: &str, media_subtype: &str) -> Self {
        ContentType {
            media_type: media_type.to_string(),
            media_subtype: media_subtype.to_string(),
            parameters: HashMap::new(),
            boundary: None,
        }
    }

    pub fn text_plain() -> Self {
        ContentType::new("text", "plain")
    }

    pub fn text_html() -> Self {
        ContentType::new("text", "html")
    }

    pub fn multipart_mixed() -> Self {
        let mut ct = ContentType::new("multipart", "mixed");
        ct.boundary = Some(format!("boundary_{}", Uuid::new_v4().simple()));
        ct
    }

    pub fn multipart_alternative() -> Self {
        let mut ct = ContentType::new("multipart", "alternative");
        ct.boundary = Some(format!("boundary_{}", Uuid::new_v4().simple()));
        ct
    }

    pub fn multipart_related() -> Self {
        let mut ct = ContentType::new("multipart", "related");
        ct.boundary = Some(format!("boundary_{}", Uuid::new_v4().simple()));
        ct
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{}/{}", self.media_type, self.media_subtype);

        for (key, value) in &self.parameters {
            result.push_str(&format!("; {}={}", key, value));
        }

        if let Some(ref boundary) = self.boundary {
            result.push_str(&format!("; boundary={}", boundary));
        }

        result
    }
}

impl EmailAddress {
    pub fn new(email: &str, name: Option<&str>) -> Self {
        EmailAddress {
            email: email.to_string(),
            name: name.map(|s| s.to_string()),
            display_name: name.map(|s| s.to_string()),
        }
    }

    pub fn with_display_name(email: &str, display_name: &str) -> Self {
        EmailAddress {
            email: email.to_string(),
            name: Some(display_name.to_string()),
            display_name: Some(display_name.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        if let Some(ref name) = self.display_name {
            format!("{} <{}>", name, self.email)
        } else {
            self.email.clone()
        }
    }

    pub fn parse(address_str: &str) -> Result<Self> {
        if address_str.contains('<') && address_str.contains('>') {
            // Format: "Display Name <email@domain.com>"
            let parts: Vec<&str> = address_str.splitn(2, '<').collect();
            if parts.len() == 2 {
                let display_name = parts[0].trim().trim_matches('"');
                let email = parts[1].trim_end_matches('>').trim();
                Ok(EmailAddress::with_display_name(email, display_name))
            } else {
                Ok(EmailAddress::new(address_str.trim(), None))
            }
        } else {
            // Plain email address
            Ok(EmailAddress::new(address_str.trim(), None))
        }
    }
}

impl EmlDocument {
    pub fn new() -> Self {
        let now = Utc::now();
        EmlDocument {
            headers: EmailHeaders::default(),
            body: EmailBody::default(),
            attachments: HashMap::new(),

            // Enhanced metadata matching other Office formats
            author: "Office SDK".to_string(),
            title: "New Email".to_string(),
            subject: "New Message".to_string(),
            keywords: "".to_string(),
            description: "".to_string(),
            category: "".to_string(),
            created: now,
            modified: now,
            last_modified_by: "Office SDK".to_string(),
            revision: 1,

            // Email-specific properties
            message_id: format!("<{}@office-sdk.local>", Uuid::new_v4()),
            conversation_id: None,
            thread_id: None,
            in_reply_to: None,
            references: Vec::new(),
            priority: EmailPriority::Normal,
            sensitivity: EmailSensitivity::Normal,
            importance: EmailImportance::Normal,
            delivery_receipt: false,
            read_receipt: false,

            // Advanced features
            encryption: None,
            digital_signature: None,
            tags: Vec::new(),
            flags: EmailFlags::default(),
            custom_headers: HashMap::new(),

            // Content organization
            alternative_views: Vec::new(),
            embedded_objects: HashMap::new(),
            related_messages: Vec::new(),

            // Tracking and analytics
            tracking_info: None,
            delivery_status: DeliveryStatus::Pending,
            read_status: ReadStatus::Unread,
        }
    }

    // Enhanced metadata methods (matching other Office formats)
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
        self.headers.from =
            EmailAddress::parse(author).unwrap_or_else(|_| EmailAddress::new(author, None));
        self.modified = Utc::now();
        self.revision += 1;
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.subject = title.to_string();
        self.headers.subject = title.to_string();
        self.modified = Utc::now();
        self.revision += 1;
    }

    pub fn set_subject(&mut self, subject: &str) {
        self.subject = subject.to_string();
        self.title = subject.to_string();
        self.headers.subject = subject.to_string();
        self.modified = Utc::now();
    }

    pub fn set_keywords(&mut self, keywords: &str) {
        self.keywords = keywords.to_string();
        self.custom_headers
            .insert("X-Keywords".to_string(), keywords.to_string());
        self.modified = Utc::now();
    }

    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
        self.modified = Utc::now();
    }

    pub fn set_category(&mut self, category: &str) {
        self.category = category.to_string();
        self.custom_headers
            .insert("X-Category".to_string(), category.to_string());
        self.modified = Utc::now();
    }

    // Email-specific methods
    pub fn set_from(&mut self, email: &str, name: Option<&str>) -> Result<()> {
        self.headers.from = EmailAddress::new(email, name);
        self.author = if let Some(name) = name {
            format!("{} <{}>", name, email)
        } else {
            email.to_string()
        };
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_to(&mut self, email: &str, name: Option<&str>) -> Result<()> {
        self.headers.to.push(EmailAddress::new(email, name));
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_cc(&mut self, email: &str, name: Option<&str>) -> Result<()> {
        self.headers.cc.push(EmailAddress::new(email, name));
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_bcc(&mut self, email: &str, name: Option<&str>) -> Result<()> {
        self.headers.bcc.push(EmailAddress::new(email, name));
        self.modified = Utc::now();
        Ok(())
    }

    pub fn set_reply_to(&mut self, email: &str, name: Option<&str>) -> Result<()> {
        self.headers.reply_to = vec![EmailAddress::new(email, name)];
        self.modified = Utc::now();
        Ok(())
    }

    // Content methods
    pub fn set_text_body(&mut self, text: &str) {
        self.body.text_content = Some(text.to_string());
        self.body.content_type = ContentType::text_plain();
        self.description =
            text.chars().take(100).collect::<String>() + if text.len() > 100 { "..." } else { "" };
        self.modified = Utc::now();
    }

    pub fn set_html_body(&mut self, html: &str) {
        self.body.html_content = Some(html.to_string());
        self.body.content_type = ContentType::text_html();

        // Extract text for description (simple HTML stripping)
        let text_content = html
            .replace("<br>", "\n")
            .replace("<p>", "\n")
            .replace("</p>", "\n")
            .replace("<div>", "\n")
            .replace("</div>", "\n");

        // Simple tag removal
        let mut in_tag = false;
        let text_only: String = text_content
            .chars()
            .filter(|&c| {
                if c == '<' {
                    in_tag = true;
                    false
                } else if c == '>' {
                    in_tag = false;
                    false
                } else {
                    !in_tag
                }
            })
            .collect();

        self.description = text_only.chars().take(100).collect::<String>()
            + if text_only.len() > 100 { "..." } else { "" };
        self.modified = Utc::now();
    }

    pub fn set_multipart_body(&mut self, text: &str, html: &str) {
        self.body.text_content = Some(text.to_string());
        self.body.html_content = Some(html.to_string());
        self.body.content_type = ContentType::multipart_alternative();
        self.description =
            text.chars().take(100).collect::<String>() + if text.len() > 100 { "..." } else { "" };
        self.modified = Utc::now();
    }

    pub fn set_markdown_body(&mut self, markdown: &str) {
        self.body.markdown_content = Some(markdown.to_string());

        // Convert markdown to HTML for HTML view
        let html = self.markdown_to_html(markdown);
        self.body.html_content = Some(html);

        // Convert markdown to plain text
        let text = self.markdown_to_text(markdown);
        self.body.text_content = Some(text.clone());

        self.body.content_type = ContentType::multipart_alternative();
        self.description =
            text.chars().take(100).collect::<String>() + if text.len() > 100 { "..." } else { "" };
        self.modified = Utc::now();
    }

    // Attachment methods (matching other Office formats)
    pub fn add_attachment<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        filename: Option<&str>,
    ) -> Result<String> {
        let data = std::fs::read(&file_path)?;
        let file_name = filename.unwrap_or_else(|| {
            file_path
                .as_ref()
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("attachment")
        });

        self.add_attachment_from_bytes(&data, file_name, None)
    }

    pub fn add_attachment_from_bytes(
        &mut self,
        data: &[u8],
        filename: &str,
        description: Option<&str>,
    ) -> Result<String> {
        let attachment_id = format!("attachment{}", self.attachments.len() + 1);

        // Determine content type from filename
        let content_type = self.determine_content_type(filename);

        // Calculate checksum
        let checksum = format!("{:x}", md5::compute(data));

        let attachment = AttachmentInfo {
            id: attachment_id.clone(),
            filename: filename.to_string(),
            data: data.to_vec(),
            content_type,
            content_disposition: format!("attachment; filename=\"{}\"", filename),
            content_transfer_encoding: "base64".to_string(),
            content_id: None,
            size: data.len(),
            checksum: Some(checksum),
            is_inline: false,
            description: description.map(|s| s.to_string()),
            creation_date: Some(Utc::now()),
            modification_date: Some(Utc::now()),
        };

        self.attachments.insert(attachment_id.clone(), attachment);
        self.modified = Utc::now();
        Ok(attachment_id)
    }

    pub fn add_inline_attachment(
        &mut self,
        data: &[u8],
        filename: &str,
        content_id: &str,
    ) -> Result<String> {
        let attachment_id = format!("inline{}", self.embedded_objects.len() + 1);

        let embedded_object = EmbeddedObject {
            id: attachment_id.clone(),
            content_id: content_id.to_string(),
            data: data.to_vec(),
            content_type: self.determine_content_type(filename),
            filename: Some(filename.to_string()),
        };

        self.embedded_objects
            .insert(attachment_id.clone(), embedded_object);
        self.modified = Utc::now();
        Ok(attachment_id)
    }

    // Email configuration methods
    pub fn set_priority(&mut self, priority: EmailPriority) {
        self.priority = priority;
        self.headers.x_priority = Some(match priority {
            EmailPriority::Low => "5".to_string(),
            EmailPriority::Normal => "3".to_string(),
            EmailPriority::High => "2".to_string(),
            EmailPriority::Urgent => "1".to_string(),
        });
        self.modified = Utc::now();
    }

    pub fn set_importance(&mut self, importance: EmailImportance) {
        self.importance = importance;
        self.custom_headers.insert(
            "Importance".to_string(),
            match importance {
                EmailImportance::Low => "Low".to_string(),
                EmailImportance::Normal => "Normal".to_string(),
                EmailImportance::High => "High".to_string(),
            },
        );
        self.modified = Utc::now();
    }

    pub fn set_sensitivity(&mut self, sensitivity: EmailSensitivity) {
        self.sensitivity = sensitivity;
        self.custom_headers.insert(
            "Sensitivity".to_string(),
            match sensitivity {
                EmailSensitivity::Normal => "Normal".to_string(),
                EmailSensitivity::Personal => "Personal".to_string(),
                EmailSensitivity::Private => "Private".to_string(),
                EmailSensitivity::Confidential => "Confidential".to_string(),
            },
        );
        self.modified = Utc::now();
    }

    pub fn request_delivery_receipt(&mut self) {
        self.delivery_receipt = true;
        self.custom_headers.insert(
            "Return-Receipt-To".to_string(),
            self.headers.from.email.clone(),
        );
        self.modified = Utc::now();
    }

    pub fn request_read_receipt(&mut self) {
        self.read_receipt = true;
        self.custom_headers.insert(
            "Disposition-Notification-To".to_string(),
            self.headers.from.email.clone(),
        );
        self.modified = Utc::now();
    }

    // Advanced features
    pub fn add_custom_header(&mut self, name: &str, value: &str) {
        self.custom_headers
            .insert(name.to_string(), value.to_string());
        self.modified = Utc::now();
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
            self.custom_headers
                .insert("X-Tags".to_string(), self.tags.join(", "));
            self.modified = Utc::now();
        }
    }

    pub fn set_conversation_thread(&mut self, conversation_id: &str, thread_id: Option<&str>) {
        self.conversation_id = Some(conversation_id.to_string());
        self.thread_id = thread_id.map(|s| s.to_string());
        self.custom_headers
            .insert("X-Conversation-ID".to_string(), conversation_id.to_string());
        if let Some(tid) = thread_id {
            self.custom_headers
                .insert("X-Thread-ID".to_string(), tid.to_string());
        }
        self.modified = Utc::now();
    }

    pub fn set_in_reply_to(&mut self, message_id: &str) {
        self.in_reply_to = Some(message_id.to_string());
        self.custom_headers
            .insert("In-Reply-To".to_string(), message_id.to_string());
        self.modified = Utc::now();
    }

    pub fn add_reference(&mut self, message_id: &str) {
        if !self.references.contains(&message_id.to_string()) {
            self.references.push(message_id.to_string());
            self.custom_headers
                .insert("References".to_string(), self.references.join(" "));
            self.modified = Utc::now();
        }
    }

    // Tracking and analytics
    pub fn enable_tracking(&mut self, campaign_id: Option<&str>) {
        let tracking = TrackingInfo {
            tracking_pixel: Some(format!(
                "https://track.example.com/pixel/{}",
                Uuid::new_v4()
            )),
            open_tracking: true,
            click_tracking: true,
            bounce_tracking: true,
            delivery_tracking: true,
            campaign_id: campaign_id.map(|s| s.to_string()),
            analytics_tags: HashMap::new(),
        };

        self.tracking_info = Some(tracking);
        self.modified = Utc::now();
    }

    pub fn add_analytics_tag(&mut self, key: &str, value: &str) {
        if let Some(ref mut tracking) = self.tracking_info {
            tracking
                .analytics_tags
                .insert(key.to_string(), value.to_string());
        } else {
            self.enable_tracking(None);
            if let Some(ref mut tracking) = self.tracking_info {
                tracking
                    .analytics_tags
                    .insert(key.to_string(), value.to_string());
            }
        }
        self.modified = Utc::now();
    }

    // Analysis and statistics methods
    pub fn get_document_stats(&self) -> DocumentStats {
        let text_length = self
            .body
            .text_content
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0);
        let html_length = self
            .body
            .html_content
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0);
        let attachment_total_size = self.attachments.values().map(|a| a.size).sum();

        DocumentStats {
            total_size: text_length + html_length + attachment_total_size,
            attachment_count: self.attachments.len(),
            attachment_total_size,
            recipient_count: self.headers.to.len() + self.headers.cc.len() + self.headers.bcc.len(),
            text_length,
            html_length,
            has_attachments: !self.attachments.is_empty(),
            has_html_content: self.body.html_content.is_some(),
            has_encryption: self.encryption.is_some(),
            has_digital_signature: self.digital_signature.is_some(),
            has_tracking: self.tracking_info.is_some(),
            sender: self.headers.from.to_string(),
            subject: self.headers.subject.clone(),
            date_sent: self.headers.date,
            priority: self.priority,
            importance: self.importance,
        }
    }

    pub fn get_text(&self) -> Result<String> {
        let mut text = String::new();

        // Add headers summary
        text.push_str(&format!("From: {}\n", self.headers.from.to_string()));
        text.push_str(&format!("Subject: {}\n", self.headers.subject));
        text.push_str(&format!(
            "Date: {}\n",
            self.headers.date.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if !self.headers.to.is_empty() {
            let to_list: Vec<String> = self
                .headers
                .to
                .iter()
                .map(|addr| addr.to_string())
                .collect();
            text.push_str(&format!("To: {}\n", to_list.join(", ")));
        }

        if !self.headers.cc.is_empty() {
            let cc_list: Vec<String> = self
                .headers
                .cc
                .iter()
                .map(|addr| addr.to_string())
                .collect();
            text.push_str(&format!("CC: {}\n", cc_list.join(", ")));
        }

        text.push_str("\n--- Email Body ---\n\n");

        // Add body content
        if let Some(ref text_content) = self.body.text_content {
            text.push_str(text_content);
        } else if let Some(ref html_content) = self.body.html_content {
            // Simple HTML to text conversion
            text.push_str(&self.html_to_text(html_content));
        } else if let Some(ref md_content) = self.body.markdown_content {
            text.push_str(&self.markdown_to_text(md_content));
        }

        // Add attachment information
        if !self.attachments.is_empty() {
            text.push_str("\n\n--- Attachments ---\n");
            for attachment in self.attachments.values() {
                text.push_str(&format!(
                    "• {} ({} bytes)\n",
                    attachment.filename, attachment.size
                ));
            }
        }

        Ok(text)
    }

    pub fn get_html_body(&self) -> Option<&String> {
        self.body.html_content.as_ref()
    }

    pub fn get_text_body(&self) -> Option<&String> {
        self.body.text_content.as_ref()
    }

    // Conversion and formatting helpers
    fn markdown_to_html(&self, markdown: &str) -> String {
        // Simple markdown to HTML conversion
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html><body>");

        for line in markdown.lines() {
            if line.starts_with("# ") {
                html.push_str(&format!(
                    "<h1>{}</h1>",
                    html_escape(line.trim_start_matches("# "))
                ));
            } else if line.starts_with("## ") {
                html.push_str(&format!(
                    "<h2>{}</h2>",
                    html_escape(line.trim_start_matches("## "))
                ));
            } else if line.starts_with("### ") {
                html.push_str(&format!(
                    "<h3>{}</h3>",
                    html_escape(line.trim_start_matches("### "))
                ));
            } else if line.starts_with("- ") || line.starts_with("* ") {
                html.push_str(&format!(
                    "<li>{}</li>",
                    html_escape(line.trim_start_matches(if line.starts_with("- ") {
                        "- "
                    } else {
                        "* "
                    }))
                ));
            } else if !line.trim().is_empty() {
                html.push_str(&format!("<p>{}</p>", html_escape(line)));
            }
        }

        html.push_str("</body></html>");
        html
    }

    fn markdown_to_text(&self, markdown: &str) -> String {
        // Convert markdown to plain text
        let mut text = String::new();

        for line in markdown.lines() {
            if line.starts_with("# ") {
                text.push_str(&format!("{}\n", line.trim_start_matches("# ")));
                text.push_str(&"=".repeat(line.len() - 2));
                text.push_str("\n\n");
            } else if line.starts_with("## ") {
                text.push_str(&format!("{}\n", line.trim_start_matches("## ")));
                text.push_str(&"-".repeat(line.len() - 3));
                text.push_str("\n\n");
            } else if line.starts_with("### ") {
                text.push_str(&format!("{}\n\n", line.trim_start_matches("### ")));
            } else if line.starts_with("- ") || line.starts_with("* ") {
                text.push_str(&format!(
                    "• {}\n",
                    line.trim_start_matches(if line.starts_with("- ") { "- " } else { "* " })
                ));
            } else {
                text.push_str(&format!("{}\n", line));
            }
        }

        text
    }

    fn html_to_text(&self, html: &str) -> String {
        // Simple HTML to text conversion
        let text = html
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<p>", "\n")
            .replace("</p>", "\n")
            .replace("<div>", "\n")
            .replace("</div>", "\n")
            .replace("<li>", "• ")
            .replace("</li>", "\n");

        // Remove HTML tags
        let mut in_tag = false;
        let text_only: String = text
            .chars()
            .filter(|&c| {
                if c == '<' {
                    in_tag = true;
                    false
                } else if c == '>' {
                    in_tag = false;
                    false
                } else {
                    !in_tag
                }
            })
            .collect();

        // Clean up extra whitespace
        text_only
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn determine_content_type(&self, filename: &str) -> String {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "pdf" => "application/pdf",
            "doc" | "docx" => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            "xls" | "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" | "pptx" => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "zip" => "application/zip",
            "json" => "application/json",
            "xml" => "application/xml",
            "csv" => "text/csv",
            _ => "application/octet-stream",
        }
        .to_string()
    }

    // Email validation and formatting
    pub fn validate_email(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check required fields
        if self.headers.subject.trim().is_empty() {
            issues.push("Subject is empty".to_string());
        }

        if self.headers.to.is_empty() && self.headers.cc.is_empty() && self.headers.bcc.is_empty() {
            issues.push("No recipients specified".to_string());
        }

        if self.body.text_content.is_none() && self.body.html_content.is_none() {
            issues.push("Email body is empty".to_string());
        }

        // Validate email addresses
        for addr in &self.headers.to {
            if !self.is_valid_email(&addr.email) {
                issues.push(format!("Invalid TO address: {}", addr.email));
            }
        }

        for addr in &self.headers.cc {
            if !self.is_valid_email(&addr.email) {
                issues.push(format!("Invalid CC address: {}", addr.email));
            }
        }

        if !self.is_valid_email(&self.headers.from.email) {
            issues.push(format!("Invalid FROM address: {}", self.headers.from.email));
        }

        Ok(issues)
    }

    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && !email.trim().is_empty()
    }

    // Template and formatting methods
    pub fn apply_template(&mut self, template_name: &str) -> Result<()> {
        match template_name {
            "business" => self.apply_business_template(),
            "newsletter" => self.apply_newsletter_template(),
            "notification" => self.apply_notification_template(),
            "invitation" => self.apply_invitation_template(),
            _ => Err(OfficeError::UnsupportedFeature(format!(
                "Unknown template: {}",
                template_name
            ))),
        }
    }

    fn apply_business_template(&mut self) -> Result<()> {
        self.set_importance(EmailImportance::Normal);
        self.set_sensitivity(EmailSensitivity::Normal);
        self.add_custom_header("X-Auto-Response-Suppress", "DR, RN, NRN, OOF, AutoReply");
        self.add_tag("business");
        Ok(())
    }

    fn apply_newsletter_template(&mut self) -> Result<()> {
        self.set_importance(EmailImportance::Low);
        self.add_custom_header("List-Unsubscribe", "<mailto:unsubscribe@example.com>");
        self.add_custom_header("Precedence", "bulk");
        self.add_tag("newsletter");
        self.enable_tracking(Some("newsletter_campaign"));
        Ok(())
    }

    fn apply_notification_template(&mut self) -> Result<()> {
        self.set_importance(EmailImportance::High);
        self.add_custom_header("X-Auto-Response-Suppress", "All");
        self.add_tag("notification");
        Ok(())
    }

    fn apply_invitation_template(&mut self) -> Result<()> {
        self.set_importance(EmailImportance::Normal);
        self.request_read_receipt();
        self.add_tag("invitation");
        Ok(())
    }
}

impl OfficeDocument for EmlDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        // Parse EML using mail-parser
        let message = MessageParser::default()
            .parse(data)
            .ok_or_else(|| OfficeError::InvalidFormat("Invalid EML format".to_string()))?;

        let mut eml_doc = EmlDocument::new();

        // Extract headers
        if let Some(from) = message.from() {
            if let Some(addr) = from.first() {
                eml_doc.headers.from =
                    EmailAddress::new(addr.address().unwrap_or("unknown@example.com"), addr.name());
                eml_doc.author = eml_doc.headers.from.to_string();
            }
        }

        if let Some(to_list) = message.to() {
            eml_doc.headers.to = to_list
                .iter()
                .map(|addr| {
                    EmailAddress::new(addr.address().unwrap_or("unknown@example.com"), addr.name())
                })
                .collect();
        }

        if let Some(subject) = message.subject() {
            eml_doc.headers.subject = subject.to_string();
            eml_doc.subject = subject.to_string();
            eml_doc.title = subject.to_string();
        }

        if let Some(date) = message.date() {
            // Convert mail-parser DateTime to chrono DateTime
            let timestamp = date.to_timestamp();
            let utc_date = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
            eml_doc.headers.date = utc_date;
            eml_doc.created = utc_date;
        }

        if let Some(message_id) = message.message_id() {
            eml_doc.message_id = message_id.to_string();
            eml_doc.headers.message_id = message_id.to_string();
        }

        // Extract body content
        if let Some(text_body) = message.body_text(0) {
            eml_doc.body.text_content = Some(text_body.to_string());
        }

        if let Some(html_body) = message.body_html(0) {
            eml_doc.body.html_content = Some(html_body.to_string());
        }

        // Extract attachments
        for attachment in message.attachments() {
            if let Some(filename) = attachment.attachment_name() {
                let attachment_data = attachment.contents();
                let _attachment_id =
                    eml_doc.add_attachment_from_bytes(attachment_data, filename, None)?;
            }
        }

        eml_doc.modified = Utc::now();
        Ok(eml_doc)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut eml_content = String::new();

        // Generate email headers
        eml_content.push_str(&self.generate_headers());

        // Generate body content
        if self.attachments.is_empty() && self.body.html_content.is_none() {
            // Simple text email
            eml_content.push_str(&self.generate_simple_body());
        } else {
            // Multipart email
            eml_content.push_str(&self.generate_multipart_body());
        }

        Ok(eml_content.into_bytes())
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.to_bytes()?;
        fs::write(path, data)?;
        Ok(())
    }
}

impl EmlDocument {
    fn generate_headers(&self) -> String {
        let mut headers = String::new();

        // Essential headers
        headers.push_str(&format!("From: {}\r\n", self.headers.from.to_string()));

        if !self.headers.to.is_empty() {
            let to_list: Vec<String> = self
                .headers
                .to
                .iter()
                .map(|addr| addr.to_string())
                .collect();
            headers.push_str(&format!("To: {}\r\n", to_list.join(", ")));
        }

        if !self.headers.cc.is_empty() {
            let cc_list: Vec<String> = self
                .headers
                .cc
                .iter()
                .map(|addr| addr.to_string())
                .collect();
            headers.push_str(&format!("Cc: {}\r\n", cc_list.join(", ")));
        }

        headers.push_str(&format!("Subject: {}\r\n", self.headers.subject));
        headers.push_str(&format!(
            "Date: {}\r\n",
            self.headers.date.format("%a, %d %b %Y %H:%M:%S %z")
        ));
        headers.push_str(&format!("Message-ID: {}\r\n", self.message_id));
        headers.push_str(&format!("MIME-Version: {}\r\n", self.headers.mime_version));

        // Optional headers
        if let Some(ref reply_to) = self.headers.reply_to.first() {
            headers.push_str(&format!("Reply-To: {}\r\n", reply_to.to_string()));
        }

        if let Some(ref in_reply_to) = self.in_reply_to {
            headers.push_str(&format!("In-Reply-To: {}\r\n", in_reply_to));
        }

        if !self.references.is_empty() {
            headers.push_str(&format!("References: {}\r\n", self.references.join(" ")));
        }

        if let Some(ref x_mailer) = self.headers.x_mailer {
            headers.push_str(&format!("X-Mailer: {}\r\n", x_mailer));
        }

        if let Some(ref x_priority) = self.headers.x_priority {
            headers.push_str(&format!("X-Priority: {}\r\n", x_priority));
        }

        // Custom headers
        for (key, value) in &self.custom_headers {
            headers.push_str(&format!("{}: {}\r\n", key, value));
        }

        // Content type header
        headers.push_str(&format!(
            "Content-Type: {}\r\n",
            self.body.content_type.to_string()
        ));

        if self.attachments.is_empty() && self.body.html_content.is_none() {
            headers.push_str(&format!(
                "Content-Transfer-Encoding: {}\r\n",
                self.body.content_transfer_encoding
            ));
        }

        headers.push_str("\r\n"); // End of headers
        headers
    }

    fn generate_simple_body(&self) -> String {
        self.body.text_content.as_deref().unwrap_or("").to_string()
    }

    fn generate_multipart_body(&self) -> String {
        let boundary = self
            .body
            .content_type
            .boundary
            .as_deref()
            .unwrap_or_else(|| "boundary_office_sdk");

        let mut body = String::new();

        // Text and HTML parts
        if let Some(ref text_content) = self.body.text_content {
            body.push_str(&format!("--{}\r\n", boundary));
            body.push_str("Content-Type: text/plain; charset=UTF-8\r\n");
            body.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
            body.push_str(text_content);
            body.push_str("\r\n\r\n");
        }

        if let Some(ref html_content) = self.body.html_content {
            body.push_str(&format!("--{}\r\n", boundary));
            body.push_str("Content-Type: text/html; charset=UTF-8\r\n");
            body.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
            body.push_str(html_content);
            body.push_str("\r\n\r\n");
        }

        // Attachments
        for attachment in self.attachments.values() {
            body.push_str(&format!("--{}\r\n", boundary));
            body.push_str(&format!("Content-Type: {}\r\n", attachment.content_type));
            body.push_str(&format!(
                "Content-Disposition: {}\r\n",
                attachment.content_disposition
            ));
            body.push_str(&format!(
                "Content-Transfer-Encoding: {}\r\n",
                attachment.content_transfer_encoding
            ));

            if let Some(ref content_id) = attachment.content_id {
                body.push_str(&format!("Content-ID: <{}>\r\n", content_id));
            }

            body.push_str("\r\n");

            // Encode attachment data
            if attachment.content_transfer_encoding == "base64" {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&attachment.data);
                // Split into 76-character lines as per RFC
                for line in encoded.as_bytes().chunks(76) {
                    body.push_str(&String::from_utf8_lossy(line));
                    body.push_str("\r\n");
                }
            } else {
                body.push_str(&String::from_utf8_lossy(&attachment.data));
            }

            body.push_str("\r\n");
        }

        // End boundary
        body.push_str(&format!("--{}--\r\n", boundary));

        body
    }
}

// Utility functions
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
