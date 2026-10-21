use crate::OfficeDocument;
use crate::error::{OfficeError, Result};
use crate::utils::{decode_xml_reference, decode_xml_text, read_zip_entry, validate_office_format};
use chrono::{DateTime, Utc};
use image::ImageReader;
use quick_xml::{Reader, events::Event};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Clone)]
pub struct PptxDocument {
    pub presentation: String,
    pub slides: HashMap<String, String>,
    pub slide_layouts: HashMap<String, String>,
    pub slide_masters: HashMap<String, String>,
    pub theme: Option<String>,
    pub relationships: Option<String>,
    pub app_properties: Option<String>,
    pub core_properties: Option<String>,

    // Advanced features matching DOCX and XLSX
    pub author: String,
    pub title: String,
    pub subject: String,
    pub keywords: String,
    pub description: String,
    pub category: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_modified_by: String,
    pub revision: u32,
    pub slide_size: SlideSize,
    pub images: HashMap<String, ImageInfo>,
    pub animations: HashMap<String, Animation>,
    pub vba_project: Option<Vec<u8>>,
    pub notes: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlideSize {
    Standard4x3,
    Widescreen16x9,
    A3,
    A4,
    B4ISO,
    B5ISO,
    Banner,
    Custom { width: u32, height: u32 },
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub id: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub width: u32,
    pub height: u32,
    pub alt_text: String,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub id: String,
    pub animation_type: AnimationType,
    pub target_shape: String,
    pub duration: f64,
    pub delay: f64,
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    FadeIn,
    FadeOut,
    SlideIn,
    SlideOut,
    Zoom,
    Rotate,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Slide {
    pub id: String,
    pub title: String,
    pub content: Vec<String>,
}

impl Default for SlideSize {
    fn default() -> Self {
        SlideSize::Widescreen16x9
    }
}

impl SlideSize {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            SlideSize::Standard4x3 => (9144000, 6858000),
            SlideSize::Widescreen16x9 => (12192000, 6858000),
            SlideSize::A3 => (14173200, 10063200),
            SlideSize::A4 => (10063200, 7581600),
            SlideSize::B4ISO => (11339600, 8035200),
            SlideSize::B5ISO => (8035200, 5715000),
            SlideSize::Banner => (7315200, 914400),
            SlideSize::Custom { width, height } => (*width, *height),
        }
    }
}

impl Default for PptxDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl PptxDocument {
    pub fn new() -> Self {
        let now = Utc::now();
        let mut doc = PptxDocument {
            presentation: Self::default_presentation_xml(),
            slides: HashMap::new(),
            slide_layouts: HashMap::new(),
            slide_masters: HashMap::new(),
            theme: Some(Self::default_theme_xml()),
            relationships: Some(Self::default_relationships_xml()),
            app_properties: Some(Self::default_app_xml()),
            core_properties: Some(Self::default_core_xml()),

            // Enhanced metadata
            author: "Office SDK".to_string(),
            title: "New Presentation".to_string(),
            subject: "".to_string(),
            keywords: "".to_string(),
            description: "".to_string(),
            category: "".to_string(),
            created: now,
            modified: now,
            last_modified_by: "Office SDK".to_string(),
            revision: 1,
            slide_size: SlideSize::default(),
            images: HashMap::new(),
            animations: HashMap::new(),
            vba_project: None,
            notes: HashMap::new(),
        };

        // Add default slide master and layout
        doc.slide_masters
            .insert("slideMaster1".to_string(), Self::default_slide_master_xml());
        doc.slide_layouts
            .insert("slideLayout1".to_string(), Self::default_slide_layout_xml());

        doc
    }

    pub fn add_slide(&mut self, title: &str, content: &[&str]) -> Result<String> {
        let slide_number = self.slides.len() + 1;
        let slide_id = format!("slide{}", slide_number);
        let slide_xml = self.create_slide_xml(title, content)?;
        self.slides.insert(slide_id.clone(), slide_xml);

        // Update presentation.xml to include the new slide
        self.update_presentation_xml();
        self.modified = Utc::now();

        Ok(slide_id)
    }

