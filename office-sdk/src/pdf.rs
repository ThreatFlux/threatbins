use crate::OfficeDocument;
use crate::error::{OfficeError, Result};
use ::image::ImageReader;
use ::lopdf::Document as LoDocument;
use chrono::{DateTime, Utc};
use printpdf::*;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use uuid::Uuid;

/// Comprehensive PDF document with advanced features matching other Office formats
#[derive(Debug, Clone)]
pub struct PdfDocument {
    // Core PDF structure
    pub pages: Vec<PdfPageInfo>,
    pub fonts: HashMap<String, FontInfo>,
    pub images: HashMap<String, ImageInfo>,
    pub bookmarks: Vec<Bookmark>,
    pub annotations: HashMap<String, Annotation>,
    pub forms: HashMap<String, FormField>,

    // Enhanced metadata (matching DOCX/XLSX/PPTX)
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

    // PDF-specific properties
    pub pdf_version: PdfVersion,
    pub page_layout: PageLayout,
    pub page_mode: PageMode,
    pub security_settings: SecuritySettings,
    pub javascript: HashMap<String, String>,
    pub attachments: HashMap<String, AttachmentInfo>,

    // Document structure and advanced features
    pub outline: Vec<OutlineItem>,
    pub layers: HashMap<String, LayerInfo>,
    pub color_profiles: HashMap<String, ColorProfile>,
    pub viewer_preferences: ViewerPreferences,

    // Internal document state
    #[allow(dead_code)]
    document_id: String,
}

#[derive(Debug, Clone)]
pub struct PdfPageInfo {
    pub id: String,
    pub width: f64,
    pub height: f64,
    pub rotation: PageRotation,
    pub content: Vec<PageContent>,
    pub annotations: Vec<String>, // Reference to annotation IDs
    pub media_box: Rectangle,
    pub crop_box: Option<Rectangle>,
    pub bleed_box: Option<Rectangle>,
    pub trim_box: Option<Rectangle>,
    pub art_box: Option<Rectangle>,
    pub transparency_group: Option<TransparencyGroup>,
}

#[derive(Debug, Clone)]
pub enum PageContent {
    Text {
        content: String,
        x: f64,
        y: f64,
        font: String,
        size: f64,
        color: Color,
        style: TextStyle,
        transformation: Option<TextTransformation>,
    },
    Image {
        image_id: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        rotation: f64,
        alpha: f64,
    },
    Shape {
        shape_type: ShapeType,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f64,
        style: ShapeStyle,
    },
    Line {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        color: Color,
        width: f64,
        style: LineStyle,
        cap_style: LineCapStyle,
        join_style: LineJoinStyle,
    },
    Path {
        commands: Vec<PathCommand>,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f64,
        fill_rule: FillRule,
    },
    Table {
        table_id: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        rows: Vec<TableRow>,
        style: TableStyle,
    },
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub height: f64,
    pub background_color: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: String,
    pub width: f64,
    pub alignment: TextAlignment,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f64,
    pub padding: Padding,
}

#[derive(Debug, Clone)]
pub struct Padding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Debug, Clone)]
pub struct TableStyle {
    pub border_width: f64,
    pub border_color: Color,
    pub header_background: Option<Color>,
    pub alternate_row_color: Option<Color>,
}

