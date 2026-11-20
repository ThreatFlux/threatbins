//! Measured section layout for the native widescreen presentation.

use super::{layout::*, *};
use std::fmt::Write;

const COLUMN_GAP: f64 = 24.0;
const BLOCK_GAP: f64 = 8.0;
const CHART_HEIGHT: f64 = 182.0;
const GRID_CHART_HEIGHT: f64 = 126.0;
const LINE_PADDING: f64 = 4.0;
const CELL_INSET: f64 = 4.0;
const COMPACT_CELL_INSET: f64 = 2.0;

pub(super) const TABLE_PREVIEW_PREFIX: &str = "office-report-table-preview-v1:";
pub(super) const TABLE_PREVIEW_CAPTION_SUFFIX: &str =
    " rows shown. Complete table in speaker notes.";
pub(super) const TABLE_PREVIEW_SIZE: f64 = 10.0;

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub(super) struct Slide<'a> {
    pub id: &'a str,
    pub items: Vec<(Frame, Item<'a>)>,
    pub notes: String,
    pub table_previews: Vec<String>,
    pub row_ids: Vec<&'a str>,
}

#[derive(Clone)]
pub(super) enum Item<'a> {
    Text {
        lines: Vec<String>,
        size: f64,
        target: Option<&'a str>,
        color: &'static str,
    },
    Table(Table),
    TablePreview(Vec<String>),
    Chart {
        chart: &'a ReportChart,
        show_categories: bool,
        native_legend: bool,
        height: f64,
    },
}

#[derive(Clone)]
pub(super) struct Table {
    pub headers: Vec<Vec<String>>,
    pub rows: Vec<Vec<Vec<String>>>,
    pub widths: Vec<f64>,
    pub size: f64,
    pub inset: f64,
}

impl Table {
    pub fn row_height(&self, cells: &[Vec<String>]) -> f64 {
        text_height(cells.iter().map(Vec::len).max().unwrap_or(1), self.size) + self.inset * 2.0
    }
}

pub(super) fn line_height(size: f64) -> f64 {
    (size * 1.28).ceil()
}

fn text_height(lines: usize, size: f64) -> f64 {
    lines as f64 * line_height(size) + LINE_PADDING
}

impl Item<'_> {
    fn height(&self) -> f64 {
        match self {
            Self::Text { lines, size, .. } => text_height(lines.len(), *size),
            Self::TablePreview(lines) => text_height(lines.len(), TABLE_PREVIEW_SIZE),
            Self::Table(table) => std::iter::once(&table.headers)
                .chain(&table.rows)
                .map(|row| table.row_height(row))
                .sum(),
            Self::Chart { height, .. } => *height,
        }
    }
}

fn measured_text<'a>(
    metrics: &FontMetrics,
    text: &str,
    size: f64,
    width: f64,
    target: Option<&'a str>,
) -> ReportResult<Item<'a>> {
    Ok(Item::Text {
        // Leave space for the small advance differences between font engines.
        lines: metrics.wrap(text, size, width * 0.95)?,
        size,
        target,
        color: if target.is_some() { "2563EB" } else { "172033" },
    })
}

#[derive(Clone, Copy)]
struct LayoutStyle {
    width: f64,
    table_size: f64,
    cell_inset: f64,
    chart_height: f64,
    preview_rows: Option<usize>,
}

impl LayoutStyle {
    fn full(table_size: f64) -> Self {
        Self {
            width: BODY_WIDTH,
            table_size,
            cell_inset: CELL_INSET,
            chart_height: CHART_HEIGHT,
            preview_rows: None,
        }
    }

    fn compact(table_size: f64, preview_rows: Option<usize>) -> Self {
        Self {
            cell_inset: COMPACT_CELL_INSET,
            chart_height: GRID_CHART_HEIGHT,
            preview_rows,
            ..Self::full(table_size)
        }
    }

    fn with_width(self, width: f64) -> Self {
        Self { width, ..self }
    }
}