    // Enhanced metadata methods
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
        self.modified = Utc::now();
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.modified = Utc::now();
    }

    pub fn set_subject(&mut self, subject: &str) {
        self.subject = subject.to_string();
        self.modified = Utc::now();
    }

    pub fn set_keywords(&mut self, keywords: &str) {
        self.keywords = keywords.to_string();
        self.modified = Utc::now();
    }

    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
        self.modified = Utc::now();
    }

    pub fn set_category(&mut self, category: &str) {
        self.category = category.to_string();
        self.modified = Utc::now();
    }

    // Slide size and layout methods
    pub fn set_slide_size(&mut self, slide_size: SlideSize) {
        self.slide_size = slide_size;
        self.update_presentation_xml();
        self.modified = Utc::now();
    }

    pub fn get_slide_size(&self) -> SlideSize {
        self.slide_size
    }

    // Image methods
    pub fn add_image<P: AsRef<Path>>(
        &mut self,
        image_path: P,
        alt_text: Option<&str>,
    ) -> Result<String> {
        let image_data = std::fs::read(&image_path)?;
        let extension = image_path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png")
            .to_lowercase();

        let content_type = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            _ => "image/png",
        };

        self.add_image_from_bytes(&image_data, content_type, alt_text)
    }

    pub fn add_image_from_bytes(
        &mut self,
        image_data: &[u8],
        content_type: &str,
        alt_text: Option<&str>,
    ) -> Result<String> {
        let image_id = format!("image{}", self.images.len() + 1);

        let cursor = Cursor::new(image_data);
        let reader = ImageReader::new(cursor).with_guessed_format()?;
        let dimensions = reader.into_dimensions()?;

        let image_info = ImageInfo {
            id: image_id.clone(),
            data: image_data.to_vec(),
            content_type: content_type.to_string(),
            width: dimensions.0,
            height: dimensions.1,
            alt_text: alt_text.unwrap_or("Image").to_string(),
        };

        self.images.insert(image_id.clone(), image_info);
        self.modified = Utc::now();
        Ok(image_id)
    }

    // Animation methods
    pub fn add_animation(
        &mut self,
        target_shape: &str,
        animation_type: AnimationType,
        duration: f64,
        delay: f64,
    ) -> Result<String> {
        let animation_id = format!("anim{}", self.animations.len() + 1);

        let animation = Animation {
            id: animation_id.clone(),
            animation_type,
            target_shape: target_shape.to_string(),
            duration,
            delay,
        };

        self.animations.insert(animation_id.clone(), animation);
        self.modified = Utc::now();
        Ok(animation_id)
    }

    // Speaker notes methods
    pub fn add_slide_notes(&mut self, slide_id: &str, notes: &str) -> Result<()> {
        self.notes.insert(slide_id.to_string(), notes.to_string());
        self.modified = Utc::now();
        Ok(())
    }

    pub fn get_slide_notes(&self, slide_id: &str) -> Option<&String> {
        self.notes.get(slide_id)
    }

    // VBA methods
    pub fn add_vba_macro(&mut self, vba_data: Vec<u8>) -> Result<()> {
        self.vba_project = Some(vba_data);
        self.modified = Utc::now();
        Ok(())
    }

    pub fn has_vba_macros(&self) -> bool {
        self.vba_project.is_some()
    }

    pub fn get_vba_project(&self) -> Option<&Vec<u8>> {
        self.vba_project.as_ref()
    }

    // Advanced slide creation methods
    pub fn add_title_slide(&mut self, title: &str, subtitle: &str) -> Result<String> {
        self.add_slide(title, &[subtitle])
    }

    pub fn add_content_slide(&mut self, title: &str, bullet_points: &[&str]) -> Result<String> {
        self.add_slide(title, bullet_points)
    }

    pub fn add_image_slide(
        &mut self,
        title: &str,
        image_id: &str,
        caption: Option<&str>,
    ) -> Result<String> {
        let content = if let Some(cap) = caption {
            vec![format!("Image: {}", image_id), cap.to_string()]
        } else {
            vec![format!("Image: {}", image_id)]
        };

        let content_refs: Vec<&str> = content.iter().map(|s| s.as_str()).collect();
        self.add_slide(title, &content_refs)
    }

    // Statistics and analysis
    pub fn get_document_stats(&self) -> DocumentStats {
        let total_text_length: usize = self
            .slides
            .values()
            .map(|slide_xml| {
                if let Ok(slide) = self.parse_slide_xml("temp", slide_xml) {
                    slide.title.len() + slide.content.iter().map(|c| c.len()).sum::<usize>()
                } else {
                    0
                }
            })
            .sum();

        DocumentStats {
            slide_count: self.slides.len(),
            image_count: self.images.len(),
            animation_count: self.animations.len(),
            notes_count: self.notes.len(),
            total_text_length,
            has_vba: self.vba_project.is_some(),
            author: self.author.clone(),
            created: self.created,
            modified: self.modified,
        }
    }

    pub fn get_slide_content(&self, slide_id: &str) -> Result<Slide> {
        let slide_xml = self
            .slides
            .get(slide_id)
            .ok_or_else(|| OfficeError::MissingFile(format!("Slide: {}", slide_id)))?;

        self.parse_slide_xml(slide_id, slide_xml)
    }

    pub fn get_all_slides(&self) -> Result<Vec<Slide>> {
        let mut slides = Vec::new();

        // Sort slide IDs to maintain order (slide1, slide2, etc.)
        let mut slide_ids: Vec<String> = self.slides.keys().cloned().collect();
        slide_ids.sort_by(|a, b| {
            // Extract number from slide ID (e.g., "slide1" -> 1)
            let a_num = a
                .strip_prefix("slide")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let b_num = b
                .strip_prefix("slide")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            a_num.cmp(&b_num)
        });

        for slide_id in slide_ids {
            slides.push(self.get_slide_content(&slide_id)?);
        }
        Ok(slides)
    }

    pub fn get_slide_text(&self) -> Result<String> {
        let slides = self.get_all_slides()?;
        let mut text = String::new();

        for slide in slides {
            if !slide.title.is_empty() {
                text.push_str(&format!("{}\n", slide.title));
            }
            for content in slide.content {
                text.push_str(&format!("{}\n", content));
            }
            text.push('\n');
        }

        Ok(text.trim().to_string())
    }

    fn create_slide_xml(&self, title: &str, content: &[&str]) -> Result<String> {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>"#,
        );

        // Add title text box if title is provided
        if !title.is_empty() {
            xml.push_str(&format!(
                r#"
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="2" name="Title 1"/>
                    <p:cNvSpPr>
                        <a:spLocks noGrp="1"/>
                    </p:cNvSpPr>
                    <p:nvPr>
                        <p:ph type="ctrTitle"/>
                    </p:nvPr>
                </p:nvSpPr>
                <p:spPr/>
                <p:txBody>
                    <a:bodyPr/>
                    <a:lstStyle/>
                    <a:p>
                        <a:r>
                            <a:rPr lang="en-US"/>
                            <a:t>{}</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>"#,
                quick_xml::escape::escape(title)
            ));
        }

        // Add content text boxes
        for (i, text) in content.iter().enumerate() {
            xml.push_str(&format!(
                r#"
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="{}" name="Content {}"/>
                    <p:cNvSpPr>
                        <a:spLocks noGrp="1"/>
                    </p:cNvSpPr>
                    <p:nvPr>
                        <p:ph type="body"/>
                    </p:nvPr>
                </p:nvSpPr>
                <p:spPr/>
                <p:txBody>
                    <a:bodyPr/>
                    <a:lstStyle/>
                    <a:p>
                        <a:r>
                            <a:rPr lang="en-US"/>
                            <a:t>{}</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>"#,
                i + 3,
                i + 1,
                quick_xml::escape::escape(*text)
            ));
        }

        xml.push_str(
            r#"
        </p:spTree>
    </p:cSld>
    <p:clrMapOvr>
        <a:masterClrMapping/>
    </p:clrMapOvr>
</p:sld>"#,
        );

        Ok(xml)
    }

    fn parse_slide_xml(&self, slide_id: &str, slide_xml: &str) -> Result<Slide> {
        let mut reader = Reader::from_str(slide_xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut all_text = Vec::new();
        let mut current_text = String::new();
        let mut in_text = false;

        // Extract all text elements first
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == "a:t" => {
                    in_text = true;
                    current_text.clear();
                }
                Ok(Event::Text(e)) if in_text => {
                    current_text.push_str(&decode_xml_text(&e)?);
                }
                Ok(Event::GeneralRef(reference)) if in_text => {
                    current_text.push_str(&decode_xml_reference(&reference)?);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == "a:t" => {
                    in_text = false;
                    if !current_text.trim().is_empty() {
                        all_text.push(current_text.trim().to_string());
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(OfficeError::XmlParsing(e)),
                _ => {}
            }
            buf.clear();
        }

        // More sophisticated parsing: look for patterns in the created XML structure
        // When we create slides, title comes first if present, then content
        let title;
        let content;

        if all_text.is_empty() {
            title = String::new();
            content = Vec::new();
        } else if slide_xml.contains(r#"<p:ph type="ctrTitle"/>"#) {
            // If we have a title placeholder, first text is title
            title = all_text[0].clone();
            content = if all_text.len() > 1 {
                all_text[1..].to_vec()
            } else {
                Vec::new()
            };
        } else {
            // No title placeholder, all text is content
            title = String::new();
            content = all_text;
        }

        Ok(Slide {
            id: slide_id.to_string(),
            title,
            content,
        })
    }

    fn update_presentation_xml(&mut self) {
        let slide_count = self.slides.len();
        if slide_count == 0 {
            return;
        }

        let mut slides_xml = String::new();
        for i in 1..=slide_count {
            slides_xml.push_str(&format!(
                r#"        <p:sldId id="{}" r:id="rId{}"/>
"#,
                255 + i,
                i + 1
            ));
        }

        self.presentation = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:sldMasterIdLst>
        <p:sldMasterId id="2147483648" r:id="rId1"/>
    </p:sldMasterIdLst>
    <p:sldIdLst>
{}    </p:sldIdLst>
    <p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>
    <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
            slides_xml
        );
    }

    fn default_presentation_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:sldMasterIdLst>
        <p:sldMasterId id="2147483648" r:id="rId1"/>
    </p:sldMasterIdLst>
    <p:sldIdLst>
    </p:sldIdLst>
    <p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>
    <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#.to_string()
    }

    fn default_slide_master_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>
        </p:spTree>
    </p:cSld>
    <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
    <p:sldLayoutIdLst>
        <p:sldLayoutId id="2147483649" r:id="rId1"/>
    </p:sldLayoutIdLst>
