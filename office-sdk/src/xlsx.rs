use crate::OfficeDocument;
use crate::error::{OfficeError, Result};
use crate::utils::{
    decode_xml_reference, decode_xml_text, read_zip_entry, read_zip_entry_bytes,
    validate_office_format,
};
use chrono::{DateTime, Utc};
use quick_xml::{Reader, events::Event};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Clone)]
pub struct XlsxDocument {
    pub workbook: String,
    pub worksheets: HashMap<String, String>,
    pub shared_strings: Option<String>,
    pub styles: Option<String>,
    pub relationships: Option<String>,
    pub app_properties: Option<String>,
    pub core_properties: Option<String>,

    // Advanced features matching DOCX
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
    pub page_setup: PageSetup,
    pub images: HashMap<String, ImageInfo>,
    pub charts: HashMap<String, Chart>,
    pub vba_project: Option<Vec<u8>>,
    pub tables: HashMap<String, Table>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperSize {
    Letter,
    A4,
    A3,
    Legal,
    Tabloid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub struct PageSetup {
    pub paper_size: PaperSize,
    pub orientation: PageOrientation,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_header: f64,
    pub margin_footer: f64,
    pub scale: u32,
    pub fit_to_width: Option<u32>,
    pub fit_to_height: Option<u32>,
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
pub struct Chart {
    pub id: String,
    pub chart_type: ChartType,
    pub title: String,
    pub data_range: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum ChartType {
    Column,
    Line,
    Pie,
    Bar,
    Area,
    Scatter,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub id: String,
    pub name: String,
    pub range: String,
    pub headers: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub style: TableStyle,
}

#[derive(Debug, Clone)]
pub struct TableStyle {
    pub name: String,
    pub show_first_column: bool,
    pub show_last_column: bool,
    pub show_row_stripes: bool,
    pub show_column_stripes: bool,
}

#[derive(Debug, Clone)]
pub struct Worksheet {
    pub name: String,
    pub data: Vec<Vec<String>>,
}

impl Default for PageSetup {
    fn default() -> Self {
        PageSetup {
            paper_size: PaperSize::Letter,
            orientation: PageOrientation::Portrait,
            margin_top: 0.75,
            margin_bottom: 0.75,
            margin_left: 0.7,
            margin_right: 0.7,
            margin_header: 0.3,
            margin_footer: 0.3,
            scale: 100,
            fit_to_width: None,
            fit_to_height: None,
        }
    }
}

impl Default for TableStyle {
    fn default() -> Self {
        TableStyle {
            name: "TableStyleMedium2".to_string(),
            show_first_column: false,
            show_last_column: false,
            show_row_stripes: true,
            show_column_stripes: false,
        }
    }
}

impl Default for XlsxDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl XlsxDocument {
    pub fn new() -> Self {
        let now = Utc::now();
        let mut doc = XlsxDocument {
            workbook: Self::default_workbook_xml(),
            worksheets: HashMap::new(),
            shared_strings: Some(Self::default_shared_strings_xml()),
            styles: Some(Self::default_styles_xml()),
            relationships: Some(Self::default_relationships_xml()),
            app_properties: Some(Self::default_app_xml()),
            core_properties: Some(Self::default_core_xml()),

            // Enhanced metadata
            author: "Office SDK".to_string(),
            title: "New Spreadsheet".to_string(),
            subject: "".to_string(),
            keywords: "".to_string(),
            description: "".to_string(),
            category: "".to_string(),
            created: now,
            modified: now,
            last_modified_by: "Office SDK".to_string(),
            revision: 1,
            page_setup: PageSetup::default(),
            images: HashMap::new(),
            charts: HashMap::new(),
            vba_project: None,
            tables: HashMap::new(),
        };

        doc.worksheets
            .insert("sheet1".to_string(), Self::default_worksheet_xml());
        doc
    }

    pub fn add_worksheet(&mut self, name: &str, data: &[Vec<String>]) -> Result<()> {
        let worksheet_xml = self.create_worksheet_xml(data)?;
        self.worksheets.insert(name.to_string(), worksheet_xml);
        Ok(())
    }

    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
        self.last_modified_by = author.to_string();
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

    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
        self.modified = Utc::now();
    }

    pub fn set_keywords(&mut self, keywords: &str) {
        self.keywords = keywords.to_string();
        self.modified = Utc::now();
    }

    pub fn set_category(&mut self, category: &str) {
        self.category = category.to_string();
        self.modified = Utc::now();
    }

    pub fn set_page_size(&mut self, paper_size: PaperSize) {
        self.page_setup.paper_size = paper_size;
    }

    pub fn set_orientation(&mut self, orientation: PageOrientation) {
        self.page_setup.orientation = orientation;
    }

    /// Set top, bottom, left and right margins in inches.
    pub fn set_margins(&mut self, top: f64, bottom: f64, left: f64, right: f64) {
        self.page_setup.margin_top = top;
        self.page_setup.margin_bottom = bottom;
        self.page_setup.margin_left = left;
        self.page_setup.margin_right = right;
    }

    pub fn set_scale(&mut self, scale: u32) {
        self.page_setup.scale = scale;
    }

    /// Add a chart to the document's in-memory chart collection.
    pub fn add_chart(
        &mut self,
        chart_type: ChartType,
        title: &str,
        data_range: &str,
        width: u32,
        height: u32,
    ) -> Result<String> {
        let id = format!("chart{}", self.charts.len() + 1);
        self.charts.insert(
            id.clone(),
            Chart {
                id: id.clone(),
                chart_type,
                title: title.to_string(),
                data_range: data_range.to_string(),
                width,
                height,
            },
        );
        self.modified = Utc::now();
        Ok(id)
    }

    /// Add a table to the document's in-memory table collection.
    pub fn create_table(
        &mut self,
        name: &str,
        range: &str,
        headers: &[&str],
        data: &[Vec<&str>],
    ) -> Result<String> {
        let id = format!("table{}", self.tables.len() + 1);
        self.tables.insert(
            id.clone(),
            Table {
                id: id.clone(),
                name: name.to_string(),
                range: range.to_string(),
                headers: headers.iter().map(|value| value.to_string()).collect(),
                data: data
                    .iter()
                    .map(|row| row.iter().map(|value| value.to_string()).collect())
                    .collect(),
                style: TableStyle::default(),
            },
        );
        self.modified = Utc::now();
        Ok(id)
    }

    /// Store an already compiled VBA project (vbaProject.bin).
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

    pub fn get_text(&self) -> Result<String> {
        let mut names: Vec<_> = self.worksheets.keys().collect();
        names.sort();
        let mut text = String::new();
        for name in names {
            for row in self.get_worksheet_data(name)? {
                text.push_str(&row.join("\t"));
                text.push('\n');
            }
        }
        Ok(text)
    }

    pub fn get_document_stats(&self) -> DocumentStats {
        DocumentStats {
            worksheet_count: self.worksheets.len(),
            image_count: self.images.len(),
            chart_count: self.charts.len(),
            table_count: self.tables.len(),
            total_cells: self
                .worksheets
                .keys()
                .filter_map(|name| self.get_worksheet_data(name).ok())
                .flatten()
                .map(|row| row.len())
                .sum(),
            has_vba: self.has_vba_macros(),
            author: self.author.clone(),
            created: self.created,
            modified: self.modified,
        }
    }

    fn create_core_properties(&self) -> String {
        use quick_xml::escape::escape;
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:creator>{}</dc:creator><dc:title>{}</dc:title><dc:subject>{}</dc:subject><dc:description>{}</dc:description><cp:keywords>{}</cp:keywords><cp:category>{}</cp:category><cp:lastModifiedBy>{}</cp:lastModifiedBy><cp:revision>{}</cp:revision><dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>
</cp:coreProperties>"#,
            escape(&self.author),
            escape(&self.title),
            escape(&self.subject),
            escape(&self.description),
            escape(&self.keywords),
            escape(&self.category),
            escape(&self.last_modified_by),
            self.revision,
            self.created.to_rfc3339(),
            self.modified.to_rfc3339()
        )
    }

    pub fn get_worksheet_data(&self, sheet_name: &str) -> Result<Vec<Vec<String>>> {
        let worksheet_xml = self
            .worksheets
            .get(sheet_name)
            .ok_or_else(|| OfficeError::MissingFile(format!("Worksheet: {}", sheet_name)))?;

        self.parse_worksheet_data(worksheet_xml)
    }

    fn create_worksheet_xml(&self, data: &[Vec<String>]) -> Result<String> {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
    <sheetData>"#,
        );

        for (row_idx, row) in data.iter().enumerate() {
            xml.push_str(&format!("\n        <row r=\"{}\">", row_idx + 1));

            for (col_idx, cell_value) in row.iter().enumerate() {
                let col_letter = self.number_to_column(col_idx + 1);
                let cell_ref = format!("{}{}", col_letter, row_idx + 1);

                if cell_value.parse::<f64>().is_ok() {
                    xml.push_str(&format!(
                        "\n            <c r=\"{}\"><v>{}</v></c>",
                        cell_ref, cell_value
                    ));
                } else {
                    xml.push_str(&format!(
                        "\n            <c r=\"{}\" t=\"inlineStr\"><is><t>{}</t></is></c>",
                        cell_ref,
                        quick_xml::escape::escape(cell_value)
                    ));
                }
            }

            xml.push_str("\n        </row>");
        }

        xml.push_str("\n    </sheetData>\n</worksheet>");
        Ok(xml)
    }

    fn parse_worksheet_data(&self, worksheet_xml: &str) -> Result<Vec<Vec<String>>> {
        let mut reader = Reader::from_str(worksheet_xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut data = Vec::new();
        let mut current_row = Vec::new();
        let mut current_row_num = 0;
        let mut in_cell_value = false;
        let mut current_cell_value = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    "row" => {
                        if let Some(Ok(attr)) = e
                            .attributes()
                            .find(|attr| attr.as_ref().map(|a| a.key.as_ref()) == Ok("r"))
                            && let Ok(row_num) = attr.value.parse::<usize>()
                        {
                            while data.len() < row_num - 1 {
                                data.push(Vec::new());
                            }
                            current_row = Vec::new();
                            current_row_num = row_num - 1;
                        }
                    }
                    "v" | "t" => {
                        in_cell_value = true;
                        current_cell_value.clear();
                    }
                    _ => {}
                },
                Ok(Event::Text(e)) if in_cell_value => {
                    current_cell_value.push_str(&decode_xml_text(&e)?);
                }
                Ok(Event::GeneralRef(reference)) if in_cell_value => {
                    current_cell_value.push_str(&decode_xml_reference(&reference)?);
                }
                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    "row" => {
                        if data.len() <= current_row_num {
                            data.resize(current_row_num + 1, Vec::new());
                        }
                        data[current_row_num] = current_row.clone();
                    }
                    "v" | "t" => {
                        current_row.push(std::mem::take(&mut current_cell_value));
                        in_cell_value = false;
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(OfficeError::XmlParsing(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(data)
    }

    fn number_to_column(&self, mut num: usize) -> String {
        let mut result = String::new();
        while num > 0 {
            num -= 1;
            result = ((b'A' + (num % 26) as u8) as char).to_string() + &result;
            num /= 26;
        }
        result
    }

    pub fn add_image<P: AsRef<Path>>(
        &mut self,
        image_path: P,
        alt_text: Option<&str>,
    ) -> Result<String> {
        use std::io::Read;

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

        self.add_image_from_bytes(&buffer, &content_type, alt_text)
    }

    pub fn add_image_from_bytes(
        &mut self,
        image_data: &[u8],
        content_type: &str,
        alt_text: Option<&str>,
    ) -> Result<String> {
        let image_id = format!("image{}", self.images.len() + 1);

        // Use image crate to get dimensions
        let (width, height) = match image::load_from_memory(image_data) {
            Ok(img) => (img.width(), img.height()),
            Err(_) => (100, 100), // default dimensions
        };

        let image_info = ImageInfo {
            id: image_id.clone(),
            data: image_data.to_vec(),
            content_type: content_type.to_string(),
            width,
            height,
            alt_text: alt_text.unwrap_or("").to_string(),
        };

        self.images.insert(image_id.clone(), image_info);

        Ok(image_id)
    }

    pub fn add_image_to_worksheet(
        &mut self,
        _worksheet_name: &str,
        image_id: &str,
        _col: u32,
        _row: u32,
        _width_cells: Option<u32>,
        _height_cells: Option<u32>,
    ) -> Result<()> {
        if !self.images.contains_key(image_id) {
            return Err(OfficeError::MissingFile(format!(
                "Image not found: {}",
                image_id
            )));
        }

        // For now, we'll store the image reference in the images HashMap
        // A full implementation would modify the worksheet XML to include drawing relationships
        // and create the necessary xl/drawings/ and xl/media/ entries in the ZIP

        Ok(())
    }

    fn default_workbook_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
    <sheets>
        <sheet name="Sheet1" sheetId="1" r:id="rId1" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>
    </sheets>
</workbook>"#.to_string()
    }

    fn default_worksheet_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
    <sheetData>
    </sheetData>
</worksheet>"#
            .to_string()
    }

    fn default_shared_strings_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
</sst>"#
            .to_string()
    }

    fn default_styles_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
    <numFmts count="0"/>
    <fonts count="1">
        <font>
            <sz val="11"/>
            <name val="Calibri"/>
        </font>
    </fonts>
    <fills count="1">
        <fill>
            <patternFill patternType="none"/>
        </fill>
    </fills>
    <borders count="1">
        <border>
            <left/>
            <right/>
            <top/>
            <bottom/>
            <diagonal/>
        </border>
    </borders>
    <cellStyleXfs count="1">
        <xf numFmtId="0" fontId="0" fillId="0" borderId="0"/>
    </cellStyleXfs>
    <cellXfs count="1">
        <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
    </cellXfs>
</styleSheet>"#
            .to_string()
    }