fn measure_table(
    table: &ReportTable,
    metrics: &FontMetrics,
    style: LayoutStyle,
) -> ReportResult<Table> {
    let LayoutStyle {
        width,
        table_size: size,
        cell_inset,
        preview_rows,
        ..
    } = style;
    let rows = &table.rows[..preview_rows
        .unwrap_or(table.rows.len())
        .min(table.rows.len())];
    let count = table.headers.len();
    let floor = 52.0_f64.min(width / count as f64);
    let mut desired = vec![floor; count];
    for row in std::iter::once(&table.headers).chain(rows) {
        for (col, cell) in row.iter().enumerate() {
            for line in cell.lines() {
                desired[col] = desired[col].max(metrics.width(line, size)? + 2.0 * cell_inset);
            }
        }
    }
    let total: f64 = desired.iter().sum();
    let widths = if total <= width {
        desired
            .into_iter()
            .map(|w| w + (width - total) / count as f64)
            .collect::<Vec<_>>()
    } else {
        let remaining = width - floor * count as f64;
        desired
            .into_iter()
            .map(|w| floor + (w - floor) / (total - floor * count as f64) * remaining)
            .collect()
    };
    let wrap_row = |row: &[String]| {
        row.iter()
            .zip(&widths)
            .map(|(cell, width)| metrics.wrap(cell, size, (width - 2.0 * cell_inset) * 0.95))
            .collect::<ReportResult<Vec<_>>>()
    };
    Ok(Table {
        headers: wrap_row(&table.headers)?,
        rows: rows
            .iter()
            .map(|row| wrap_row(row))
            .collect::<ReportResult<_>>()?,
        widths,
        size,
        inset: cell_inset,
    })
}

struct Block<'a> {
    items: Vec<Item<'a>>,
    height: f64,
}

fn measure_block<'a>(
    block: &'a ReportBlock,
    metrics: &FontMetrics,
    style: LayoutStyle,
) -> ReportResult<Block<'a>> {
    let width = style.width;
    let body_size = style.table_size + 4.0;
    let items = match block {
        ReportBlock::Paragraph { text } => {
            vec![measured_text(metrics, text, body_size, width, None)?]
        }
        ReportBlock::Links { links } => links
            .iter()
            .map(|link| {
                measured_text(
                    metrics,
                    &link.label,
                    body_size,
                    width,
                    Some(&link.target_id),
                )
            })
            .collect::<ReportResult<_>>()?,
        ReportBlock::Table { table } | ReportBlock::AnchoredTable { table, .. } => {
            let mut items = Vec::new();
            if !table.caption.is_empty() {
                items.push(measured_text(
                    metrics,
                    &table.caption,
                    body_size,
                    width,
                    None,
                )?);
            }
            items.push(Item::Table(measure_table(table, metrics, style)?));
            if let Some(visible) = style.preview_rows.filter(|n| *n < table.rows.len()) {
                items.push(Item::TablePreview(metrics.wrap(
                    &format!(
                        "{visible} of {}{TABLE_PREVIEW_CAPTION_SUFFIX}",
                        table.rows.len()
                    ),
                    TABLE_PREVIEW_SIZE,
                    width * 0.95,
                )?));
            }
            items
        }
        ReportBlock::Chart { chart } => {
            let tick_width = (width - 72.0) / chart.labels.len() as f64;
            let mut show_categories = chart.labels.len() <= 10;
            for label in &chart.labels {
                show_categories &= !label.contains(['\n', '\r', '\t'])
                    && metrics.width(label, 12.0)? <= tick_width;
            }
            let target = chart.target.map(|target| format!("Target {target}"));
            let labels = chart
                .series
                .iter()
                .zip(COLORS)
                .map(|(series, color)| (series.name.as_str(), color))
                .chain(target.as_deref().map(|label| (label, "172033")))
                .collect::<Vec<_>>();
            let mut legend_width = 0.0;
            let mut native_legend = true;
            for (label, _) in &labels {
                legend_width += metrics.width(label, 12.0)? + 30.0;
                native_legend &= !label.contains(['\n', '\r', '\t']);
            }
            native_legend &= legend_width <= width * 0.9;
            let mut items = vec![
                measured_text(metrics, &chart.title, body_size, width, None)?,
                measured_text(
                    metrics,
                    "Full observations and gaps in chart data and speaker notes.",
                    10.0,
                    width,
                    None,
                )?,
            ];
            if !native_legend {
                for (label, color) in labels {
                    items.push(Item::Text {
                        lines: metrics.wrap(label, 10.0, width * 0.95)?,
                        size: 10.0,
                        target: None,
                        color,
                    });
                }
            }
            items.push(Item::Chart {
                chart,
                show_categories,
                native_legend,
                height: style.chart_height,
            });
            items
        }
    };
    let height = items.iter().map(Item::height).sum();
    Ok(Block { items, height })
}

