use crate::OfficeDocument;
use crate::error::{OfficeError, Result};
use crate::utils::{decode_xml_reference, decode_xml_text, read_zip_entry, validate_office_format};
use chrono::{DateTime, Utc};
use quick_xml::{Reader, events::Event};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Clone)]
pub struct DocxDocument {
    pub content: String,
    pub styles: Option<String>,
    pub relationships: Option<String>,
    pub app_properties: Option<String>,
    pub core_properties: Option<String>,
    pub numbering: Option<String>,
    pub settings: Option<String>,
    pub web_settings: Option<String>,
    pub font_table: Option<String>,
    pub media: HashMap<String, Vec<u8>>,
    pub vba_project: Option<Vec<u8>>,
    pub metadata: DocumentMetadata,
    pub page_setup: PageSetup,
    pub images: HashMap<String, ImageData>,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub category: Option<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_modified_by: Option<String>,
    pub revision: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageSetup {
    pub width: f64,         // in twips (1/20th of a point)
    pub height: f64,        // in twips
    pub margin_top: f64,    // in twips
    pub margin_bottom: f64, // in twips
    pub margin_left: f64,   // in twips
    pub margin_right: f64,  // in twips
    pub orientation: PageOrientation,
    pub paper_size: PaperSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaperSize {
    Letter,
    A4,
    Legal,
    Executive,
    Custom { width: f64, height: f64 },
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub id: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub id: String,
    pub rows: Vec<TableRow>,
    pub style: Option<String>,
    pub width: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: String,
    pub width: Option<f64>,
    pub background_color: Option<String>,
    pub text_align: Option<TextAlign>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        let now = Utc::now();
        DocumentMetadata {
            title: None,
            author: Some("Office SDK User".to_string()),
            subject: None,
            description: None,
            keywords: None,
            category: None,
            created: now,
            modified: now,
            last_modified_by: Some("Office SDK".to_string()),
            revision: Some("1".to_string()),
            version: Some("1.0".to_string()),
        }
    }
}

impl Default for PageSetup {
    fn default() -> Self {
        PageSetup {
            width: 11906.0,        // A4 width in twips
            height: 16838.0,       // A4 height in twips
            margin_top: 1440.0,    // 1 inch in twips
            margin_bottom: 1440.0, // 1 inch in twips
            margin_left: 1440.0,   // 1 inch in twips
            margin_right: 1440.0,  // 1 inch in twips
            orientation: PageOrientation::Portrait,
            paper_size: PaperSize::A4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentXml {
    #[serde(rename = "body")]
    body: Option<Body>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "p")]
    paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Paragraph {
    #[serde(rename = "r")]
    runs: Vec<Run>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Run {
    #[serde(rename = "t")]
    text: String,
}

impl Default for DocxDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl DocxDocument {
    pub fn new() -> Self {
        let now = Utc::now();
        DocxDocument {
            content: Self::default_document_xml(),
            styles: Some(Self::comprehensive_styles_xml()),
            relationships: Some(Self::default_relationships_xml()),
            app_properties: Some(Self::default_app_xml()),
            core_properties: Some(Self::default_core_xml()),
            numbering: Some(Self::default_numbering_xml()),
            settings: Some(Self::default_settings_xml()),
            web_settings: Some(Self::default_web_settings_xml()),
            font_table: Some(Self::default_font_table_xml()),
            media: HashMap::new(),
            vba_project: None,
            metadata: DocumentMetadata {
                title: None,
                author: Some("Office SDK User".to_string()),
                subject: None,
                description: None,
                keywords: None,
                category: None,
                created: now,
                modified: now,
                last_modified_by: Some("Office SDK".to_string()),
                revision: Some("1".to_string()),
                version: Some("1.0".to_string()),
            },
            page_setup: PageSetup::default(),
            images: HashMap::new(),
            tables: Vec::new(),
        }
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.metadata.author = Some(author.to_string());
        self.metadata.last_modified_by = Some(author.to_string());
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.metadata.title = Some(title.to_string());
        self
    }

    pub fn with_subject(mut self, subject: &str) -> Self {
        self.metadata.subject = Some(subject.to_string());
        self
    }

    pub fn set_page_size(&mut self, paper_size: PaperSize) {
        match &paper_size {
            PaperSize::Letter => {
                self.page_setup.width = 12240.0; // 8.5 inches
                self.page_setup.height = 15840.0; // 11 inches
            }
            PaperSize::A4 => {
                self.page_setup.width = 11906.0; // 8.27 inches
                self.page_setup.height = 16838.0; // 11.69 inches
            }
            PaperSize::Legal => {
                self.page_setup.width = 12240.0; // 8.5 inches
                self.page_setup.height = 20160.0; // 14 inches
            }
            PaperSize::Executive => {
                self.page_setup.width = 10440.0; // 7.25 inches
                self.page_setup.height = 15120.0; // 10.5 inches
            }
            PaperSize::Custom { width, height } => {
                self.page_setup.width = *width;
                self.page_setup.height = *height;
            }
        }
        self.page_setup.paper_size = paper_size;
    }

    pub fn set_orientation(&mut self, orientation: PageOrientation) {
        self.page_setup.orientation = orientation;
        if orientation == PageOrientation::Landscape {
            std::mem::swap(&mut self.page_setup.width, &mut self.page_setup.height);
        }
    }

    pub fn set_margins(&mut self, top: f64, bottom: f64, left: f64, right: f64) {
        self.page_setup.margin_top = top;
        self.page_setup.margin_bottom = bottom;
        self.page_setup.margin_left = left;
        self.page_setup.margin_right = right;
    }

    pub fn add_paragraph(&mut self, text: &str) -> Result<()> {
        let paragraph_xml = format!(
            "<w:p xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:r><w:t>{}</w:t></w:r></w:p>",
            quick_xml::escape::escape(text)
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_heading(&mut self, level: u32, text: &str) -> Result<()> {
        let heading_style = format!("Heading{}", level);
        let paragraph_xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:pPr>
                    <w:pStyle w:val="{}"/>
                </w:pPr>
                <w:r>
                    <w:t>{}</w:t>
                </w:r>
            </w:p>"#,
            heading_style,
            quick_xml::escape::escape(text)
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_code_block(&mut self, code: &str) -> Result<()> {
        let paragraph_xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:pPr>
                    <w:pStyle w:val="CodeBlock"/>
                </w:pPr>
                <w:r>
                    <w:rPr>
                        <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
                        <w:color w:val="000000"/>
                        <w:sz w:val="20"/>
                    </w:rPr>
                    <w:t xml:space="preserve">{}</w:t>
                </w:r>
            </w:p>"#,
            quick_xml::escape::escape(code)
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_bulleted_list_item(&mut self, text: &str) -> Result<()> {
        let paragraph_xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:pPr>
                    <w:pStyle w:val="ListParagraph"/>
                    <w:numPr>
                        <w:ilvl w:val="0"/>
                        <w:numId w:val="1"/>
                    </w:numPr>
                </w:pPr>
                <w:r>
                    <w:t>{}</w:t>
                </w:r>
            </w:p>"#,
            quick_xml::escape::escape(text)
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_numbered_list_item(&mut self, text: &str, _number: usize) -> Result<()> {
        let paragraph_xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:pPr>
                    <w:pStyle w:val="ListParagraph"/>
                    <w:numPr>
                        <w:ilvl w:val="0"/>
                        <w:numId w:val="2"/>
                    </w:numPr>
                </w:pPr>
                <w:r>
                    <w:t>{}</w:t>
                </w:r>
            </w:p>"#,
            quick_xml::escape::escape(text)
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_quote(&mut self, text: &str) -> Result<()> {
        let paragraph_xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:pPr>
                    <w:pStyle w:val="Quote"/>
                    <w:ind w:left="720"/>
                </w:pPr>
                <w:r>
                    <w:rPr>
                        <w:i/>
                        <w:color w:val="666666"/>
                    </w:rPr>
                    <w:t>{}</w:t>
                </w:r>
            </w:p>"#,
            quick_xml::escape::escape(text)
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_horizontal_rule(&mut self) -> Result<()> {
        let paragraph_xml = r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:pPr>
                <w:pBdr>
                    <w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/>
                </w:pBdr>
            </w:pPr>
        </w:p>"#;

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, paragraph_xml);
        }

        Ok(())
    }

    pub fn add_image<P: AsRef<Path>>(
        &mut self,
        image_path: P,
        alt_text: Option<&str>,
    ) -> Result<String> {
        let path = image_path.as_ref();
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let image_id = format!("image{}", self.images.len() + 1);
        let content_type = match path.extension().and_then(|s| s.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("bmp") => "image/bmp",
            _ => "image/png", // default
        }
        .to_string();

        // Use image crate to get dimensions
        let (width, height) = match image::load_from_memory(&buffer) {
            Ok(img) => (Some(img.width()), Some(img.height())),
            Err(_) => (None, None),
        };

        let image_data = ImageData {
            id: image_id.clone(),
            data: buffer,
            content_type,
            width,
            height,
            alt_text: alt_text.map(|s| s.to_string()),
        };

        self.images.insert(image_id.clone(), image_data);
        self.add_image_to_document(&image_id)?;

        Ok(image_id)
    }

    pub fn add_image_from_bytes(
        &mut self,
        image_data: &[u8],
        content_type: &str,
        alt_text: Option<&str>,
    ) -> Result<String> {
        let image_id = format!("image{}", self.images.len() + 1);

        let (width, height) = match image::load_from_memory(image_data) {
            Ok(img) => (Some(img.width()), Some(img.height())),
            Err(_) => (None, None),
        };

        let image_data_struct = ImageData {
            id: image_id.clone(),
            data: image_data.to_vec(),
            content_type: content_type.to_string(),
            width,
            height,
            alt_text: alt_text.map(|s| s.to_string()),
        };

        self.images.insert(image_id.clone(), image_data_struct);
        self.add_image_to_document(&image_id)?;

        Ok(image_id)
    }

    fn add_image_to_document(&mut self, image_id: &str) -> Result<()> {
        let image_data = self.images.get(image_id).unwrap();
        let width_emu = image_data.width.unwrap_or(300) * 9525; // Convert pixels to EMUs
        let height_emu = image_data.height.unwrap_or(200) * 9525;

        let paragraph_xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:r>
                    <w:drawing>
                        <wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
                            <wp:extent cx="{}" cy="{}"/>
                            <wp:docPr id="1" name="{}"/>
                            <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
                                <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
                                    <pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
                                        <pic:nvPicPr>
                                            <pic:cNvPr id="1" name="{}"/>
                                            <pic:cNvPicPr/>
                                        </pic:nvPicPr>
                                        <pic:blipFill>
                                            <a:blip r:embed="rId{}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>
                                            <a:stretch>
                                                <a:fillRect/>
                                            </a:stretch>
                                        </pic:blipFill>
                                    </pic:pic>
                                </a:graphicData>
                            </a:graphic>
                        </wp:inline>
                    </w:drawing>
                </w:r>
            </w:p>"#,
            width_emu,
            height_emu,
            image_data.alt_text.as_deref().unwrap_or("Image"),
            image_data.alt_text.as_deref().unwrap_or("Image"),
            self.images.len() + 10 // Offset for other relationships
        );

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &paragraph_xml);
        }

        Ok(())
    }

    pub fn add_table(&mut self, table: Table) -> Result<String> {
        let table_id = table.id.clone();
        let table_xml = self.create_table_xml(&table)?;

        if let Some(body_end) = self.content.rfind("</w:body>") {
            self.content.insert_str(body_end, &table_xml);
        }

        self.tables.push(table);
        Ok(table_id)
    }

    pub fn create_table(&mut self, headers: &[&str], rows: &[Vec<&str>]) -> Result<String> {
        let table_id = format!("table{}", self.tables.len() + 1);

        let mut table_rows = Vec::new();

        // Add header row
        if !headers.is_empty() {
            let header_cells: Vec<TableCell> = headers
                .iter()
                .map(|&h| TableCell {
                    content: h.to_string(),
                    width: None,
                    background_color: Some("D9D9D9".to_string()), // Light gray for headers
                    text_align: Some(TextAlign::Center),
                })
                .collect();

            table_rows.push(TableRow {
                cells: header_cells,
                height: None,
            });
        }

        // Add data rows
        for row_data in rows {
            let cells: Vec<TableCell> = row_data
                .iter()
                .map(|&cell| TableCell {
                    content: cell.to_string(),
                    width: None,
                    background_color: None,
                    text_align: Some(TextAlign::Left),
                })
                .collect();

            table_rows.push(TableRow {
                cells,
                height: None,
            });
        }

        let table = Table {
            id: table_id.clone(),
            rows: table_rows,
            style: Some("TableGrid".to_string()),
            width: None,
        };

        self.add_table(table)?;
        Ok(table_id)
    }

    fn create_table_xml(&self, table: &Table) -> Result<String> {
        let mut xml = String::from(
            r#"<w:tbl xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:tblPr>
                <w:tblStyle w:val="TableGrid"/>
                <w:tblW w:w="0" w:type="auto"/>
                <w:tblLook w:val="04A0" w:firstRow="1" w:lastRow="0" w:firstColumn="1" w:lastColumn="0" w:noHBand="0" w:noVBand="1"/>
            </w:tblPr>
            <w:tblGrid>"#,
        );

        // Add table grid columns (simplified - equal width)
        if let Some(first_row) = table.rows.first() {
            for _ in &first_row.cells {
                xml.push_str(r#"<w:gridCol w:w="2880"/>"#);
            }
        }
        xml.push_str("</w:tblGrid>");

        // Add table rows
        for row in table.rows.iter() {
            xml.push_str("<w:tr>");

            for cell in &row.cells {
                xml.push_str("<w:tc>");

                // Cell properties
                xml.push_str("<w:tcPr>");
                if let Some(bg_color) = &cell.background_color {
                    xml.push_str(&format!(
                        r#"<w:shd w:val="clear" w:color="auto" w:fill="{}"/>"#,
                        bg_color
                    ));
                }
                if let Some(width) = cell.width {
                    xml.push_str(&format!(r#"<w:tcW w:w="{}" w:type="dxa"/>"#, width as u32));
                }
                xml.push_str("</w:tcPr>");

                // Cell content
                let align_val = match cell.text_align {
                    Some(TextAlign::Left) => "left",
                    Some(TextAlign::Center) => "center",
                    Some(TextAlign::Right) => "right",
                    Some(TextAlign::Justify) => "both",
                    None => "left",
                };

                xml.push_str(&format!(
                    r#"<w:p>
                        <w:pPr>
                            <w:jc w:val="{}"/>
                        </w:pPr>
                        <w:r>
                            <w:t>{}</w:t>
                        </w:r>
                    </w:p>"#,
                    align_val,
                    quick_xml::escape::escape(&cell.content)
                ));

                xml.push_str("</w:tc>");
            }

            xml.push_str("</w:tr>");
        }

        xml.push_str("</w:tbl>");
        Ok(xml)
    }

    pub fn add_vba_macro(&mut self, vba_data: Vec<u8>) -> Result<()> {
        self.vba_project = Some(vba_data);
        Ok(())
    }

    pub fn update_metadata(&mut self) {
        self.metadata.modified = Utc::now();
        if let Some(revision) = &self.metadata.revision
            && let Ok(rev_num) = revision.parse::<u32>()
        {
            self.metadata.revision = Some((rev_num + 1).to_string());
        }
    }

    pub fn get_text(&self) -> Result<String> {
        let mut reader = Reader::from_str(&self.content);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut text_content = String::new();
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == "w:t" => {
                    in_text = true;
                }
                Ok(Event::Text(e)) if in_text => {
                    text_content.push_str(&decode_xml_text(&e)?);
                }
                Ok(Event::GeneralRef(reference)) if in_text => {
                    text_content.push_str(&decode_xml_reference(&reference)?);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == "w:t" => {
                    in_text = false;
                    text_content.push(' ');
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(OfficeError::XmlParsing(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(text_content.trim().to_string())
    }

    fn default_document_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
    </w:body>
</w:document>"#
            .to_string()
    }

    fn comprehensive_styles_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:docDefaults>
        <w:rPrDefault>
            <w:rPr>
                <w:rFonts w:ascii="Calibri" w:eastAsia="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/>
                <w:sz w:val="22"/>
                <w:szCs w:val="22"/>
                <w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/>
            </w:rPr>
        </w:rPrDefault>
        <w:pPrDefault>
            <w:pPr>
                <w:spacing w:after="200" w:line="276" w:lineRule="auto"/>
            </w:pPr>
        </w:pPrDefault>
    </w:docDefaults>

    <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
        <w:name w:val="Normal"/>
        <w:qFormat/>
    </w:style>

    <w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont">
        <w:name w:val="Default Paragraph Font"/>
        <w:uiPriority w:val="1"/>
        <w:semiHidden/>
        <w:unhideWhenUsed/>
    </w:style>

    <w:style w:type="table" w:default="1" w:styleId="TableNormal">
        <w:name w:val="Normal Table"/>
        <w:uiPriority w:val="99"/>
        <w:semiHidden/>
        <w:unhideWhenUsed/>
        <w:tblPr>
            <w:tblInd w:w="0" w:type="dxa"/>
            <w:tblCellMar>
                <w:top w:w="0" w:type="dxa"/>
                <w:left w:w="108" w:type="dxa"/>
                <w:bottom w:w="0" w:type="dxa"/>
                <w:right w:w="108" w:type="dxa"/>
            </w:tblCellMar>
        </w:tblPr>
    </w:style>

    <w:style w:type="table" w:styleId="TableGrid">
        <w:name w:val="Table Grid"/>
        <w:basedOn w:val="TableNormal"/>
        <w:uiPriority w:val="39"/>
        <w:tblPr>
            <w:tblBorders>
                <w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                <w:left w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                <w:bottom w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                <w:right w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                <w:insideH w:val="single" w:sz="4" w:space="0" w:color="000000"/>
                <w:insideV w:val="single" w:sz="4" w:space="0" w:color="000000"/>
            </w:tblBorders>
        </w:tblPr>
    </w:style>

    <w:style w:type="paragraph" w:styleId="Heading1">
        <w:name w:val="heading 1"/>
        <w:basedOn w:val="Normal"/>
        <w:next w:val="Normal"/>
        <w:link w:val="Heading1Char"/>
        <w:uiPriority w:val="9"/>
        <w:qFormat/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="480" w:after="0"/>
            <w:outlineLvl w:val="0"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:asciiTheme="majorHAnsi" w:eastAsiaTheme="majorEastAsia" w:hAnsiTheme="majorHAnsi" w:cstheme="majorBidi"/>
            <w:b/>
            <w:bCs/>
            <w:color w:val="2F5496" w:themeColor="accent1" w:themeShade="BF"/>
            <w:sz w:val="32"/>
            <w:szCs w:val="32"/>
        </w:rPr>
    </w:style>

    <w:style w:type="paragraph" w:styleId="Heading2">
        <w:name w:val="heading 2"/>
        <w:basedOn w:val="Normal"/>
        <w:next w:val="Normal"/>
        <w:link w:val="Heading2Char"/>
        <w:uiPriority w:val="9"/>
        <w:qFormat/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="200" w:after="0"/>
            <w:outlineLvl w:val="1"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:asciiTheme="majorHAnsi" w:eastAsiaTheme="majorEastAsia" w:hAnsiTheme="majorHAnsi" w:cstheme="majorBidi"/>
            <w:b/>
            <w:bCs/>
            <w:color w:val="2F5496" w:themeColor="accent1" w:themeShade="BF"/>
            <w:sz w:val="26"/>
            <w:szCs w:val="26"/>
        </w:rPr>
    </w:style>

    <w:style w:type="paragraph" w:styleId="CodeBlock">
        <w:name w:val="Code Block"/>
        <w:basedOn w:val="Normal"/>
        <w:uiPriority w:val="99"/>
        <w:pPr>
            <w:spacing w:after="0"/>
            <w:ind w:left="720"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
            <w:color w:val="000000"/>
            <w:sz w:val="20"/>
            <w:szCs w:val="20"/>
        </w:rPr>
    </w:style>

    <w:style w:type="paragraph" w:styleId="Quote">
        <w:name w:val="Quote"/>
        <w:basedOn w:val="Normal"/>
        <w:uiPriority w:val="99"/>
        <w:pPr>
            <w:ind w:left="720"/>
            <w:spacing w:after="0"/>
        </w:pPr>
        <w:rPr>
            <w:i/>
            <w:color w:val="666666"/>
        </w:rPr>
    </w:style>

    <w:style w:type="paragraph" w:styleId="ListParagraph">
        <w:name w:val="List Paragraph"/>
        <w:basedOn w:val="Normal"/>
        <w:uiPriority w:val="34"/>
        <w:qFormat/>
        <w:pPr>
            <w:ind w:left="720"/>
            <w:contextualSpacing/>
        </w:pPr>
    </w:style>
</w:styles>"#.to_string()
    }

    fn default_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#.to_string()
    }

    fn default_app_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
    <Application>Office SDK</Application>
</Properties>"#
            .to_string()
    }

    fn default_core_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">
    <dc:creator xmlns:dc="http://purl.org/dc/elements/1.1/">Office SDK</dc:creator>
</cp:coreProperties>"#.to_string()
    }

    fn default_numbering_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:abstractNum w:abstractNumId="0">
        <w:nsid w:val="2DD860C0"/>
        <w:multiLevelType w:val="singleLevel"/>
        <w:lvl w:ilvl="0">
            <w:start w:val="1"/>
            <w:numFmt w:val="bullet"/>
            <w:lvlText w:val="·"/>
            <w:lvlJc w:val="left"/>
            <w:pPr>
                <w:ind w:left="720" w:hanging="360"/>
            </w:pPr>
        </w:lvl>
    </w:abstractNum>
    <w:abstractNum w:abstractNumId="1">
        <w:nsid w:val="3DD860C1"/>
        <w:multiLevelType w:val="singleLevel"/>
        <w:lvl w:ilvl="0">
            <w:start w:val="1"/>
            <w:numFmt w:val="decimal"/>
            <w:lvlText w:val="%1."/>
            <w:lvlJc w:val="left"/>
            <w:pPr>
                <w:ind w:left="720" w:hanging="360"/>
            </w:pPr>
        </w:lvl>
    </w:abstractNum>
    <w:num w:numId="1">
        <w:abstractNumId w:val="0"/>
    </w:num>
    <w:num w:numId="2">
        <w:abstractNumId w:val="1"/>
    </w:num>
</w:numbering>"#
            .to_string()
    }

    fn default_settings_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:zoom w:percent="100"/>
    <w:defaultTabStop w:val="708"/>
    <w:characterSpacingControl w:val="doNotCompress"/>
    <w:compat>
        <w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/>
    </w:compat>
</w:settings>"#.to_string()
    }

    fn default_web_settings_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:webSettings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:optimizeForBrowser/>
    <w:allowPNG/>
</w:webSettings>"#
            .to_string()
    }

    fn default_font_table_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:font w:name="Calibri">
        <w:panose1 w:val="020F0502020204030204"/>
        <w:charset w:val="00"/>
        <w:family w:val="swiss"/>
        <w:pitch w:val="variable"/>
        <w:sig w:usb0="E00002FF" w:usb1="4000ACFF" w:usb2="00000001" w:usb3="00000000" w:csb0="0000019F" w:csb1="00000000"/>
    </w:font>
    <w:font w:name="Courier New">
        <w:panose1 w:val="02070309020205020404"/>
        <w:charset w:val="00"/>
        <w:family w:val="modern"/>
        <w:pitch w:val="fixed"/>
        <w:sig w:usb0="E0002AFF" w:usb1="C0007841" w:usb2="00000009" w:usb3="00000000" w:csb0="000001FF" w:csb1="00000000"/>
    </w:font>
</w:fonts>"#.to_string()
    }

    pub fn create_enhanced_core_properties(&self) -> String {
        let created_str = self
            .metadata
            .created
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let modified_str = self
            .metadata
            .modified
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <dc:title>{}</dc:title>
    <dc:subject>{}</dc:subject>
    <dc:creator>{}</dc:creator>
    <cp:keywords>{}</cp:keywords>
    <dc:description>{}</dc:description>
    <cp:lastModifiedBy>{}</cp:lastModifiedBy>
    <cp:revision>{}</cp:revision>
    <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
    <dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>
    <cp:category>{}</cp:category>
</cp:coreProperties>"#,
            self.metadata.title.as_deref().unwrap_or(""),
            self.metadata.subject.as_deref().unwrap_or(""),
            self.metadata.author.as_deref().unwrap_or(""),
            self.metadata.keywords.as_deref().unwrap_or(""),
            self.metadata.description.as_deref().unwrap_or(""),
            self.metadata.last_modified_by.as_deref().unwrap_or(""),
            self.metadata.revision.as_deref().unwrap_or("1"),
            created_str,
            modified_str,
            self.metadata.category.as_deref().unwrap_or("")
        )
    }

    fn default_content_types_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
    <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
    <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#.to_string()
    }
}

impl OfficeDocument for DocxDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;

