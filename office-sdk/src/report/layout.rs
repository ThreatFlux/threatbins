use super::*;

pub(crate) const WIDTH: f64 = 960.0;
pub(crate) const HEIGHT: f64 = 540.0;
pub(crate) const LEFT: f64 = 40.0;
pub(crate) const BODY_TOP: f64 = 120.0;
pub(crate) const BODY_WIDTH: f64 = 880.0;
pub(crate) const BODY_HEIGHT: f64 = 360.0;
pub(crate) const LINE: f64 = 16.0;

pub(crate) struct FontMetrics {
    font: printpdf::ParsedFont,
}
impl FontMetrics {
    pub fn new(bytes: &[u8]) -> ReportResult<Self> {
        Ok(Self {
            font: printpdf::ParsedFont::from_bytes(bytes, 0, &mut Vec::new())
                .ok_or_else(|| invalid("font_data", "invalid TrueType/OpenType font"))?,
        })
    }
    pub fn width(&self, text: &str, size: f64) -> ReportResult<f64> {
        let mut width = 0.0;
        for c in text.chars() {
            if c == '\n' || c == '\r' {
                continue;
            }
            let c = if c == '\t' { ' ' } else { c };
            let glyph = self
                .font
                .lookup_glyph_index(c as u32)
                .ok_or(ReportError::UnsupportedGlyph(c as u32))?;
            width += f64::from(self.font.get_glyph_width(glyph).unwrap_or(0)) * size
                / f64::from(self.font.units_per_em);
        }
        Ok(width)
    }
    pub fn wrap(&self, text: &str, size: f64, width: f64) -> ReportResult<Vec<String>> {
        let mut result = Vec::new();
        for paragraph in text.split('\n') {
            let mut line = String::new();
            let mut line_width = 0.0;
            // Prefer word boundaries; exceptionally long words are split by measured
            // glyph width so complete source names remain within the page.
            for c in paragraph.chars() {
                let c = if c == '\t' {
                    ' '
                } else if c == '\r' {
                    continue;
                } else {
                    c
                };
                let w = self.width(&c.to_string(), size)?;
                if line_width + w > width && !line.is_empty() {
                    if let Some(split) = line.rfind(char::is_whitespace).filter(|i| *i > 0) {
                        let separator_len = line[split..].chars().next().unwrap().len_utf8();
                        let tail = line[split + separator_len..].to_string();
                        result.push(line[..split].to_string());
                        line = tail;
                        line_width = self.width(&line, size)?;
                    } else {
                        result.push(line);
                        line = String::new();
                        line_width = 0.0;
                    }
                }
                line.push(c);
                line_width += w;
            }
            result.push(line);
        }
        Ok(result)
    }
}
#[derive(Debug, Clone)]
pub(crate) enum PageBody {
    Text(Vec<String>),
    Table {
        headers: Vec<Vec<String>>,
        rows: Vec<Vec<Vec<String>>>,
    },
    Chart(ReportChart),
}
#[derive(Debug, Clone)]
pub(crate) struct Page {
    pub title: String,
    pub body: PageBody,
    pub notes: String,
}