fn place<'a>(
    items: &mut Vec<(Frame, Item<'a>)>,
    blocks: impl IntoIterator<Item = Block<'a>>,
    x: f64,
    mut y: f64,
    width: f64,
) {
    for (index, block) in blocks.into_iter().enumerate() {
        if index > 0 {
            y += BLOCK_GAP;
        }
        for item in block.items {
            let height = item.height();
            items.push((
                Frame {
                    x,
                    y,
                    width,
                    height,
                },
                item,
            ));
            y += height;
        }
    }
}

type PlacedItems<'a> = Vec<(Frame, Item<'a>)>;

fn fit_columns<'a>(
    section: &'a ReportSection,
    metrics: &FontMetrics,
    top: f64,
    available: f64,
    style: LayoutStyle,
) -> ReportResult<Option<PlacedItems<'a>>> {
    let measure = |style| {
        section
            .blocks
            .iter()
            .map(|block| measure_block(block, metrics, style))
            .collect::<ReportResult<Vec<_>>>()
    };
    let blocks = measure(style)?;
    let occupied = blocks.iter().map(|b| b.height).sum::<f64>()
        + BLOCK_GAP * blocks.len().saturating_sub(1) as f64;
    if occupied <= available {
        let mut items = Vec::new();
        place(&mut items, blocks, LEFT, top, BODY_WIDTH);
        return Ok(Some(items));
    }
    if section.blocks.len() < 2 {
        return Ok(None);
    }
    let width = (BODY_WIDTH - COLUMN_GAP) / 2.0;
    let mut blocks = measure(style.with_width(width))?;
    // Each prefix charges a gap per block. Both columns have at least one
    // block, so remove the last gap from the taller column's height.
    let total: f64 = blocks.iter().map(|b| b.height + BLOCK_GAP).sum();
    let split = blocks
        .iter()
        .enumerate()
        .take(blocks.len() - 1)
        .scan(0.0, |left, (index, block)| {
            *left += block.height + BLOCK_GAP;
            Some((index + 1, left.max(total - *left) - BLOCK_GAP))
        })
        .filter(|(_, height)| *height <= available)
        .min_by(|a, b| a.1.total_cmp(&b.1));
    Ok(split.map(|(split, _)| {
        let right = blocks.split_off(split);
        let mut items = Vec::new();
        place(&mut items, blocks, LEFT, top, width);
        place(&mut items, right, LEFT + width + COLUMN_GAP, top, width);
        items
    }))
}