</p:sldMaster>"#.to_string()
    }

    fn default_slide_layout_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="title" preserve="1">
    <p:cSld name="Title Slide">
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>
        </p:spTree>
    </p:cSld>
</p:sldLayout>"#.to_string()
    }

    fn default_theme_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
    <a:themeElements>
        <a:clrScheme name="Office">
            <a:dk1>
                <a:sysClr val="windowText" lastClr="000000"/>
            </a:dk1>
            <a:lt1>
                <a:sysClr val="window" lastClr="FFFFFF"/>
            </a:lt1>
            <a:dk2>
                <a:srgbClr val="1F497D"/>
            </a:dk2>
            <a:lt2>
                <a:srgbClr val="EEECE1"/>
            </a:lt2>
            <a:accent1>
                <a:srgbClr val="4F81BD"/>
            </a:accent1>
            <a:accent2>
                <a:srgbClr val="F79646"/>
            </a:accent2>
            <a:accent3>
                <a:srgbClr val="9BBB59"/>
            </a:accent3>
            <a:accent4>
                <a:srgbClr val="8064A2"/>
            </a:accent4>
            <a:accent5>
                <a:srgbClr val="4BACC6"/>
            </a:accent5>
            <a:accent6>
                <a:srgbClr val="F366A7"/>
            </a:accent6>
            <a:hlink>
                <a:srgbClr val="0000FF"/>
            </a:hlink>
            <a:folHlink>
                <a:srgbClr val="800080"/>
            </a:folHlink>
        </a:clrScheme>
        <a:fontScheme name="Office">
            <a:majorFont>
                <a:latin typeface="Calibri"/>
                <a:ea typeface=""/>
                <a:cs typeface=""/>
            </a:majorFont>
            <a:minorFont>
                <a:latin typeface="Calibri"/>
                <a:ea typeface=""/>
                <a:cs typeface=""/>
            </a:minorFont>
        </a:fontScheme>
        <a:fmtScheme name="Office">
            <a:fillStyleLst>
                <a:solidFill>
                    <a:schemeClr val="phClr"/>
                </a:solidFill>
            </a:fillStyleLst>
            <a:lnStyleLst>
                <a:ln w="9525" cap="flat" cmpd="sng" algn="ctr">
                    <a:solidFill>
                        <a:schemeClr val="phClr">
                            <a:shade val="95000"/>
                        </a:schemeClr>
                    </a:solidFill>
                </a:ln>
            </a:lnStyleLst>
            <a:effectStyleLst>
                <a:effectStyle>
                    <a:effectLst>
                        <a:outerShdw blurRad="40000" dist="20000" dir="5400000" rotWithShape="0">
                            <a:srgbClr val="000000">
                                <a:alpha val="38000"/>
                            </a:srgbClr>
                        </a:outerShdw>
                    </a:effectLst>
                </a:effectStyle>
            </a:effectStyleLst>
            <a:bgFillStyleLst>
                <a:solidFill>
                    <a:schemeClr val="phClr"/>
                </a:solidFill>
            </a:bgFillStyleLst>
        </a:fmtScheme>
    </a:themeElements>
