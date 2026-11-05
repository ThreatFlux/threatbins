//! Versioned, offline report rendering shared by pipeline and MCP consumers.
//!
//! The caller supplies immutable facts and approved prose. This module formats
//! them; it never evaluates scores, fetches URLs, authorizes users or stores files.
#[cfg(feature = "report")]
mod html;
#[cfg(feature = "report")]
mod inspect;
#[cfg(feature = "report")]
pub use inspect::inspect_report;
#[cfg(feature = "report")]
mod layout;
mod navigation;
#[cfg(feature = "report")]
mod pdf;
#[cfg(feature = "report")]
mod pptx;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const REPORT_SCHEMA_VERSION: u32 = 1;
pub const REPORT_TEMPLATE: &str = "executive_v1";
#[cfg(feature = "report")]
pub(crate) const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("invalid report at {path}: {message}")]
    InvalidInput { path: String, message: String },
    #[error("report limit exceeded: {0}")]
    LimitExceeded(String),
    #[error("document digest does not match expected_document_sha256")]
    DigestMismatch,
    #[error("unsupported report template: {0}")]
    UnsupportedTemplate(String),
    #[error("font cannot represent U+{0:04X}; supply a suitable font")]
    UnsupportedGlyph(u32),
    #[error("report serialization: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("report document: {0}")]
    Document(#[from] crate::OfficeError),
    #[cfg(feature = "report")]
    #[error("report ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("report IO: {0}")]
    Io(#[from] std::io::Error),
}
pub type ReportResult<T> = std::result::Result<T, ReportError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDocument {
    pub schema_version: u32,
    pub document_id: String,
    pub revision: String,
    pub report_type: String,
    pub title: String,
    pub subtitle: String,
    pub period: String,
    pub scope: String,
    pub comparison: String,
    pub provenance: ReportProvenance,
    #[serde(default)]
    pub definitions: Vec<ReportDefinition>,
    #[serde(default)]
    pub limitations: Vec<String>,
    #[serde(default)]
    pub sections: Vec<ReportSection>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportProvenance {
    pub evidence_sha256: String,
    pub sources: Vec<ReportSource>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportSource {
    pub id: String,
    pub revision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDefinition {
    pub term: String,
    pub meaning: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportSection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blocks: Vec<ReportBlock>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReportBlock {
    Paragraph {
        text: String,
    },
    Table {
        table: ReportTable,
    },
    AnchoredTable {
        table: ReportTable,
        row_ids: Vec<String>,
    },
    Links {
        links: Vec<ReportLink>,
    },
    Chart {
        chart: ReportChart,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportLink {
    pub label: String,
    /// An exact section or anchored-table row in this document. External URLs are not supported.
    pub target_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportTable {
    pub caption: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportChart {
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
    pub series: Vec<ReportSeries>,
    pub minimum: f64,
    pub maximum: f64,
    pub target: Option<f64>,
    #[serde(default)]
    pub bands: Vec<ReportBand>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportSeries {
    pub name: String,
    pub values: Vec<Option<f64>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportBand {
    pub label: String,
    pub start: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Pdf,
    Html,
    Pptx,
}
impl ReportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Html => "html",
            Self::Pptx => "pptx",
        }
    }
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Html => "text/html; charset=utf-8",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportLimits {
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_pages: usize,
    pub max_table_rows: usize,
    pub max_chart_points: usize,
}
impl Default for ReportLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 2 * 1024 * 1024,
            max_output_bytes: 32 * 1024 * 1024,
            max_pages: 256,
            max_table_rows: 10_000,
            max_chart_points: 1_000,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ReportRenderOptions {
    pub format: ReportFormat,
    pub template: String,
    pub expected_document_sha256: Option<String>,
    pub limits: ReportLimits,
    /// Optional embeddable TrueType/OpenType font. No filesystem/network reads.
    /// The caller is responsible for its font license. Unsupported glyphs fail.
    pub font_data: Option<Vec<u8>>,
    /// Installed family name used by editable PowerPoint text; it must match
    /// the supplied font data. PDF and HTML embed the bytes directly.
    pub font_family: String,
}
impl ReportRenderOptions {
    pub fn new(format: ReportFormat) -> Self {
        Self {
            format,
            template: REPORT_TEMPLATE.into(),
            expected_document_sha256: None,
            limits: ReportLimits::default(),
            font_data: None,
            font_family: "Noto Sans".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportCapabilities {
    pub schema_version: u32,
    pub templates: Vec<String>,
    pub formats: Vec<ReportFormat>,
    pub native_powerpoint_charts: bool,
    pub chart_types: Vec<String>,
    pub native_powerpoint_tables: bool,
    pub embedded_chart_workbook: bool,
    pub offline_html: bool,
    pub embedded_pdf_font: bool,
    pub complex_script_shaping: bool,
    pub default_limits: ReportLimits,
}
#[cfg(feature = "report")]
pub fn report_capabilities() -> ReportCapabilities {
    ReportCapabilities {
        schema_version: REPORT_SCHEMA_VERSION,
        templates: vec![REPORT_TEMPLATE.into()],
        formats: vec![ReportFormat::Pdf, ReportFormat::Html, ReportFormat::Pptx],
        native_powerpoint_charts: true,
        chart_types: vec!["line".into()],
        native_powerpoint_tables: true,
        embedded_chart_workbook: true,
        offline_html: true,
        embedded_pdf_font: true,
        complex_script_shaping: false,
        default_limits: ReportLimits::default(),
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportManifest {
    pub schema_version: u32,
    pub sdk_version: String,
    pub document_id: String,
    pub revision: String,
    pub document_sha256: String,
    pub evidence_sha256: String,
    pub template: String,
    pub format: ReportFormat,
    pub mime_type: String,
    pub byte_size: usize,
    pub output_sha256: String,
    /// None for continuous HTML; PDF/PPTX have a verified page/slide count.
    pub pages_or_slides: Option<usize>,
    pub inspection: ReportInspection,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportInspection {
    /// SDK structural generation checks, never a human visual-acceptance claim.
    pub structural_checks: Vec<String>,
    pub native_editable_text: bool,
    pub native_editable_tables: bool,
    pub native_editable_charts: bool,
    pub font_sha256: String,
    pub visual_review_required: bool,
}
#[derive(Debug, Clone)]
pub struct RenderedReport {
    pub bytes: Vec<u8>,
    pub manifest: ReportManifest,
}

pub fn report_document_sha256(document: &ReportDocument) -> ReportResult<String> {
    validate(document, &ReportLimits::default())?;
    Ok(sha256(&serde_json::to_vec(document)?))
}
pub(crate) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
pub(crate) fn invalid(path: &str, message: &str) -> ReportError {
    ReportError::InvalidInput {
        path: path.into(),
        message: message.into(),
    }
}
#[cfg(feature = "report")]
pub(crate) fn xml(s: &str) -> String {
    quick_xml::escape::escape(s).into_owned()
}

#[cfg(feature = "report")]
pub fn render_report(
    document: &ReportDocument,
    options: &ReportRenderOptions,
) -> ReportResult<RenderedReport> {
    if options.template != REPORT_TEMPLATE {
        return Err(ReportError::UnsupportedTemplate(options.template.clone()));
    }
    validate(document, &options.limits)?;
    let digest = report_document_sha256(document)?;
    if options
        .expected_document_sha256
        .as_ref()
        .is_some_and(|expected| expected != &digest)
    {
        return Err(ReportError::DigestMismatch);
    }
    if options.font_family.trim().is_empty()
        || options.font_family.chars().count() > 128
        || options.font_family.chars().any(char::is_control)
    {
        return Err(invalid("font_family", "expected a short font family name"));
    }
    let font = options.font_data.as_deref().unwrap_or(DEFAULT_FONT);
    if font.len() > 10 * 1024 * 1024 {
        return Err(ReportError::LimitExceeded("font bytes".into()));
    }
    let metrics = layout::FontMetrics::new(font)?;
    let pages = layout::paginate(document, &metrics, options.limits.max_pages)?;
    let bytes = match options.format {
        ReportFormat::Html => html::render(document, &digest, font).into_bytes(),
        ReportFormat::Pdf => pdf::render(document, &pages, font)?,
        ReportFormat::Pptx => pptx::render(document, &pages, font, &options.font_family)?,
    };
    if bytes.len() > options.limits.max_output_bytes {
        return Err(ReportError::LimitExceeded("output bytes".into()));
    }
    let mut checks = vec![
        "validated schema, shape, bounds and finite values".into(),
        "shared source document and measured layout".into(),
    ];
    checks.push(
        match options.format {
            ReportFormat::Pdf => "embedded font and bounded paginated PDF",
            ReportFormat::Html => "escaped script-free self-contained HTML",
            ReportFormat::Pptx => {
                "widescreen OOXML, native editable text/tables/charts and embedded chart data"
            }
        }
        .into(),
    );
    let mut result = RenderedReport {
        manifest: ReportManifest {
            schema_version: 1,
            sdk_version: env!("CARGO_PKG_VERSION").into(),
            document_id: document.document_id.clone(),
            revision: document.revision.clone(),
            document_sha256: digest,
            evidence_sha256: document.provenance.evidence_sha256.clone(),
            template: options.template.clone(),
            format: options.format,
            mime_type: options.format.mime_type().into(),
            byte_size: bytes.len(),
            output_sha256: sha256(&bytes),
            pages_or_slides: (options.format != ReportFormat::Html).then_some(pages.len()),
            inspection: ReportInspection {
                structural_checks: checks,
                native_editable_text: options.format == ReportFormat::Pptx,
                native_editable_tables: options.format == ReportFormat::Pptx
                    && document.sections.iter().flat_map(|s| &s.blocks).any(|b| {
                        matches!(
                            b,
                            ReportBlock::Table { .. }
                                | ReportBlock::AnchoredTable { .. }
                                | ReportBlock::Chart { .. }
                        )
                    }),
                native_editable_charts: options.format == ReportFormat::Pptx
                    && document
                        .sections
                        .iter()
                        .flat_map(|s| &s.blocks)
                        .any(|b| matches!(b, ReportBlock::Chart { .. })),
                font_sha256: sha256(font),
                visual_review_required: true,
            },
        },
        bytes,
    };
    result.manifest.inspection = inspect_report(&result.bytes, &result.manifest)?;
    Ok(result)
}
/// Validate the typed document without loading any rendering engine.
pub fn validate_report_document(
    document: &ReportDocument,
    limits: &ReportLimits,
) -> ReportResult<()> {
    validate(document, limits)
}

fn validate(d: &ReportDocument, limits: &ReportLimits) -> ReportResult<()> {
    let cap = ReportLimits::default();
    if limits.max_input_bytes == 0
        || limits.max_input_bytes > cap.max_input_bytes
        || limits.max_output_bytes == 0
        || limits.max_output_bytes > cap.max_output_bytes
        || limits.max_pages == 0
        || limits.max_pages > cap.max_pages
        || limits.max_table_rows > cap.max_table_rows
        || limits.max_chart_points > cap.max_chart_points
    {
        return Err(invalid(
            "limits",
            "limits may only reduce the documented hard bounds",
        ));
    }
    // Bound serialization before allocating a JSON copy. The writer stops on
    // the first oversized chunk, including oversized caller-owned collections.
    struct Budget(usize);
    impl std::io::Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.0 {
                return Err(std::io::Error::other("report input limit"));
            }
            self.0 -= bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget(limits.max_input_bytes), d)
        .map_err(|_| ReportError::LimitExceeded("input bytes".into()))?;
    if d.schema_version != REPORT_SCHEMA_VERSION {
        return Err(invalid("schema_version", "unsupported version"));
    }
    for (path, value) in [
        ("document_id", &d.document_id),
        ("revision", &d.revision),
        ("report_type", &d.report_type),
        ("title", &d.title),
        ("period", &d.period),
        ("scope", &d.scope),
        ("comparison", &d.comparison),
    ] {
        if value.chars().count() > 256 && matches!(path, "document_id" | "revision" | "report_type")
        {
            return Err(invalid(path, "identifier exceeds 256 characters"));
        }
        if value.trim().is_empty() {
            return Err(invalid(path, "required nonempty text"));
        }
    }
    fn text(s: &str) -> ReportResult<()> {
        for c in s.chars() {
            if ('\u{202A}'..='\u{202E}').contains(&c) || ('\u{2066}'..='\u{2069}').contains(&c) {
                return Err(invalid("text", "bidirectional overrides are not supported"));
            }
            if matches!(c as u32, 0x0590..=0x109F | 0x1780..=0x18AF | 0xA800..=0xABFF | 0xFB1D..=0xFEFC)
            {
                return Err(invalid(
                    "text",
                    "this script requires bidirectional or complex-script shaping, which is unavailable",
                ));
            }
        }
        if s.chars().any(|c| {
            (c < ' ' && !matches!(c, '\n' | '\t' | '\r')) || c == '\u{fffe}' || c == '\u{ffff}'
        }) {
            return Err(invalid("text", "XML-illegal control character"));
        }
        if s.chars().count() > 20_000 {
            return Err(ReportError::LimitExceeded("text field".into()));
        }
        Ok(())
    }
    let val = serde_json::to_value(d)?;
    fn walk(v: &serde_json::Value) -> ReportResult<()> {
        match v {
            serde_json::Value::String(s) => text(s)?,
            serde_json::Value::Array(a) => {
                for v in a {
                    walk(v)?;
                }
            }
            serde_json::Value::Object(o) => {
                for v in o.values() {
                    walk(v)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    walk(&val)?;
    navigation::validate(d)?;
    if d.provenance.evidence_sha256.len() != 64
        || !d
            .provenance
            .evidence_sha256
            .bytes()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(invalid(
            "provenance.evidence_sha256",
            "expected SHA-256 hex",
        ));
    }
    if d.sections.len() > 128
        || d.provenance.sources.len() > 1000
        || d.definitions.len() > 256
        || d.limitations.len() > 256
    {
        return Err(ReportError::LimitExceeded("document collections".into()));
    }
    let mut ids = std::collections::HashSet::new();
    let mut rows = 0;
    let mut links = 0;
    let mut points = 0;
    for section in &d.sections {
        if section.id.is_empty() || !ids.insert(&section.id) {
            return Err(invalid("sections.id", "IDs must be nonempty and unique"));
        }
        if section.blocks.len() > 256 {
            return Err(ReportError::LimitExceeded("section blocks".into()));
        }
        for block in &section.blocks {
            match block {
                ReportBlock::Paragraph { .. } => {}
                ReportBlock::Links { links: entries } => {
                    links += entries.len();
                }
                ReportBlock::Table { table } | ReportBlock::AnchoredTable { table, .. } => {
                    if table.headers.is_empty()
                        || table.headers.len() > 8
                        || table.rows.iter().any(|r| r.len() != table.headers.len())
                    {
                        return Err(invalid(
                            "table",
                            "requires 1–8 columns and rectangular rows",
                        ));
                    }
                    rows += table.rows.len();
                }
                ReportBlock::Chart { chart: c } => {
                    points += c.labels.len() * c.series.len();
                    if c.labels.is_empty()
                        || c.labels.len() > 1000
                        || c.series.is_empty()
                        || c.series.len() > 8
                        || !c.minimum.is_finite()
                        || !c.maximum.is_finite()
                        || c.minimum >= c.maximum
                    {
                        return Err(invalid("chart", "invalid axis, series or labels"));
                    }
                    for s in &c.series {
                        if s.values.len() != c.labels.len()
                            || s.values
                                .iter()
                                .flatten()
                                .any(|v| !v.is_finite() || *v < c.minimum || *v > c.maximum)
                        {
                            return Err(invalid(
                                "chart.series",
                                "points must match labels and finite axis bounds",
                            ));
                        }
                    }
                    if c.target
                        .is_some_and(|v| !v.is_finite() || v < c.minimum || v > c.maximum)
                    {
                        return Err(invalid("chart.target", "outside axis bounds"));
                    }
                    let mut prev = c.minimum - 1.0;
                    for b in &c.bands {
                        if !b.start.is_finite()
                            || b.start < c.minimum
                            || b.start > c.maximum
                            || b.start <= prev
                        {
                            return Err(invalid(
                                "chart.bands",
                                "starts must be increasing within axis bounds",
                            ));
                        }
                        prev = b.start;
                    }
                }
            }
        }
    }
    if rows > limits.max_table_rows
        || links > limits.max_table_rows
        || points > limits.max_chart_points
    {
        return Err(ReportError::LimitExceeded(
            "table rows, internal links or chart points".into(),
        ));
    }
    Ok(())
}