        validate_office_format(&mut archive, &["word/document.xml", "[Content_Types].xml"])?;

        let content = read_zip_entry(&mut archive, "word/document.xml")?;
        let styles = read_zip_entry(&mut archive, "word/styles.xml").ok();
        let relationships = read_zip_entry(&mut archive, "_rels/.rels").ok();
        let app_properties = read_zip_entry(&mut archive, "docProps/app.xml").ok();
        let core_properties = read_zip_entry(&mut archive, "docProps/core.xml").ok();

        // Read additional files if they exist
        let numbering = read_zip_entry(&mut archive, "word/numbering.xml").ok();
        let settings = read_zip_entry(&mut archive, "word/settings.xml").ok();
        let web_settings = read_zip_entry(&mut archive, "word/webSettings.xml").ok();
        let font_table = read_zip_entry(&mut archive, "word/fontTable.xml").ok();

        // Read media files
        let mut media = HashMap::new();
        let images = HashMap::new();
        let mut vba_project = None;

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name()?.into_owned();

            if name.starts_with("word/media/") {
                drop(file);
                let data = crate::utils::read_zip_entry_bytes(&mut archive, &name)?;
                let filename = name.split('/').next_back().unwrap_or(&name).to_string();
                media.insert(filename, data);
            } else if name.starts_with("word/vbaProject.bin") {
                drop(file);
                vba_project = Some(crate::utils::read_zip_entry_bytes(&mut archive, &name)?);
            }
        }