#[derive(Debug, Clone)]
pub enum PathCommand {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
    },
    QuadCurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    ClosePath,
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
    V1_4,
    V1_5,
    V1_6,
    V1_7,
    V2_0,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageLayout {
    SinglePage,
    OneColumn,
    TwoColumnLeft,
    TwoColumnRight,
    TwoPageLeft,
    TwoPageRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageMode {
    UseNone,
    UseOutlines,
    UseThumbs,
    FullScreen,
    UseOC,
    UseAttachments,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageRotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

#[derive(Debug, Clone)]
pub struct SecuritySettings {
    pub encryption_enabled: bool,
    pub user_password: Option<String>,
    pub owner_password: Option<String>,
    pub permissions: PermissionFlags,
    pub encryption_level: EncryptionLevel,
    pub metadata_encrypted: bool,
    pub attachments_encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct PermissionFlags {
    pub allow_printing: bool,
    pub allow_modify_contents: bool,
    pub allow_copy: bool,
    pub allow_modify_annotations: bool,
    pub allow_fill_forms: bool,
    pub allow_extract_for_accessibility: bool,
    pub allow_assemble: bool,
    pub allow_print_high_quality: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionLevel {
    Rc4_40Bit,
    Rc4_128Bit,
    Aes128Bit,
    Aes256Bit,
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    pub id: String,
    pub name: String,
    pub font_type: FontType,
    pub encoding: String,
    pub is_embedded: bool,
    pub font_data: Option<Vec<u8>>,
    pub font_size: f64,
    pub is_subset: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontType {
    Type1,
    TrueType,
    Type0,
    Type3,
    CIDFont,
    OpenType,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub id: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub width: u32,
    pub height: u32,
    pub color_space: ColorSpace,
    pub bits_per_component: u8,
    pub alt_text: String,
    pub dpi: u32,
    pub compression: ImageCompression,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    DeviceGray,
    DeviceRGB,
    DeviceCMYK,
    ICCBased,
    Indexed,
    Separation,
    DeviceN,
    Lab,
    CalGray,
    CalRGB,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageCompression {
    None,
    JPEG,
    JPEG2000,
    Flate,
    LZW,
    CCITTFax,
}

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub children: Vec<String>, // Child bookmark IDs
    pub is_open: bool,
    pub color: Option<Color>,
    pub style: BookmarkStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BookmarkStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub id: String,
    pub annotation_type: AnnotationType,
    pub page: usize,
    pub rect: Rectangle,
    pub content: String,
    pub author: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub color: Option<Color>,
    pub flags: AnnotationFlags,
    pub border: Option<AnnotationBorder>,
    pub popup: Option<String>, // Reference to popup annotation
}

#[derive(Debug, Clone)]
pub enum AnnotationType {
    Text,
    Link,
    FreeText,
    Line,
    Square,
    Circle,
    Polygon,
    PolyLine,
    Highlight,
    Underline,
    Squiggly,
    StrikeOut,
    Stamp,
    Caret,
    Ink,
    Popup,
    FileAttachment,
    Sound,
    Movie,
    Widget,
    Screen,
    PrinterMark,
    TrapNet,
    Watermark,
    ThreeD,
    Redact,
    RichMedia,
}

#[derive(Debug, Clone)]
pub struct AnnotationFlags {
    pub invisible: bool,
    pub hidden: bool,
    pub print: bool,
    pub no_zoom: bool,
    pub no_rotate: bool,
    pub no_view: bool,
    pub read_only: bool,
    pub locked: bool,
    pub toggle_no_view: bool,
    pub locked_contents: bool,
}

#[derive(Debug, Clone)]
pub struct AnnotationBorder {
    pub width: f64,
    pub style: BorderStyle,
    pub dash_pattern: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Beveled,
    Inset,
    Underline,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub id: String,
    pub field_type: FormFieldType,
    pub name: String,
    pub value: String,
    pub page: usize,
    pub rect: Rectangle,
    pub flags: FormFieldFlags,
    pub options: Vec<String>, // For choice fields
    pub default_value: Option<String>,
    pub max_length: Option<u32>,
    pub validation_script: Option<String>,
    pub calculation_script: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FormFieldType {
    Text {
        multiline: bool,
        password: bool,
        rich_text: bool,
    },
    Button {
        button_type: ButtonType,
    },
    Choice {
        choice_type: ChoiceType,
        options: Vec<ChoiceOption>,
    },
    Signature,
    Barcode {
        barcode_type: BarcodeType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType {
    PushButton,
    CheckBox,
    RadioButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChoiceType {
    ComboBox,
    ListBox,
}

#[derive(Debug, Clone)]
pub struct ChoiceOption {
    pub display_value: String,
    pub export_value: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarcodeType {
    Code128,
    Code39,
    QRCode,
    DataMatrix,
    PDF417,
}

#[derive(Debug, Clone)]
pub struct FormFieldFlags {
    pub read_only: bool,
    pub required: bool,
    pub no_export: bool,
    pub multiline: bool,
    pub password: bool,
    pub radio: bool,
    pub pushbutton: bool,
    pub combo: bool,
    pub edit: bool,
    pub sort: bool,
    pub file_select: bool,
    pub multiselect: bool,
    pub do_not_spell_check: bool,
    pub do_not_scroll: bool,
    pub comb: bool,
    pub rich_text: bool,
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64, // Alpha
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub superscript: bool,
    pub subscript: bool,
}

#[derive(Debug, Clone)]
pub struct TextTransformation {
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotate: f64,
    pub skew_x: f64,
    pub skew_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Ellipse,
    Triangle,
    Polygon,
    RoundedRectangle { radius: f64 },
}

#[derive(Debug, Clone)]
pub struct ShapeStyle {
    pub fill_opacity: f64,
    pub stroke_opacity: f64,
    pub blend_mode: BlendMode,
    pub shadow: Option<Shadow>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Darken,
    Lighten,
    Difference,
    Exclusion,
}

#[derive(Debug, Clone)]
pub struct Shadow {
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur_radius: f64,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    DashDotDot,
    Custom { pattern: [f64; 8], count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineCapStyle {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineJoinStyle {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone)]
pub struct AttachmentInfo {
    pub id: String,
    pub filename: String,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub description: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub checksum: Option<String>,
    pub compressed: bool,
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    pub id: String,
    pub title: String,
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub children: Vec<OutlineItem>,
    pub is_open: bool,
    pub color: Option<Color>,
    pub style: OutlineStyle,
    pub action: Option<OutlineAction>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutlineStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Debug, Clone)]
pub enum OutlineAction {
    GoTo {
        page: usize,
        x: f64,
        y: f64,
    },
    GoToR {
        file: String,
        page: usize,
    },
    Launch {
        file: String,
        parameters: Option<String>,
    },
    URI {
        uri: String,
    },
    JavaScript {
        script: String,
    },
}

#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub printable: bool,
    pub locked: bool,
    pub zoom_range: Option<(f64, f64)>,
    pub intent: LayerIntent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerIntent {
    View,
    Design,
    All,
}

#[derive(Debug, Clone)]
pub struct ColorProfile {
    pub id: String,
    pub name: String,
    pub profile_data: Vec<u8>,
    pub profile_type: ColorProfileType,
    pub description: String,
    pub intent: RenderingIntent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorProfileType {
    Input,
    Display,
    Output,
    ColorSpace,
    Abstract,
    NamedColor,
    DeviceLink,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

#[derive(Debug, Clone)]
pub struct ViewerPreferences {
    pub hide_toolbar: bool,
    pub hide_menubar: bool,
    pub hide_window_ui: bool,
    pub fit_window: bool,
    pub center_window: bool,
    pub display_doc_title: bool,
    pub non_full_screen_page_mode: PageMode,
    pub direction: ReadingDirection,
    pub view_area: ViewArea,
    pub view_clip: ViewArea,
    pub print_area: ViewArea,
    pub print_clip: ViewArea,
    pub print_scaling: PrintScaling,
    pub duplex: DuplexMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadingDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewArea {
    MediaBox,
    CropBox,
    BleedBox,
    TrimBox,
    ArtBox,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrintScaling {
    None,
    AppDefault,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DuplexMode {
    Simplex,
    DuplexFlipShortEdge,
    DuplexFlipLongEdge,
}

#[derive(Debug, Clone)]
pub struct TransparencyGroup {
    pub isolated: bool,
    pub knockout: bool,
    pub color_space: Option<ColorSpace>,
}

#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub page_count: usize,
    pub image_count: usize,
    pub annotation_count: usize,
    pub form_field_count: usize,
    pub bookmark_count: usize,
    pub attachment_count: usize,
    pub font_count: usize,
    pub layer_count: usize,
    pub javascript_count: usize,
    pub total_text_length: usize,
    pub has_security: bool,
    pub has_javascript: bool,
    pub has_forms: bool,
    pub has_annotations: bool,
    pub pdf_version: PdfVersion,
    pub file_size: usize,
    pub author: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub is_linearized: bool,
    pub is_tagged: bool,
}

// Implementation section
impl Default for PdfDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        SecuritySettings {
            encryption_enabled: false,
            user_password: None,
            owner_password: None,
            permissions: PermissionFlags::default(),
            encryption_level: EncryptionLevel::Aes128Bit,
            metadata_encrypted: true,
            attachments_encrypted: true,
        }
    }
}

impl Default for PermissionFlags {
    fn default() -> Self {
        PermissionFlags {
            allow_printing: true,
            allow_modify_contents: true,
            allow_copy: true,
            allow_modify_annotations: true,
            allow_fill_forms: true,
            allow_extract_for_accessibility: true,
            allow_assemble: true,
            allow_print_high_quality: true,
        }
    }
}

impl Default for AnnotationFlags {
    fn default() -> Self {
        AnnotationFlags {
            invisible: false,
            hidden: false,
            print: true,
            no_zoom: false,
            no_rotate: false,
            no_view: false,
            read_only: false,
            locked: false,
            toggle_no_view: false,
            locked_contents: false,
        }
    }
}

impl Default for FormFieldFlags {
    fn default() -> Self {
        FormFieldFlags {
            read_only: false,
            required: false,
            no_export: false,
            multiline: false,
            password: false,
            radio: false,
            pushbutton: false,
            combo: false,
            edit: false,
            sort: false,
            file_select: false,
            multiselect: false,
            do_not_spell_check: false,
            do_not_scroll: false,
            comb: false,
            rich_text: false,
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            superscript: false,
            subscript: false,
        }
    }
}

impl Default for ShapeStyle {
    fn default() -> Self {
        ShapeStyle {
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
            blend_mode: BlendMode::Normal,
            shadow: None,
        }
    }
}

impl Default for ViewerPreferences {
    fn default() -> Self {
        ViewerPreferences {
            hide_toolbar: false,
            hide_menubar: false,
            hide_window_ui: false,
            fit_window: false,
            center_window: false,
            display_doc_title: false,
            non_full_screen_page_mode: PageMode::UseNone,
            direction: ReadingDirection::LeftToRight,
            view_area: ViewArea::CropBox,
            view_clip: ViewArea::CropBox,
            print_area: ViewArea::CropBox,
            print_clip: ViewArea::CropBox,
            print_scaling: PrintScaling::AppDefault,
            duplex: DuplexMode::Simplex,
        }
    }
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(OfficeError::InvalidFormat(
                "Invalid hex color format".to_string(),
            ));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| OfficeError::InvalidFormat("Invalid hex color".to_string()))?
            as f64
            / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| OfficeError::InvalidFormat("Invalid hex color".to_string()))?
            as f64
            / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| OfficeError::InvalidFormat("Invalid hex color".to_string()))?
            as f64
            / 255.0;

        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| OfficeError::InvalidFormat("Invalid hex color".to_string()))?
                as f64
                / 255.0
        } else {
            1.0
        };

        Ok(Color { r, g, b, a })
    }

    // Predefined colors
    pub fn black() -> Self {
        Color::rgb(0.0, 0.0, 0.0)
    }
    pub fn white() -> Self {
        Color::rgb(1.0, 1.0, 1.0)
    }
    pub fn red() -> Self {
        Color::rgb(1.0, 0.0, 0.0)
    }
    pub fn green() -> Self {
        Color::rgb(0.0, 1.0, 0.0)
    }
    pub fn blue() -> Self {
        Color::rgb(0.0, 0.0, 1.0)
    }
    pub fn yellow() -> Self {
        Color::rgb(1.0, 1.0, 0.0)
    }
    pub fn cyan() -> Self {
        Color::rgb(0.0, 1.0, 1.0)
    }
    pub fn magenta() -> Self {
        Color::rgb(1.0, 0.0, 1.0)
    }
    pub fn gray(value: f64) -> Self {
        Color::rgb(value, value, value)
    }
    pub fn transparent() -> Self {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    }

    pub fn to_printpdf_color(&self) -> printpdf::Color {
        printpdf::Color::Rgb(Rgb::new(self.r as f32, self.g as f32, self.b as f32, None))
    }
}

impl Rectangle {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_corners(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Rectangle {
            x: x1.min(x2),
            y: y1.min(y2),
            width: (x2 - x1).abs(),
            height: (y2 - y1).abs(),
        }
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(self.x + self.width < other.x
            || other.x + other.width < self.x
            || self.y + self.height < other.y
            || other.y + other.height < self.y)
    }
}

impl PdfDocument {
    pub fn new() -> Self {
        let now = Utc::now();
        PdfDocument {
            pages: Vec::new(),
            fonts: HashMap::new(),
            images: HashMap::new(),
            bookmarks: Vec::new(),
            annotations: HashMap::new(),
            forms: HashMap::new(),

            // Enhanced metadata matching other Office formats
            author: "Office SDK".to_string(),
            title: "New PDF Document".to_string(),
            subject: "".to_string(),
            keywords: "".to_string(),
            description: "".to_string(),
            category: "".to_string(),
            created: now,
            modified: now,
            last_modified_by: "Office SDK".to_string(),
            revision: 1,

            // PDF-specific properties
            pdf_version: PdfVersion::V1_7,
            page_layout: PageLayout::SinglePage,
            page_mode: PageMode::UseNone,
            security_settings: SecuritySettings::default(),
            javascript: HashMap::new(),
            attachments: HashMap::new(),

            // Advanced document structure
            outline: Vec::new(),
            layers: HashMap::new(),
            color_profiles: HashMap::new(),
            viewer_preferences: ViewerPreferences::default(),

            document_id: Uuid::new_v4().to_string(),
        }
    }

    // Enhanced metadata methods (matching DOCX/XLSX/PPTX)
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
        self.modified = Utc::now();
        self.revision += 1;
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.modified = Utc::now();
        self.revision += 1;
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

    // PDF-specific configuration methods
    pub fn set_pdf_version(&mut self, version: PdfVersion) {
        self.pdf_version = version;
        self.modified = Utc::now();
    }

    pub fn set_page_layout(&mut self, layout: PageLayout) {
        self.page_layout = layout;
        self.modified = Utc::now();
    }

    pub fn set_page_mode(&mut self, mode: PageMode) {
        self.page_mode = mode;
        self.modified = Utc::now();
    }

    pub fn set_viewer_preferences(&mut self, preferences: ViewerPreferences) {
        self.viewer_preferences = preferences;
        self.modified = Utc::now();
    }

    // Page management methods
    pub fn add_page(&mut self, width: f64, height: f64) -> Result<String> {
        let page_id = format!("page{}", self.pages.len() + 1);
        let page = PdfPageInfo {
            id: page_id.clone(),
            width,
            height,
            rotation: PageRotation::None,
            content: Vec::new(),
            annotations: Vec::new(),
            media_box: Rectangle::new(0.0, 0.0, width, height),
            crop_box: None,
            bleed_box: None,
            trim_box: None,
            art_box: None,
            transparency_group: None,
        };

        self.pages.push(page);
        self.modified = Utc::now();
        Ok(page_id)
    }

    pub fn add_a4_page(&mut self) -> Result<String> {
        // A4 dimensions in points (595.28 × 841.89 points)
        self.add_page(595.28, 841.89)
    }

    pub fn add_letter_page(&mut self) -> Result<String> {
        // US Letter dimensions in points (612 × 792 points)
        self.add_page(612.0, 792.0)
    }

    pub fn add_legal_page(&mut self) -> Result<String> {
        // US Legal dimensions in points (612 × 1008 points)
        self.add_page(612.0, 1008.0)
    }

    pub fn add_tabloid_page(&mut self) -> Result<String> {
        // Tabloid dimensions in points (792 × 1224 points)
        self.add_page(792.0, 1224.0)
    }

    // Content creation methods
    pub fn add_text(
        &mut self,
        page_index: usize,
        text: &str,
        x: f64,
        y: f64,
        font: &str,
        size: f64,
    ) -> Result<()> {
        self.add_styled_text(
            page_index,
            text,
            x,
            y,
            font,
            size,
            Color::black(),
            TextStyle::default(),
        )
    }

    pub fn add_styled_text(
        &mut self,
        page_index: usize,
        text: &str,
        x: f64,
        y: f64,
        font: &str,
        size: f64,
        color: Color,
        style: TextStyle,
    ) -> Result<()> {
        if page_index >= self.pages.len() {
            return Err(OfficeError::InvalidFormat(
                "Page index out of bounds".to_string(),
            ));
        }

        let content = PageContent::Text {
            content: text.to_string(),
            x,
            y,
            font: font.to_string(),
            size,
            color,
            style,
            transformation: None,
        };

        self.pages[page_index].content.push(content);
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_heading(
        &mut self,
        page_index: usize,
        level: u32,
        text: &str,
        x: f64,
        y: f64,
    ) -> Result<()> {
        let font_size = match level {
            1 => 24.0,
            2 => 20.0,
            3 => 16.0,
            4 => 14.0,
            5 => 12.0,
            _ => 10.0,
        };

        let style = TextStyle {
            bold: true,
            italic: false,
            underline: false,
            strikethrough: false,
            superscript: false,
            subscript: false,
        };

        self.add_styled_text(
            page_index,
            text,
            x,
            y,
            "Helvetica-Bold",
            font_size,
            Color::black(),
            style,
        )
    }

    pub fn add_paragraph(
        &mut self,
        page_index: usize,
        text: &str,
        x: f64,
        y: f64,
        width: f64,
        font_size: f64,
    ) -> Result<()> {
        // Simple text wrapping implementation
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        let mut current_y = y;
        let line_height = font_size * 1.2;

        let approx_char_width = font_size * 0.6; // Rough approximation
        let max_chars_per_line = (width / approx_char_width) as usize;

        for word in words {
            if current_line.len() + word.len() + 1 > max_chars_per_line {
                if !current_line.is_empty() {
                    self.add_text(
                        page_index,
                        &current_line,
                        x,
                        current_y,
                        "Helvetica",
                        font_size,
                    )?;
                    current_y -= line_height;
                    current_line.clear();
                }
            }

            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }

        if !current_line.is_empty() {
            self.add_text(
                page_index,
                &current_line,
                x,
                current_y,
                "Helvetica",
                font_size,
            )?;
        }

        Ok(())
    }

    // Image methods (matching other Office formats)
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
            "tiff" | "tif" => "image/tiff",
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

        let color_space = match content_type {
            "image/png" | "image/jpeg" | "image/tiff" => ColorSpace::DeviceRGB,
            "image/gif" => ColorSpace::Indexed,
            "image/bmp" => ColorSpace::DeviceRGB,
            _ => ColorSpace::DeviceRGB,
        };

        let compression = match content_type {
            "image/jpeg" => ImageCompression::JPEG,
            "image/png" => ImageCompression::Flate,
            "image/tiff" => ImageCompression::LZW,
            _ => ImageCompression::Flate,
        };

        let image_info = ImageInfo {
            id: image_id.clone(),
            data: image_data.to_vec(),
            content_type: content_type.to_string(),
            width: dimensions.0,
            height: dimensions.1,
            color_space,
            bits_per_component: 8,
            alt_text: alt_text.unwrap_or("Image").to_string(),
            dpi: 300, // Default high-quality DPI
            compression,
        };

        self.images.insert(image_id.clone(), image_info);
        self.modified = Utc::now();
        Ok(image_id)
    }

    pub fn add_image_to_page(
        &mut self,
        page_index: usize,
        image_id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<()> {
        if page_index >= self.pages.len() {
            return Err(OfficeError::InvalidFormat(
                "Page index out of bounds".to_string(),
            ));
        }

        if !self.images.contains_key(image_id) {
            return Err(OfficeError::InvalidFormat("Image not found".to_string()));
        }

        let content = PageContent::Image {
            image_id: image_id.to_string(),
            x,
            y,
            width,
            height,
            rotation: 0.0,
            alpha: 1.0,
        };

        self.pages[page_index].content.push(content);
        self.modified = Utc::now();
        Ok(())
    }

    // Shape and drawing methods
    pub fn add_rectangle(
        &mut self,
        page_index: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
    ) -> Result<()> {
        if page_index >= self.pages.len() {
            return Err(OfficeError::InvalidFormat(
                "Page index out of bounds".to_string(),
            ));
        }

        let content = PageContent::Shape {
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width,
            height,
            fill_color,
            stroke_color,
            stroke_width: 1.0,
            style: ShapeStyle::default(),
        };

        self.pages[page_index].content.push(content);
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_circle(
        &mut self,
        page_index: usize,
        x: f64,
        y: f64,
        radius: f64,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
    ) -> Result<()> {
        if page_index >= self.pages.len() {
            return Err(OfficeError::InvalidFormat(
                "Page index out of bounds".to_string(),
            ));
        }

        let content = PageContent::Shape {
            shape_type: ShapeType::Circle,
            x: x - radius,
            y: y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
            fill_color,
            stroke_color,
            stroke_width: 1.0,
            style: ShapeStyle::default(),
        };

        self.pages[page_index].content.push(content);
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_line(
        &mut self,
        page_index: usize,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        color: Color,
        width: f64,
    ) -> Result<()> {
        if page_index >= self.pages.len() {
            return Err(OfficeError::InvalidFormat(
                "Page index out of bounds".to_string(),
            ));
        }

        let content = PageContent::Line {
            start_x,
            start_y,
            end_x,
            end_y,
            color,
            width,
            style: LineStyle::Solid,
            cap_style: LineCapStyle::Butt,
            join_style: LineJoinStyle::Miter,
        };

        self.pages[page_index].content.push(content);
        self.modified = Utc::now();
        Ok(())
    }

    // Table methods (matching other Office formats)
    pub fn add_table(
        &mut self,
        page_index: usize,
        x: f64,
        y: f64,
        headers: &[&str],
        data: &[Vec<&str>],
    ) -> Result<String> {
        if page_index >= self.pages.len() {
            return Err(OfficeError::InvalidFormat(
                "Page index out of bounds".to_string(),
            ));
        }

        let table_id = format!("table{}", self.pages[page_index].content.len() + 1);
        let col_width = 100.0; // Default column width
        let row_height = 20.0; // Default row height
        let table_width = col_width * headers.len() as f64;
        let table_height = row_height * (data.len() as f64 + 1.0); // +1 for header

        let mut rows = Vec::new();

        // Add header row
        let header_cells: Vec<TableCell> = headers
            .iter()
            .map(|header| TableCell {
                content: header.to_string(),
                width: col_width,
                alignment: TextAlignment::Center,
                background_color: Some(Color::gray(0.9)),
                border_color: Some(Color::black()),
                border_width: 1.0,
                padding: Padding {
                    top: 4.0,
                    right: 4.0,
                    bottom: 4.0,
                    left: 4.0,
                },
            })
            .collect();

        rows.push(TableRow {
            cells: header_cells,
            height: row_height,
            background_color: Some(Color::gray(0.9)),
        });

        // Add data rows
        for row_data in data {
            let cells: Vec<TableCell> = row_data
                .iter()
                .map(|cell_data| TableCell {
                    content: cell_data.to_string(),
                    width: col_width,
                    alignment: TextAlignment::Left,
                    background_color: None,
                    border_color: Some(Color::gray(0.5)),
                    border_width: 0.5,
                    padding: Padding {
                        top: 4.0,
                        right: 4.0,
                        bottom: 4.0,
                        left: 4.0,
                    },
                })
                .collect();

            rows.push(TableRow {
                cells,
                height: row_height,
                background_color: None,
            });
        }

        let content = PageContent::Table {
            table_id: table_id.clone(),
            x,
            y,
            width: table_width,
            height: table_height,
            rows,
            style: TableStyle {
                border_width: 1.0,
                border_color: Color::black(),
                header_background: Some(Color::gray(0.9)),
                alternate_row_color: Some(Color::gray(0.95)),
            },
        };

        self.pages[page_index].content.push(content);
        self.modified = Utc::now();
        Ok(table_id)
    }

    // Font management methods
    pub fn add_font(&mut self, name: &str, font_type: FontType) -> Result<String> {
        let font_id = format!("font{}", self.fonts.len() + 1);
        let font_info = FontInfo {
            id: font_id.clone(),
            name: name.to_string(),
            font_type,
            encoding: "WinAnsiEncoding".to_string(),
            is_embedded: false,
            font_data: None,
            font_size: 12.0,
            is_subset: false,
        };

        self.fonts.insert(font_id.clone(), font_info);
        self.modified = Utc::now();
        Ok(font_id)
    }

    pub fn embed_font(&mut self, name: &str, font_data: Vec<u8>) -> Result<String> {
        let font_id = format!("font{}", self.fonts.len() + 1);
        let font_info = FontInfo {
            id: font_id.clone(),
            name: name.to_string(),
            font_type: FontType::TrueType,
            encoding: "Identity-H".to_string(),
            is_embedded: true,
            font_data: Some(font_data),
            font_size: 12.0,
            is_subset: true,
        };

        self.fonts.insert(font_id.clone(), font_info);
        self.modified = Utc::now();
        Ok(font_id)
    }

    // Bookmark methods
    pub fn add_bookmark(&mut self, title: &str, page: usize, x: f64, y: f64) -> Result<String> {
        let bookmark_id = format!("bookmark{}", self.bookmarks.len() + 1);
        let bookmark = Bookmark {
            id: bookmark_id.clone(),
            title: title.to_string(),
            page,
            x,
            y,
            children: Vec::new(),
            is_open: false,
            color: None,
            style: BookmarkStyle::Normal,
        };

        self.bookmarks.push(bookmark);
        self.modified = Utc::now();
        Ok(bookmark_id)
    }

    pub fn add_nested_bookmark(
        &mut self,
        parent_id: &str,
        title: &str,
        page: usize,
        x: f64,
        y: f64,
    ) -> Result<String> {
        let bookmark_id = self.add_bookmark(title, page, x, y)?;

        // Find parent and add child reference
        for bookmark in &mut self.bookmarks {
            if bookmark.id == parent_id {
                bookmark.children.push(bookmark_id.clone());
                break;
            }
        }

        Ok(bookmark_id)
    }

    // Annotation methods (comprehensive)
    pub fn add_text_annotation(
        &mut self,
        page: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        content: &str,
        author: &str,
    ) -> Result<String> {
        let annotation_id = format!("annotation{}", self.annotations.len() + 1);
        let now = Utc::now();

        let annotation = Annotation {
            id: annotation_id.clone(),
            annotation_type: AnnotationType::Text,
            page,
            rect: Rectangle::new(x, y, width, height),
            content: content.to_string(),
            author: author.to_string(),
            created: now,
            modified: now,
            color: Some(Color::yellow()),
            flags: AnnotationFlags::default(),
            border: Some(AnnotationBorder {
                width: 1.0,
                style: BorderStyle::Solid,
                dash_pattern: Vec::new(),
            }),
            popup: None,
        };

        self.annotations.insert(annotation_id.clone(), annotation);

        // Add annotation reference to the page
        if page < self.pages.len() {
            self.pages[page].annotations.push(annotation_id.clone());
        }

        self.modified = Utc::now();
        Ok(annotation_id)
    }

    pub fn add_highlight_annotation(
        &mut self,
        page: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        author: &str,
    ) -> Result<String> {
        let annotation_id = format!("annotation{}", self.annotations.len() + 1);
        let now = Utc::now();

        let annotation = Annotation {
            id: annotation_id.clone(),
            annotation_type: AnnotationType::Highlight,
            page,
            rect: Rectangle::new(x, y, width, height),
            content: "".to_string(),
            author: author.to_string(),
            created: now,
            modified: now,
            color: Some(Color::rgba(1.0, 1.0, 0.0, 0.3)), // Semi-transparent yellow
            flags: AnnotationFlags::default(),
            border: None,
            popup: None,
        };

        self.annotations.insert(annotation_id.clone(), annotation);

        if page < self.pages.len() {
            self.pages[page].annotations.push(annotation_id.clone());
        }

        self.modified = Utc::now();
        Ok(annotation_id)
    }

    pub fn add_link_annotation(
        &mut self,
        page: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        url: &str,
    ) -> Result<String> {
        let annotation_id = format!("annotation{}", self.annotations.len() + 1);
        let now = Utc::now();

        let annotation = Annotation {
            id: annotation_id.clone(),
            annotation_type: AnnotationType::Link,
            page,
            rect: Rectangle::new(x, y, width, height),
            content: url.to_string(),
            author: "System".to_string(),
            created: now,
            modified: now,
            color: Some(Color::blue()),
            flags: AnnotationFlags::default(),
            border: Some(AnnotationBorder {
                width: 0.0,
                style: BorderStyle::Solid,
                dash_pattern: Vec::new(),
            }),
            popup: None,
        };

        self.annotations.insert(annotation_id.clone(), annotation);

        if page < self.pages.len() {
            self.pages[page].annotations.push(annotation_id.clone());
        }

        self.modified = Utc::now();
        Ok(annotation_id)
    }

    // Form field methods (comprehensive interactive forms)
    pub fn add_text_field(
        &mut self,
        page: usize,
        name: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        default_value: &str,
    ) -> Result<String> {
        let field_id = format!("field{}", self.forms.len() + 1);

        let form_field = FormField {
            id: field_id.clone(),
            field_type: FormFieldType::Text {
                multiline: false,
                password: false,
                rich_text: false,
            },
            name: name.to_string(),
            value: default_value.to_string(),
            page,
            rect: Rectangle::new(x, y, width, height),
            flags: FormFieldFlags::default(),
            options: Vec::new(),
            default_value: Some(default_value.to_string()),
            max_length: None,
            validation_script: None,
            calculation_script: None,
        };

        self.forms.insert(field_id.clone(), form_field);
        self.modified = Utc::now();
        Ok(field_id)
    }

    pub fn add_multiline_text_field(
        &mut self,
        page: usize,
        name: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        default_value: &str,
    ) -> Result<String> {
        let field_id = format!("field{}", self.forms.len() + 1);

        let form_field = FormField {
            id: field_id.clone(),
            field_type: FormFieldType::Text {
                multiline: true,
                password: false,
                rich_text: false,
            },
            name: name.to_string(),
            value: default_value.to_string(),
            page,
            rect: Rectangle::new(x, y, width, height),
            flags: FormFieldFlags::default(),
            options: Vec::new(),
            default_value: Some(default_value.to_string()),
            max_length: None,
            validation_script: None,
            calculation_script: None,
        };

        self.forms.insert(field_id.clone(), form_field);
        self.modified = Utc::now();
        Ok(field_id)
    }

    pub fn add_checkbox(
        &mut self,
        page: usize,
        name: &str,
        x: f64,
        y: f64,
        size: f64,
        checked: bool,
    ) -> Result<String> {
        let field_id = format!("field{}", self.forms.len() + 1);

        let form_field = FormField {
            id: field_id.clone(),
            field_type: FormFieldType::Button {
                button_type: ButtonType::CheckBox,
            },
            name: name.to_string(),
            value: if checked { "Yes" } else { "Off" }.to_string(),
            page,
            rect: Rectangle::new(x, y, size, size),
            flags: FormFieldFlags::default(),
            options: vec!["Yes".to_string(), "Off".to_string()],
            default_value: Some(if checked { "Yes" } else { "Off" }.to_string()),
            max_length: None,
            validation_script: None,
            calculation_script: None,
        };

        self.forms.insert(field_id.clone(), form_field);
        self.modified = Utc::now();
        Ok(field_id)
    }

    pub fn add_dropdown(
        &mut self,
        page: usize,
        name: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        options: &[&str],
        selected_index: Option<usize>,
    ) -> Result<String> {
        let field_id = format!("field{}", self.forms.len() + 1);

        let choice_options: Vec<ChoiceOption> = options
            .iter()
            .enumerate()
            .map(|(i, opt)| ChoiceOption {
                display_value: opt.to_string(),
                export_value: opt.to_string(),
                selected: selected_index == Some(i),
            })
            .collect();

        let selected_value = selected_index
            .and_then(|i| options.get(i))
            .unwrap_or(&"")
            .to_string();

        let form_field = FormField {
            id: field_id.clone(),
            field_type: FormFieldType::Choice {
                choice_type: ChoiceType::ComboBox,
                options: choice_options,
            },
            name: name.to_string(),
            value: selected_value.clone(),
            page,
            rect: Rectangle::new(x, y, width, height),
            flags: FormFieldFlags::default(),
            options: options.iter().map(|s| s.to_string()).collect(),
            default_value: Some(selected_value),
            max_length: None,
            validation_script: None,
            calculation_script: None,
        };

        self.forms.insert(field_id.clone(), form_field);
        self.modified = Utc::now();
        Ok(field_id)
    }

    // Security methods (comprehensive PDF security)
    pub fn set_password_protection(
        &mut self,
        user_password: Option<&str>,
        owner_password: Option<&str>,
    ) {
        self.security_settings.encryption_enabled =
            user_password.is_some() || owner_password.is_some();
        self.security_settings.user_password = user_password.map(|s| s.to_string());
        self.security_settings.owner_password = owner_password.map(|s| s.to_string());
        self.modified = Utc::now();
    }

    pub fn set_permissions(&mut self, permissions: PermissionFlags) {
        self.security_settings.permissions = permissions;
        self.modified = Utc::now();
    }

    pub fn set_encryption_level(&mut self, level: EncryptionLevel) {
        self.security_settings.encryption_level = level;
        self.modified = Utc::now();
    }

    // JavaScript methods (PDF automation)
    pub fn add_javascript(&mut self, name: &str, script: &str) -> Result<String> {
        let script_id = format!("js{}", self.javascript.len() + 1);
        self.javascript.insert(name.to_string(), script.to_string());
        self.modified = Utc::now();
        Ok(script_id)
    }

    pub fn add_document_javascript(&mut self, script: &str) -> Result<()> {
        self.add_javascript("DocumentJS", script)?;
        Ok(())
    }

    // Attachment methods
    pub fn add_attachment(
        &mut self,
        filename: &str,
        data: Vec<u8>,
        mime_type: &str,
        description: &str,
    ) -> Result<String> {
        let attachment_id = format!("attachment{}", self.attachments.len() + 1);
        let now = Utc::now();

        // Calculate checksum
        let checksum = format!("{:x}", md5::compute(&data));

        let attachment = AttachmentInfo {
            id: attachment_id.clone(),
            filename: filename.to_string(),
            data: data.clone(),
            mime_type: mime_type.to_string(),
            description: description.to_string(),
            created: now,
            modified: now,
            checksum: Some(checksum),
            compressed: data.len() > 1024, // Compress large attachments
        };

        self.attachments.insert(attachment_id.clone(), attachment);
        self.modified = Utc::now();
        Ok(attachment_id)
    }

    // Layer methods (advanced PDF layers/OCG)
    pub fn add_layer(&mut self, name: &str, visible: bool) -> Result<String> {
        let layer_id = format!("layer{}", self.layers.len() + 1);

        let layer = LayerInfo {
            id: layer_id.clone(),
            name: name.to_string(),
            visible,
            printable: true,
            locked: false,
            zoom_range: None,
            intent: LayerIntent::View,
        };

        self.layers.insert(layer_id.clone(), layer);
        self.modified = Utc::now();
        Ok(layer_id)
    }

    // Outline methods (document structure)
    pub fn add_outline_item(&mut self, title: &str, page: usize, x: f64, y: f64) -> Result<String> {
        let outline_id = format!("outline{}", self.outline.len() + 1);
        let outline_item = OutlineItem {
            id: outline_id.clone(),
            title: title.to_string(),
            page,
            x,
            y,
            children: Vec::new(),
            is_open: false,
            color: None,
            style: OutlineStyle::Normal,
            action: Some(OutlineAction::GoTo { page, x, y }),
        };

        self.outline.push(outline_item);
        self.modified = Utc::now();
        Ok(outline_id)
    }

    // Analysis and statistics methods
    pub fn get_document_stats(&self) -> DocumentStats {
        let total_text_length = self
            .pages
            .iter()
            .flat_map(|page| &page.content)
            .filter_map(|content| {
                if let PageContent::Text { content, .. } = content {
                    Some(content.len())
                } else {
                    None
                }
            })
            .sum();

        DocumentStats {
            page_count: self.pages.len(),
            image_count: self.images.len(),
            annotation_count: self.annotations.len(),
            form_field_count: self.forms.len(),
            bookmark_count: self.bookmarks.len(),
            attachment_count: self.attachments.len(),
            font_count: self.fonts.len(),
            layer_count: self.layers.len(),
            javascript_count: self.javascript.len(),
            total_text_length,
            has_security: self.security_settings.encryption_enabled,
            has_javascript: !self.javascript.is_empty(),
            has_forms: !self.forms.is_empty(),
            has_annotations: !self.annotations.is_empty(),
            pdf_version: self.pdf_version,
            file_size: 0, // Will be calculated during to_bytes()
            author: self.author.clone(),
            created: self.created,
            modified: self.modified,
            is_linearized: false, // Our generated PDFs are not linearized by default
            is_tagged: false,     // Tagged PDFs require additional structure
        }
    }

    pub fn get_text(&self) -> Result<String> {
        let mut text = String::new();

        for (page_num, page) in self.pages.iter().enumerate() {
            text.push_str(&format!("Page {}:\n", page_num + 1));

            for content in &page.content {
                match content {
                    PageContent::Text {
                        content: text_content,
                        ..
                    } => {
                        text.push_str(text_content);
                        text.push('\n');
                    }
                    PageContent::Table { rows, .. } => {
                        for row in rows {
                            for cell in &row.cells {
                                text.push_str(&cell.content);
                                text.push('\t');
                            }
                            text.push('\n');
                        }
                    }
                    _ => {} // Other content types don't contribute to text
                }
            }
            text.push('\n');
        }

        Ok(text)
    }

    // Convert to simplified format for other document types
    pub fn to_simple_text(&self) -> String {
        let mut result = String::new();

        for page in &self.pages {
            for content in &page.content {
                match content {
                    PageContent::Text { content, .. } => {
                        result.push_str(content);
                        result.push(' ');
                    }
                    PageContent::Table { rows, .. } => {
                        for row in rows {
                            for cell in &row.cells {
                                result.push_str(&cell.content);
                                result.push(' ');
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        result.trim().to_string()
    }

    // Advanced document manipulation
    pub fn merge_pages(&mut self, other: &PdfDocument) -> Result<()> {
        for page in &other.pages {
            let mut new_page = page.clone();
            new_page.id = format!("page{}", self.pages.len() + 1);
            self.pages.push(new_page);
        }

        // Merge images
        for (id, image) in &other.images {
            if !self.images.contains_key(id) {
                self.images.insert(id.clone(), image.clone());
            }
        }

        self.modified = Utc::now();
        Ok(())
    }

    pub fn extract_pages(&self, page_indices: &[usize]) -> Result<PdfDocument> {
        let mut new_doc = PdfDocument::new();
        new_doc.title = format!("{} - Extracted Pages", self.title);
        new_doc.author = self.author.clone();

        for &index in page_indices {
            if index < self.pages.len() {
                let mut page = self.pages[index].clone();
                page.id = format!("page{}", new_doc.pages.len() + 1);
                new_doc.pages.push(page);
            }
        }

        Ok(new_doc)
    }

    // Validation and repair methods
    pub fn validate_structure(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check for empty document
        if self.pages.is_empty() {
            issues.push("Document has no pages".to_string());
        }

        // Check for invalid page dimensions
        for (i, page) in self.pages.iter().enumerate() {
            if page.width <= 0.0 || page.height <= 0.0 {
                issues.push(format!("Page {} has invalid dimensions", i + 1));
            }
        }

        // Check for orphaned annotations
        for annotation in self.annotations.values() {
            if annotation.page >= self.pages.len() {
                issues.push(format!(
                    "Annotation {} references non-existent page",
                    annotation.id
                ));
            }
        }

        // Check for orphaned form fields
        for form in self.forms.values() {
            if form.page >= self.pages.len() {
                issues.push(format!(
                    "Form field {} references non-existent page",
                    form.id
                ));
            }
        }

        // Check for missing images
        for page in &self.pages {
            for content in &page.content {
                if let PageContent::Image { image_id, .. } = content {
                    if !self.images.contains_key(image_id) {
                        issues.push(format!(
                            "Page {} references missing image {}",
                            page.id, image_id
                        ));
                    }
                }
            }
        }

        Ok(issues)
    }

    pub fn optimize_document(&mut self) -> Result<()> {
        // Remove unused images
        let used_images: std::collections::HashSet<String> = self
            .pages
            .iter()
            .flat_map(|page| &page.content)
            .filter_map(|content| {
                if let PageContent::Image { image_id, .. } = content {
                    Some(image_id.clone())
                } else {
                    None
                }
            })
            .collect();

        self.images.retain(|id, _| used_images.contains(id));

        // Remove empty annotations
        self.annotations.retain(|_, annotation| {
            !annotation.content.is_empty()
                || matches!(
                    annotation.annotation_type,
                    AnnotationType::Highlight | AnnotationType::Link
                )
        });

        self.modified = Utc::now();
        Ok(())
    }
}

impl OfficeDocument for PdfDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        // Parse PDF using lopdf for comprehensive parsing
        let lopdf_doc = LoDocument::load_mem(data)
            .map_err(|e| OfficeError::InvalidFormat(format!("PDF parsing error: {}", e)))?;

        let mut pdf_doc = PdfDocument::new();

        // Extract comprehensive document metadata
        if let Ok(info_dict) = lopdf_doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info_dict.as_reference() {
                if let Ok(info_obj) = lopdf_doc.get_object(info_ref) {
                    if let Ok(info_dict) = info_obj.as_dict() {
                        // Extract all metadata fields
                        if let Ok(title) = info_dict.get(b"Title") {
                            if let Ok(title_str) = title.as_str() {
                                pdf_doc.title = String::from_utf8_lossy(title_str).to_string();
                            }
                        }
                        if let Ok(author) = info_dict.get(b"Author") {
                            if let Ok(author_str) = author.as_str() {
                                pdf_doc.author = String::from_utf8_lossy(author_str).to_string();
                            }
                        }
                        if let Ok(subject) = info_dict.get(b"Subject") {
                            if let Ok(subject_str) = subject.as_str() {
                                pdf_doc.subject = String::from_utf8_lossy(subject_str).to_string();
                            }
                        }
                        if let Ok(keywords) = info_dict.get(b"Keywords") {
                            if let Ok(keywords_str) = keywords.as_str() {
                                pdf_doc.keywords =
                                    String::from_utf8_lossy(keywords_str).to_string();
                            }
                        }
                        if let Ok(creator) = info_dict.get(b"Creator") {
                            if let Ok(creator_str) = creator.as_str() {
                                pdf_doc.last_modified_by =
                                    String::from_utf8_lossy(creator_str).to_string();
                            }
                        }
                    }
                }
            }
        }

        // Extract pages with comprehensive information
        let page_numbers = lopdf_doc.get_pages();
        for (page_num, (page_id, _)) in page_numbers.iter().enumerate() {
            if let Ok(page_obj) = lopdf_doc.get_object((*page_id, 0)) {
                if let Ok(page_dict) = page_obj.as_dict() {
                    // Get comprehensive page dimensions
                    let (width, height) = if let Ok(mediabox) = page_dict.get(b"MediaBox") {
                        if let Ok(mediabox_array) = mediabox.as_array() {
                            if mediabox_array.len() >= 4 {
                                let width = mediabox_array[2].as_i64().unwrap_or(612) as f64;
                                let height = mediabox_array[3].as_i64().unwrap_or(792) as f64;
                                (width, height)
                            } else {
                                (612.0, 792.0) // Default Letter size
                            }
                        } else {
                            (612.0, 792.0)
                        }
                    } else {
                        (612.0, 792.0)
                    };

                    // Extract rotation if present
                    let rotation = if let Ok(rotate) = page_dict.get(b"Rotate") {
                        if let Ok(rotate_val) = rotate.as_i64() {
                            match rotate_val {
                                90 => PageRotation::Clockwise90,
                                180 => PageRotation::Clockwise180,
                                270 => PageRotation::Clockwise270,
                                _ => PageRotation::None,
                            }
                        } else {
                            PageRotation::None
                        }
                    } else {
                        PageRotation::None
                    };

                    let page = PdfPageInfo {
                        id: format!("page{}", page_num + 1),
                        width,
                        height,
                        rotation,
                        content: Vec::new(), // Text extraction would require content stream parsing
                        annotations: Vec::new(),
                        media_box: Rectangle::new(0.0, 0.0, width, height),
                        crop_box: None,
                        bleed_box: None,
                        trim_box: None,
                        art_box: None,
                        transparency_group: None,
                    };

                    pdf_doc.pages.push(page);
                }
            }
        }

        // Extract text from all pages
        if !pdf_doc.pages.is_empty() {
            let page_numbers: Vec<u32> = (1..=pdf_doc.pages.len()).map(|i| i as u32).collect();
            if let Ok(extracted_text) = lopdf_doc.extract_text(&page_numbers) {
                // For now, just add the extracted text as content to the first page
                // A more sophisticated implementation would parse the positioning
                if !extracted_text.trim().is_empty() && !pdf_doc.pages.is_empty() {
                    pdf_doc.pages[0].content.push(PageContent::Text {
                        content: extracted_text.trim().to_string(),
                        x: 72.0,  // 1 inch margin
                        y: 720.0, // Near top of page
                        font: "Helvetica".to_string(),
                        size: 12.0,
                        color: Color::black(),
                        style: TextStyle::default(),
                        transformation: None,
                    });
                }
            }
        }

        // Check if document is encrypted
        if lopdf_doc.is_encrypted() {
            pdf_doc.security_settings.encryption_enabled = true;
        }

        // Set modification time
        pdf_doc.modified = Utc::now();

        Ok(pdf_doc)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        fn text_operations(text: &str, x: f64, y: f64, size: f64, color: &Color) -> Vec<Op> {
            vec![
                Op::SaveGraphicsState,
                Op::StartTextSection,
                Op::SetTextCursor {
                    pos: Point::new(Mm(x as f32 * 0.352778), Mm(y as f32 * 0.352778)),
                },
                Op::SetFont {
                    font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
                    size: Pt(size as f32),
                },
                Op::SetFillColor {
                    col: color.to_printpdf_color(),
                },
                Op::ShowText {
                    items: vec![TextItem::Text(text.to_string())],
                },
                Op::EndTextSection,
                Op::RestoreGraphicsState,
            ]
        }

        fn page_operations(page: &PdfPageInfo) -> Vec<Op> {
            let mut operations = Vec::new();

            for content in &page.content {
                match content {
                    PageContent::Text {
                        content,
                        x,
                        y,
                        size,
                        color,
                        ..
                    } => operations.extend(text_operations(content, *x, *y, *size, color)),
                    PageContent::Shape {
                        shape_type: ShapeType::Rectangle,
                        x,
                        y,
                        width,
                        height,
                        fill_color,
                        stroke_color,
                        stroke_width,
                        ..
                    } => {
                        let points = vec![
                            LinePoint {
                                p: Point::new(Mm(*x as f32 * 0.352778), Mm(*y as f32 * 0.352778)),
                                bezier: false,
                            },
                            LinePoint {
                                p: Point::new(
                                    Mm((*x + *width) as f32 * 0.352778),
                                    Mm(*y as f32 * 0.352778),
                                ),
                                bezier: false,
                            },
                            LinePoint {
                                p: Point::new(
                                    Mm((*x + *width) as f32 * 0.352778),
                                    Mm((*y + *height) as f32 * 0.352778),
                                ),
                                bezier: false,
                            },
                            LinePoint {
                                p: Point::new(
                                    Mm(*x as f32 * 0.352778),
                                    Mm((*y + *height) as f32 * 0.352778),
                                ),
                                bezier: false,
                            },
                        ];
                        let mode = match (fill_color.is_some(), stroke_color.is_some()) {
                            (true, true) => PaintMode::FillStroke,
                            (true, false) => PaintMode::Fill,
                            _ => PaintMode::Stroke,
                        };

                        operations.push(Op::SaveGraphicsState);
                        if let Some(fill) = fill_color {
                            operations.push(Op::SetFillColor {
                                col: fill.to_printpdf_color(),
                            });
                        }
                        if let Some(stroke) = stroke_color {
                            operations.push(Op::SetOutlineColor {
                                col: stroke.to_printpdf_color(),
                            });
                        }
                        operations.push(Op::SetOutlineThickness {
                            pt: Pt(*stroke_width as f32),
                        });
                        operations.push(Op::DrawPolygon {
                            polygon: Polygon {
                                rings: vec![PolygonRing { points }],
                                mode,
                                winding_order: WindingOrder::NonZero,
                            },
                        });
                        operations.push(Op::RestoreGraphicsState);
                    }
                    PageContent::Line {
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        color,
                        width,
                        ..
                    } => {
                        operations.extend([
                            Op::SaveGraphicsState,
                            Op::SetOutlineColor {
                                col: color.to_printpdf_color(),
                            },
                            Op::SetOutlineThickness {
                                pt: Pt(*width as f32),
                            },
                            Op::DrawLine {
                                line: printpdf::Line {
                                    points: vec![
                                        LinePoint {
                                            p: Point::new(
                                                Mm(*start_x as f32 * 0.352778),
                                                Mm(*start_y as f32 * 0.352778),
                                            ),
                                            bezier: false,
                                        },
                                        LinePoint {
                                            p: Point::new(
                                                Mm(*end_x as f32 * 0.352778),
                                                Mm(*end_y as f32 * 0.352778),
                                            ),
                                            bezier: false,
                                        },
                                    ],
                                    is_closed: false,
                                },
                            },
                            Op::RestoreGraphicsState,
                        ]);
                    }
                    PageContent::Table { x, y, rows, .. } => {
                        let mut table_y = *y;
                        for row in rows {
                            let mut table_x = *x;
                            for cell in &row.cells {
                                operations.extend(text_operations(
                                    &cell.content,
                                    table_x,
                                    table_y,
                                    10.0,
                                    &Color::black(),
                                ));
                                table_x += cell.width;
                            }
                            table_y -= row.height;
                        }
                    }
                    _ => {}
                }
            }

            operations
        }

        if self.pages.is_empty() {
            return Err(OfficeError::InvalidFormat(
                "Cannot create PDF with no pages".to_string(),
            ));
        }

        let mut document = printpdf::PdfDocument::new(&self.title);
        document.metadata.info.author = self.author.clone();
        document.metadata.info.subject = self.subject.clone();
        document.metadata.info.keywords = self
            .keywords
            .split(',')
            .map(str::trim)
            .filter(|keyword| !keyword.is_empty())
            .map(str::to_string)
            .collect();

        let pages = self
            .pages
            .iter()
            .map(|page| {
                printpdf::PdfPage::new(
                    Mm(page.width as f32 * 0.352778),
                    Mm(page.height as f32 * 0.352778),
                    page_operations(page),
                )
            })
            .collect();
        document.with_pages(pages);

        let mut warnings = Vec::new();
        Ok(document.save(&PdfSaveOptions::default(), &mut warnings))
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.to_bytes()?;
        fs::write(path, data)?;
        Ok(())
    }
}