</a:theme>"#
            .to_string()
    }

    fn default_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#.to_string()
    }

    fn default_app_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
    <Application>Office SDK</Application>
    <PresentationFormat>On-screen Show (4:3)</PresentationFormat>
    <Slides>0</Slides>
</Properties>"#
            .to_string()
    }

    fn default_core_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">
    <dc:creator xmlns:dc="http://purl.org/dc/elements/1.1/">Office SDK</dc:creator>
</cp:coreProperties>"#.to_string()
    }

    fn default_content_types_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
    <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
    <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
    <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
    <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
    <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#.to_string()
    }

    fn presentation_relationships_xml(&self) -> String {
        let mut relationships = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
"#,
        );

        for i in 1..=self.slides.len() {
            relationships.push_str(&format!(
                r#"    <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>
"#, i + 1, i));
        }

        relationships.push_str("</Relationships>");
        relationships
    }

    fn slide_master_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#.to_string()
    }

    fn slide_layout_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#.to_string()
    }

    fn slide_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#.to_string()
    }
}

impl OfficeDocument for PptxDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;

        validate_office_format(
            &mut archive,
            &["ppt/presentation.xml", "[Content_Types].xml"],
        )?;

        let presentation = read_zip_entry(&mut archive, "ppt/presentation.xml")?;
        let theme = read_zip_entry(&mut archive, "ppt/theme/theme1.xml").ok();
        let relationships = read_zip_entry(&mut archive, "_rels/.rels").ok();
        let app_properties = read_zip_entry(&mut archive, "docProps/app.xml").ok();
        let core_properties = read_zip_entry(&mut archive, "docProps/core.xml").ok();

        let mut slides = HashMap::new();
        let mut slide_layouts = HashMap::new();
        let mut slide_masters = HashMap::new();

        // Read slides
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name()?.into_owned();

            if name.starts_with("ppt/slides/") && name.ends_with(".xml") {
                drop(file);
                let content = read_zip_entry(&mut archive, &name)?;
                let slide_name = name.replace("ppt/slides/", "").replace(".xml", "");
                slides.insert(slide_name, content);
            } else if name.starts_with("ppt/slideLayouts/") && name.ends_with(".xml") {
                drop(file);
                let content = read_zip_entry(&mut archive, &name)?;
                let layout_name = name.replace("ppt/slideLayouts/", "").replace(".xml", "");
                slide_layouts.insert(layout_name, content);
            } else if name.starts_with("ppt/slideMasters/") && name.ends_with(".xml") {
                drop(file);
                let content = read_zip_entry(&mut archive, &name)?;
                let master_name = name.replace("ppt/slideMasters/", "").replace(".xml", "");
                slide_masters.insert(master_name, content);
            }
        }

        // Ensure we have at least the default structures
        if slide_layouts.is_empty() {
            slide_layouts.insert("slideLayout1".to_string(), Self::default_slide_layout_xml());
        }
        if slide_masters.is_empty() {
            slide_masters.insert("slideMaster1".to_string(), Self::default_slide_master_xml());
        }

        // Parse metadata from core properties if available
        let now = Utc::now();
        let (
            author,
            title,
            subject,
            description,
            keywords,
            category,
            created,
            modified,
            last_modified_by,
            revision,
        ) = if let Some(ref core_xml) = core_properties {
            Self::parse_core_properties(core_xml).unwrap_or_else(|_| {
                (
                    "Office SDK".to_string(),
                    "New Presentation".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    now,
                    now,
                    "Office SDK".to_string(),
                    1,
                )
            })
        } else {
            (
                "Office SDK".to_string(),
                "New Presentation".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                now,
                now,
                "Office SDK".to_string(),
                1,
            )
        };

        Ok(PptxDocument {
            presentation,
            slides,
            slide_layouts,
            slide_masters,
            theme,
            relationships,
            app_properties,
            core_properties,
            author,
            title,
            subject,
            keywords,
            description,
            category,
            created,
            modified,
            last_modified_by,
            revision,
            slide_size: SlideSize::default(),
            images: HashMap::new(),
            animations: HashMap::new(),
            vba_project: None,
            notes: HashMap::new(),
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);

            // Content types
            zip.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
            let mut content_types = Self::default_content_types_xml();

            // Add slide content types
            for i in 1..=self.slides.len() {
                let slide_override = format!(
                    r#"    <Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#,
                    i
                );
                // Find the position of </Types> and insert before it
                if let Some(pos) = content_types.rfind("</Types>") {
                    content_types.insert_str(pos, &slide_override);
                }
            }
            zip.write_all(content_types.as_bytes())?;

            // Root relationships
            zip.start_file("_rels/.rels", SimpleFileOptions::default())?;
            zip.write_all(
                self.relationships
                    .as_ref()
                    .unwrap_or(&Self::default_relationships_xml())
                    .as_bytes(),
            )?;

            // Presentation
            zip.start_file("ppt/presentation.xml", SimpleFileOptions::default())?;
            zip.write_all(self.presentation.as_bytes())?;

            // Presentation relationships
            zip.start_file(
                "ppt/_rels/presentation.xml.rels",
                SimpleFileOptions::default(),
            )?;
            zip.write_all(self.presentation_relationships_xml().as_bytes())?;

            // Slide masters
            for (master_name, master_content) in &self.slide_masters {
                let path = format!("ppt/slideMasters/{}.xml", master_name);
                zip.start_file(&path, SimpleFileOptions::default())?;
                zip.write_all(master_content.as_bytes())?;

                // Slide master relationships
                let rels_path = format!("ppt/slideMasters/_rels/{}.xml.rels", master_name);
                zip.start_file(&rels_path, SimpleFileOptions::default())?;
                zip.write_all(Self::slide_master_relationships_xml().as_bytes())?;
            }

            // Slide layouts
            for (layout_name, layout_content) in &self.slide_layouts {
                let path = format!("ppt/slideLayouts/{}.xml", layout_name);
                zip.start_file(&path, SimpleFileOptions::default())?;
                zip.write_all(layout_content.as_bytes())?;

                // Slide layout relationships
                let rels_path = format!("ppt/slideLayouts/_rels/{}.xml.rels", layout_name);
                zip.start_file(&rels_path, SimpleFileOptions::default())?;
                zip.write_all(Self::slide_layout_relationships_xml().as_bytes())?;
            }

            // Slides - write in correct order
            let mut slide_ids: Vec<String> = self.slides.keys().cloned().collect();
            slide_ids.sort_by(|a, b| {
                let a_num = a
                    .strip_prefix("slide")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
                let b_num = b
                    .strip_prefix("slide")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
                a_num.cmp(&b_num)
            });

            for (i, slide_id) in slide_ids.iter().enumerate() {
                let slide_content = &self.slides[slide_id];
                let path = format!("ppt/slides/slide{}.xml", i + 1);
                zip.start_file(&path, SimpleFileOptions::default())?;
                zip.write_all(slide_content.as_bytes())?;

                // Slide relationships
                let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", i + 1);
                zip.start_file(&rels_path, SimpleFileOptions::default())?;
                zip.write_all(Self::slide_relationships_xml().as_bytes())?;
            }

            // Theme
            if let Some(theme) = &self.theme {
                zip.start_file("ppt/theme/theme1.xml", SimpleFileOptions::default())?;
                zip.write_all(theme.as_bytes())?;
            }

            // App properties
            if let Some(app) = &self.app_properties {
                zip.start_file("docProps/app.xml", SimpleFileOptions::default())?;
                zip.write_all(app.as_bytes())?;
            }

            // Core properties
            if let Some(core) = &self.core_properties {
                zip.start_file("docProps/core.xml", SimpleFileOptions::default())?;
                zip.write_all(core.as_bytes())?;
            }

            zip.finish()?;
        }
        Ok(buf)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.to_bytes()?;
        fs::write(path, data)?;
        Ok(())
    }
}