    fn default_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
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

    fn default_content_types_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
    <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
    <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
    <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
    <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
    <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#.to_string()
    }

    fn workbook_relationships_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
    <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>"#.to_string()
    }
}

impl OfficeDocument for XlsxDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;

        validate_office_format(&mut archive, &["xl/workbook.xml", "[Content_Types].xml"])?;

        let workbook = read_zip_entry(&mut archive, "xl/workbook.xml")?;
        let shared_strings = read_zip_entry(&mut archive, "xl/sharedStrings.xml").ok();
        let styles = read_zip_entry(&mut archive, "xl/styles.xml").ok();
        let relationships = read_zip_entry(&mut archive, "_rels/.rels").ok();
        let app_properties = read_zip_entry(&mut archive, "docProps/app.xml").ok();
        let core_properties = read_zip_entry(&mut archive, "docProps/core.xml").ok();
        let vba_project = match read_zip_entry_bytes(&mut archive, "xl/vbaProject.bin") {
            Ok(bytes) => Some(bytes),
            Err(OfficeError::MissingFile(_)) => None,
            Err(error) => return Err(error),
        };

        let mut worksheets = HashMap::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name()?.into_owned();

            if name.starts_with("xl/worksheets/") && name.ends_with(".xml") {
                drop(file);
                let content = read_zip_entry(&mut archive, &name)?;
                let sheet_name = name.replace("xl/worksheets/", "").replace(".xml", "");
                worksheets.insert(sheet_name, content);
            }
        }

        if worksheets.is_empty() {
            worksheets.insert("sheet1".to_string(), Self::default_worksheet_xml());
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
                    "New Spreadsheet".to_string(),
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
                "New Spreadsheet".to_string(),
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

        Ok(XlsxDocument {
            workbook,
            worksheets,
            shared_strings,
            styles,
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
            page_setup: PageSetup::default(),
            images: HashMap::new(),
            charts: HashMap::new(),
            vba_project,
            tables: HashMap::new(),
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);

            zip.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
            let mut content_types = Self::default_content_types_xml();
            if self.has_vba_macros() {
                content_types = content_types.replace(
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml",
                    "application/vnd.ms-excel.sheet.macroEnabled.main+xml",
                ).replace("</Types>", "<Override PartName=\"/xl/vbaProject.bin\" ContentType=\"application/vnd.ms-office.vbaProject\"/></Types>");
            }
            zip.write_all(content_types.as_bytes())?;

            zip.start_file("_rels/.rels", SimpleFileOptions::default())?;
            zip.write_all(
                self.relationships
                    .as_ref()
                    .unwrap_or(&Self::default_relationships_xml())
                    .as_bytes(),
            )?;

            zip.start_file("xl/workbook.xml", SimpleFileOptions::default())?;
            zip.write_all(self.workbook.as_bytes())?;

            zip.start_file("xl/_rels/workbook.xml.rels", SimpleFileOptions::default())?;
            let mut workbook_relationships = Self::workbook_relationships_xml();
            if let Some(vba) = &self.vba_project {
                workbook_relationships = workbook_relationships.replace("</Relationships>", "<Relationship Id=\"rIdVba\" Type=\"http://schemas.microsoft.com/office/2006/relationships/vbaProject\" Target=\"vbaProject.bin\"/></Relationships>");
                zip.write_all(workbook_relationships.as_bytes())?;
                zip.start_file("xl/vbaProject.bin", SimpleFileOptions::default())?;
                zip.write_all(vba)?;
            } else {
                zip.write_all(workbook_relationships.as_bytes())?;
            }

            for (sheet_name, worksheet_content) in &self.worksheets {
                let path = format!("xl/worksheets/{}.xml", sheet_name);
                zip.start_file(&path, SimpleFileOptions::default())?;
                zip.write_all(worksheet_content.as_bytes())?;
            }

            if let Some(shared_strings) = &self.shared_strings {
                zip.start_file("xl/sharedStrings.xml", SimpleFileOptions::default())?;
                zip.write_all(shared_strings.as_bytes())?;
            }

            if let Some(styles) = &self.styles {
                zip.start_file("xl/styles.xml", SimpleFileOptions::default())?;
                zip.write_all(styles.as_bytes())?;
            }

            if let Some(app) = &self.app_properties {
                zip.start_file("docProps/app.xml", SimpleFileOptions::default())?;
                zip.write_all(app.as_bytes())?;
            }

            zip.start_file("docProps/core.xml", SimpleFileOptions::default())?;
            zip.write_all(self.create_core_properties().as_bytes())?;

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

impl XlsxDocument {
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
        let mut title = "New Spreadsheet".to_string();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worksheet_cells_preserve_xml_references() {
        let mut document = XlsxDocument::new();
        document.worksheets.insert(
            "entities".to_string(),
            "<worksheet><sheetData><row r=\"1\"><c><v>Fish &amp; Chips &#33;</v></c></row></sheetData></worksheet>"
                .to_string(),
        );

        assert_eq!(
            document.get_worksheet_data("entities").unwrap(),
            vec![vec!["Fish & Chips !".to_string()]]
        );
    }

    #[test]
    fn core_properties_use_local_names_and_preserve_references() {
        let properties = XlsxDocument::parse_core_properties(
            "<cp:coreProperties><dc:creator>A &amp; B</dc:creator><dc:title>Title &#38; More</dc:title></cp:coreProperties>",
        )
        .unwrap();

        assert_eq!(properties.0, "A & B");
        assert_eq!(properties.1, "Title & More");
    }
}

#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub worksheet_count: usize,
    pub image_count: usize,
    pub chart_count: usize,
    pub table_count: usize,
    pub total_cells: usize,
    pub has_vba: bool,
    pub author: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}