fn fit_grid<'a>(
    section: &'a ReportSection,
    metrics: &FontMetrics,
    mut top: f64,
    bottom: f64,
    style: LayoutStyle,
    columns: usize,
) -> ReportResult<Option<PlacedItems<'a>>> {
    let mut items = Vec::new();
    let mut blocks = section.blocks.iter().peekable();
    while let Some(block) = blocks.next() {
        if matches!(block, ReportBlock::Chart { .. }) {
            let mut charts = vec![block];
            while charts.len() < columns {
                let Some(chart) = blocks.next_if(|b| matches!(b, ReportBlock::Chart { .. })) else {
                    break;
                };
                charts.push(chart);
            }
            let width = (BODY_WIDTH - COLUMN_GAP * (columns - 1) as f64) / columns as f64;
            let mut row_height: f64 = 0.0;
            for (index, chart) in charts.into_iter().enumerate() {
                let chart = measure_block(chart, metrics, style.with_width(width))?;
                row_height = row_height.max(chart.height);
                place(
                    &mut items,
                    [chart],
                    LEFT + index as f64 * (width + COLUMN_GAP),
                    top,
                    width,
                );
            }
            top += row_height;
        } else {
            let block = measure_block(block, metrics, style)?;
            let height = block.height;
            place(&mut items, [block], LEFT, top, BODY_WIDTH);
            top += height;
        }
        if top > bottom {
            return Ok(None);
        }
        if blocks.peek().is_some() {
            top += BLOCK_GAP;
        }
    }
    Ok(Some(items))
}

fn fit<'a>(
    section: &'a ReportSection,
    metrics: &FontMetrics,
    top: f64,
    bottom: f64,
) -> ReportResult<(PlacedItems<'a>, Option<usize>)> {
    let available = bottom - top;
    for size in [12.0, 11.0, 10.0] {
        if let Some(items) = fit_columns(section, metrics, top, available, LayoutStyle::full(size))?
        {
            return Ok((items, None));
        }
    }
    let has_tables = section.blocks.iter().any(|block| {
        matches!(
            block,
            ReportBlock::Table { .. } | ReportBlock::AnchoredTable { .. }
        )
    });
    let chart_count = section
        .blocks
        .iter()
        .filter(|block| matches!(block, ReportBlock::Chart { .. }))
        .count();
    let preview_limits = [None, Some(6), Some(3), Some(1), Some(0)];
    for preview in preview_limits
        .into_iter()
        .filter(|preview| preview.is_none() || has_tables)
    {
        let style = LayoutStyle::compact(10.0, preview);
        for columns in (2..=chart_count.min(3)).rev() {
            if let Some(items) = fit_grid(section, metrics, top, bottom, style, columns)? {
                return Ok((items, preview));
            }
        }
        if let Some(items) = fit_columns(section, metrics, top, available, style)? {
            return Ok((items, preview));
        }
    }
    Err(overflow(section))
}

fn overflow(section: &ReportSection) -> ReportError {
    invalid(
        "sections",
        &format!(
            "section {:?} exceeds the editable one-slide layout after compact charts and table previews; split its content into additional sections",
            section.id
        ),
    )
}

pub(super) fn layout<'a>(
    document: &'a ReportDocument,
    metrics: &FontMetrics,
    max: usize,
    digest: &str,
) -> ReportResult<Vec<Slide<'a>>> {
    if document.sections.len() > max {
        return Err(ReportError::LimitExceeded(format!(
            "slides: section {:?} exceeds the slide limit",
            document.sections[max].id,
        )));
    }
    let context = measured_text(
        metrics,
        &period_comparison(document),
        11.0,
        BODY_WIDTH,
        None,
    )?;
    let footer_y = HEIGHT - 20.0 - context.height();
    let mut slides = Vec::with_capacity(document.sections.len());
    for section in &document.sections {
        let title = measured_text(metrics, &section.title, 28.0, BODY_WIDTH, None)?;
        let top = 24.0 + title.height();
        if footer_y - top < 64.0 {
            return Err(overflow(section));
        }
        let mut items = vec![(
            Frame {
                x: LEFT,
                y: 24.0,
                width: BODY_WIDTH,
                height: title.height(),
            },
            title,
        )];
        let (body, preview_rows) = fit(section, metrics, top + 10.0, footer_y - 10.0)?;
        items.extend(body);
        items.push((
            Frame {
                x: LEFT,
                y: footer_y,
                width: BODY_WIDTH,
                height: context.height(),
            },
            context.clone(),
        ));
        let mut notes = String::new();
        if slides.is_empty() {
            write_context(&mut notes, document, digest);
        }
        let mut anchors = Vec::new();
        let mut table_previews = Vec::new();
        for block in &section.blocks {
            match block {
                ReportBlock::AnchoredTable { table, row_ids } => {
                    anchors.extend(row_ids.iter().map(String::as_str));
                    if let Some(visible) = preview_rows.filter(|count| *count < table.rows.len()) {
                        table_previews.push(write_table_notes(
                            &mut notes,
                            table,
                            Some(row_ids),
                            visible,
                        ));
                    }
                }
                ReportBlock::Table { table } => {
                    if let Some(visible) = preview_rows.filter(|count| *count < table.rows.len()) {
                        table_previews.push(write_table_notes(&mut notes, table, None, visible));
                    }
                }
                ReportBlock::Chart { chart } => write_chart_notes(&mut notes, chart),
                _ => {}
            }
        }
        metrics.width(&notes, 1.0)?;
        slides.push(Slide {
            id: &section.id,
            items,
            notes,
            table_previews,
            row_ids: anchors,
        });
    }
    Ok(slides)
}