impl PptxDocument {
    fn parse_core_properties(
        xml: &str,
    ) -> Result<(
        String,
        String,
        String,
        String,
        String,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
        String,
        u32,
    )> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut current_element = String::new();
        let mut current_text = String::new();
        let mut author = "Office SDK".to_string();
        let mut title = "New Presentation".to_string();
        let mut subject = "".to_string();
        let mut description = "".to_string();
        let mut keywords = "".to_string();
        let mut category = "".to_string();
        let mut created = Utc::now();
        let mut modified = Utc::now();
        let mut last_modified_by = "Office SDK".to_string();
        let mut revision = 1;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_element = e.local_name().as_ref().to_string();
                    current_text.clear();
                }
                Ok(Event::Text(e)) => {
                    current_text.push_str(&decode_xml_text(&e)?);
                }
                Ok(Event::GeneralRef(reference)) => {
                    current_text.push_str(&decode_xml_reference(&reference)?);
                }
                Ok(Event::End(_)) => {
                    let text = current_text.trim().to_string();
                    match current_element.as_str() {
                        "creator" => author = text,
                        "title" => title = text,
                        "subject" => subject = text,
                        "description" => description = text,
                        "keywords" => keywords = text,
                        "category" => category = text,
                        "lastModifiedBy" => last_modified_by = text,
                        "created" => {
                            if let Ok(dt) = DateTime::parse_from_rfc3339(&text) {
                                created = dt.with_timezone(&Utc);
                            }
                        }
                        "modified" => {
                            if let Ok(dt) = DateTime::parse_from_rfc3339(&text) {
                                modified = dt.with_timezone(&Utc);
                            }
                        }
                        "revision" => {
                            if let Ok(rev) = text.parse::<u32>() {
                                revision = rev;
                            }
                        }
                        _ => {}
                    }
                    current_element.clear();
                    current_text.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(OfficeError::XmlParsing(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok((
            author,
            title,
            subject,
            description,
            keywords,
            category,
            created,
            modified,
            last_modified_by,
            revision,
        ))
    }

    fn create_notes_xml(&self, notes_text: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="2" name="Notes 1"/>
                    <p:cNvSpPr>
                        <a:spLocks noGrp="1"/>
                    </p:cNvSpPr>
                    <p:nvPr>
                        <p:ph type="body" idx="1"/>
                    </p:nvPr>
                </p:nvSpPr>
                <p:spPr/>
                <p:txBody>
                    <a:bodyPr/>
                    <a:lstStyle/>
                    <a:p>
                        <a:r>
                            <a:rPr lang="en-US"/>
                            <a:t>{}</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:notes>"#,
            quick_xml::escape::escape(notes_text)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_text_preserves_xml_references() {
        let mut document = PptxDocument::new();
        document.slides.insert(
            "slide1".to_string(),
            "<p:sld><p:ph type=\"ctrTitle\"/><a:t>Title &amp; More</a:t><a:t>Fish &#38; Chips</a:t></p:sld>".to_string(),
        );

        let slide = document.get_slide_content("slide1").unwrap();
        assert_eq!(slide.title, "Title & More");
        assert_eq!(slide.content, vec!["Fish & Chips"]);
    }

    #[test]
    fn core_properties_use_local_names_and_preserve_references() {
        let properties = PptxDocument::parse_core_properties(
            "<cp:coreProperties><dc:creator>A &amp; B</dc:creator><dc:title>Title &#38; More</dc:title></cp:coreProperties>",
        )
        .unwrap();

        assert_eq!(properties.0, "A & B");
        assert_eq!(properties.1, "Title & More");
    }
}

#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub slide_count: usize,
    pub image_count: usize,
    pub animation_count: usize,
    pub notes_count: usize,
    pub total_text_length: usize,
    pub has_vba: bool,
    pub author: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}