        Ok(DocxDocument {
            content,
            styles,
            relationships,
            app_properties,
            core_properties,
            numbering,
            settings,
            web_settings,
            font_table,
            media,
            vba_project,
            metadata: DocumentMetadata::default(),
            page_setup: PageSetup::default(),
            images,
            tables: Vec::new(),
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);

            zip.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
            zip.write_all(Self::default_content_types_xml().as_bytes())?;

            zip.start_file("_rels/.rels", SimpleFileOptions::default())?;
            zip.write_all(
                self.relationships
                    .as_ref()
                    .unwrap_or(&Self::default_relationships_xml())
                    .as_bytes(),
            )?;

            zip.start_file("word/document.xml", SimpleFileOptions::default())?;
            zip.write_all(self.content.as_bytes())?;

            if let Some(styles) = &self.styles {
                zip.start_file("word/styles.xml", SimpleFileOptions::default())?;
                zip.write_all(styles.as_bytes())?;
            }

            if let Some(numbering) = &self.numbering {
                zip.start_file("word/numbering.xml", SimpleFileOptions::default())?;
                zip.write_all(numbering.as_bytes())?;
            }

            if let Some(settings) = &self.settings {
                zip.start_file("word/settings.xml", SimpleFileOptions::default())?;
                zip.write_all(settings.as_bytes())?;
            }

            if let Some(web_settings) = &self.web_settings {
                zip.start_file("word/webSettings.xml", SimpleFileOptions::default())?;
                zip.write_all(web_settings.as_bytes())?;
            }

            if let Some(font_table) = &self.font_table {
                zip.start_file("word/fontTable.xml", SimpleFileOptions::default())?;
                zip.write_all(font_table.as_bytes())?;
            }

            // Write images
            for (image_id, image_data) in &self.images {
                let extension = match image_data.content_type.as_str() {
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    "image/gif" => "gif",
                    "image/bmp" => "bmp",
                    _ => "png",
                };
                let media_path = format!("word/media/{}.{}", image_id, extension);
                zip.start_file(&media_path, SimpleFileOptions::default())?;
                zip.write_all(&image_data.data)?;
            }

            // Write VBA project if present
            if let Some(vba_data) = &self.vba_project {
                zip.start_file("word/vbaProject.bin", SimpleFileOptions::default())?;
                zip.write_all(vba_data)?;
            }

            if let Some(app) = &self.app_properties {
                zip.start_file("docProps/app.xml", SimpleFileOptions::default())?;
                zip.write_all(app.as_bytes())?;
            }

            // Use enhanced core properties with metadata
            zip.start_file("docProps/core.xml", SimpleFileOptions::default())?;
            zip.write_all(self.create_enhanced_core_properties().as_bytes())?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_references_are_preserved_in_text() {
        let mut document = DocxDocument::new();
        document.content =
            "<w:document><w:body><w:p><w:r><w:t>Fish &amp; Chips &#33;</w:t></w:r></w:p></w:body></w:document>"
                .to_string();

        assert_eq!(document.get_text().unwrap(), "Fish & Chips !");
    }

    #[test]
    fn malformed_xml_entity_returns_error() {
        let mut document = DocxDocument::new();
        document.content =
            "<w:document><w:body><w:p><w:r><w:t>&unknown;</w:t></w:r></w:p></w:body></w:document>"
                .to_string();

        assert!(matches!(
            document.get_text(),
            Err(OfficeError::XmlParsing(_))
        ));
    }
}