fn write_context(notes: &mut String, d: &ReportDocument, digest: &str) {
    writeln!(notes, "{}\n{}\nType: {}\nPeriod: {}\nScope: {}\nComparison: {}\nDocument: {}\nRevision: {}\nDocument SHA-256: {}\nEvidence SHA-256: {}\nTemplate: {}",
        d.title, d.subtitle, d.report_type, d.period, d.scope, d.comparison,
        d.document_id, d.revision, digest, d.provenance.evidence_sha256, SECTION_FIRST_TEMPLATE).unwrap();
    writeln!(notes, "\nDefinitions:").unwrap();
    for definition in &d.definitions {
        writeln!(notes, "{}: {}", definition.term, definition.meaning).unwrap();
    }
    writeln!(notes, "\nLimitations:").unwrap();
    for limit in &d.limitations {
        writeln!(notes, "{limit}").unwrap();
    }
    writeln!(notes, "\nSources:").unwrap();
    for source in &d.provenance.sources {
        writeln!(notes, "{} @ {}", source.id, source.revision).unwrap();
    }
}

fn write_table_notes(
    notes: &mut String,
    table: &ReportTable,
    row_ids: Option<&[String]>,
    visible: usize,
) -> String {
    let total = table.rows.len();
    let columns = table.headers.len();
    let mut record = format!("Complete table: {}\nRows: {total}\n", table.caption);
    for (index, row) in table.rows.iter().enumerate() {
        let number = index + 1;
        if let Some(ids) = row_ids {
            writeln!(record, "Row {number} ({})", ids[index]).unwrap();
        } else {
            writeln!(record, "Row {number}").unwrap();
        }
        for (header, cell) in table.headers.iter().zip(row) {
            writeln!(record, "{header}: {cell}").unwrap();
        }
    }
    // Match XML 1.0 line-ending normalization before binding the exact text.
    let record = record.replace("\r\n", "\n").replace('\r', "\n");
    let record = record.strip_suffix('\n').unwrap();
    notes.push('\n');
    let start = notes.bytes().filter(|byte| *byte == b'\n').count();
    let length = record.split('\n').count();
    notes.push_str(record);
    notes.push('\n');
    format!(
        "{TABLE_PREVIEW_PREFIX}{visible}:{total}:{columns}:{start}:{length}:{}",
        sha256(record.as_bytes())
    )
}

fn write_chart_notes(notes: &mut String, chart: &ReportChart) {
    writeln!(
        notes,
        "\n{}\n{}\n{}",
        chart.title,
        chart.description,
        chart_context(chart)
    )
    .unwrap();
    for (index, label) in chart.labels.iter().enumerate() {
        writeln!(notes, "{} · {label}", index + 1).unwrap();
        for series in &chart.series {
            let value =
                series.values[index].map_or_else(|| "Not measured".into(), |n| n.to_string());
            writeln!(notes, "{}: {value}", series.name).unwrap();
        }
    }
}