pub(crate) fn paginate(d: &ReportDocument, m: &FontMetrics, max: usize) -> ReportResult<Vec<Page>> {
    let common = format!(
        "Document: {}\nRevision: {}\nPeriod: {}\nScope: {}\nComparison: {}\nEvidence SHA-256: {}\nTemplate: {}",
        d.document_id,
        d.revision,
        d.period,
        d.scope,
        d.comparison,
        d.provenance.evidence_sha256,
        REPORT_TEMPLATE
    );
    let mut pages = Vec::new();
    let intro = format!(
        "{}\n{}\n\nPeriod: {}\nScope: {}\nComparison: {}\nType: {}\nDocument: {}\nRevision: {}",
        d.title,
        d.subtitle,
        d.period,
        d.scope,
        d.comparison,
        d.report_type,
        d.document_id,
        d.revision
    );
    add_text(&mut pages, "Report overview", &intro, m, &common, max)?;
    for section in &d.sections {
        for block in &section.blocks {
            match block {
                ReportBlock::Paragraph { text } => {
                    add_text(&mut pages, &section.title, text, m, &common, max)?
                }
                ReportBlock::Table { table } => {
                    add_table(&mut pages, &section.title, table, m, &common, max)?
                }
                ReportBlock::Chart { chart } => {
                    add_text(
                        &mut pages,
                        &section.title,
                        &format!(
                            "{}\n{}\n{}",
                            chart.title,
                            chart.description,
                            chart_context(chart)
                        ),
                        m,
                        &common,
                        max,
                    )?;
                    push(
                        &mut pages,
                        Page {
                            title: chart.title.clone(),
                            body: PageBody::Chart(chart.clone()),
                            notes: format!(
                                "{common}\n{}\n{}",
                                chart.description,
                                chart_context(chart)
                            ),
                        },
                        m,
                        max,
                    )?;
                    let mut headers = vec!["Observation".to_string()];
                    headers.extend(chart.series.iter().map(|s| s.name.clone()));
                    let rows = chart
                        .labels
                        .iter()
                        .enumerate()
                        .map(|(i, label)| {
                            let mut row = vec![format!("{} · {}", i + 1, label)];
                            row.extend(chart.series.iter().map(|s| {
                                s.values[i].map_or_else(|| "Not measured".into(), |v| v.to_string())
                            }));
                            row
                        })
                        .collect();
                    // Accessible data table also makes every plotted value, full
                    // label and missing point explicit in all output formats.
                    add_table(
                        &mut pages,
                        &chart.title,
                        &ReportTable {
                            caption: "Chart data · missing values are not zero".into(),
                            headers,
                            rows,
                        },
                        m,
                        &common,
                        max,
                    )?;
                }
            }
        }
    }
    if !d.definitions.is_empty() {
        add_text(
            &mut pages,
            "Definitions",
            &d.definitions
                .iter()
                .map(|v| format!("{}: {}", v.term, v.meaning))
                .collect::<Vec<_>>()
                .join("\n\n"),
            m,
            &common,
            max,
        )?;
    }
    if !d.limitations.is_empty() {
        add_text(
            &mut pages,
            "Limitations",
            &d.limitations.join("\n\n"),
            m,
            &common,
            max,
        )?;
    }
    let provenance = format!(
        "Evidence SHA-256: {}\nDocument SHA-256: {}\n{}",
        d.provenance.evidence_sha256,
        report_document_sha256(d)?,
        d.provenance
            .sources
            .iter()
            .map(|v| format!("{} @ {}", v.id, v.revision))
            .collect::<Vec<_>>()
            .join("\n")
    );
    add_text(&mut pages, "Provenance", &provenance, m, &common, max)?;
    // Keep definitions and limitations in the cover's notes without repeating
    // arbitrarily large source content in every slide's ZIP part.
    if let Some(cover) = pages.first_mut() {
        cover.notes.push_str(&format!(
            "\nDefinitions:\n{}\nLimitations:\n{}\nSources:\n{}",
            d.definitions
                .iter()
                .map(|v| format!("{}: {}", v.term, v.meaning))
                .collect::<Vec<_>>()
                .join("\n"),
            d.limitations.join("\n"),
            provenance
        ));
    }
    Ok(pages)
}
pub(crate) fn chart_context(c: &ReportChart) -> String {
    format!(
        "Axis: {} to {}. Target: {}. Bands: {}. Observation numbers refer to the complete chart data table. Missing values are gaps.",
        c.minimum,
        c.maximum,
        c.target
            .map_or_else(|| "not specified".into(), |v| v.to_string()),
        c.bands
            .iter()
            .map(|b| format!("{} >= {}", b.label, b.start))
            .collect::<Vec<_>>()
            .join("; ")
    )
}
fn push(pages: &mut Vec<Page>, page: Page, m: &FontMetrics, max: usize) -> ReportResult<()> {
    if pages.len() >= max {
        return Err(ReportError::LimitExceeded("pages/slides".into()));
    }
    if m.wrap(&page.title, 22.0, BODY_WIDTH)?.len() > 3 {
        return Err(invalid(
            "section/chart title",
            "title must fit within three header lines",
        ));
    }
    pages.push(page);
    Ok(())
}
fn add_text(
    pages: &mut Vec<Page>,
    title: &str,
    text: &str,
    m: &FontMetrics,
    notes: &str,
    max: usize,
) -> ReportResult<()> {
    let lines = m.wrap(text, 12.0, BODY_WIDTH)?;
    for chunk in lines.chunks(22) {
        push(
            pages,
            Page {
                title: title.into(),
                body: PageBody::Text(chunk.to_vec()),
                notes: notes.into(),
            },
            m,
            max,
        )?;
    }
    Ok(())
}
fn add_table(
    pages: &mut Vec<Page>,
    title: &str,
    t: &ReportTable,
    m: &FontMetrics,
    notes: &str,
    max: usize,
) -> ReportResult<()> {
    let combined = if t.caption.is_empty() {
        title.to_string()
    } else {
        format!("{} · {}", title, t.caption)
    };
    let title = if m.wrap(&combined, 22.0, BODY_WIDTH)?.len() <= 3 {
        combined.as_str()
    } else {
        add_text(pages, title, &t.caption, m, notes, max)?;
        title
    };
    let col_width = BODY_WIDTH / t.headers.len() as f64 - 16.0;
    let headers = t
        .headers
        .iter()
        .map(|v| m.wrap(v, 11.0, col_width))
        .collect::<ReportResult<Vec<_>>>()?;
    let header_height = row_height(&headers);
    if header_height > 100.0 {
        return Err(invalid("table.headers", "header exceeds bounded layout"));
    }
    let mut rows = Vec::new();
    let mut used = header_height;
    for row in &t.rows {
        let wrapped = row
            .iter()
            .map(|v| m.wrap(v, 11.0, col_width))
            .collect::<ReportResult<Vec<_>>>()?;
        let line_count = wrapped.iter().map(Vec::len).max().unwrap_or(1);
        // Split oversized rows vertically, retaining native table cells and all
        // text. Continued rows repeat the header on each new page.
        for start in (0..line_count).step_by(12) {
            let part = wrapped
                .iter()
                .map(|cell| {
                    cell.iter()
                        .skip(start)
                        .take(12)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let height = row_height(&part);
            if used + height > BODY_HEIGHT && !rows.is_empty() {
                push(
                    pages,
                    Page {
                        title: title.into(),
                        body: PageBody::Table {
                            headers: headers.clone(),
                            rows,
                        },
                        notes: notes.into(),
                    },
                    m,
                    max,
                )?;
                rows = Vec::new();
                used = header_height;
            }
            used += height;
            rows.push(part);
        }
    }
    if !rows.is_empty() || t.rows.is_empty() {
        push(
            pages,
            Page {
                title: title.into(),
                body: PageBody::Table { headers, rows },
                notes: notes.into(),
            },
            m,
            max,
        )?;
    }
    Ok(())
}
pub(crate) fn row_height(row: &[Vec<String>]) -> f64 {
    row.iter().map(Vec::len).max().unwrap_or(1).max(1) as f64 * LINE + 12.0
}
pub(crate) const COLORS: [&str; 8] = [
    "2563EB", "B45309", "047857", "9333EA", "BE123C", "0369A1", "525252", "A21CAF",
];
