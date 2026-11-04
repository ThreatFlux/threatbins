use super::*;
use layout::*;
use printpdf::{
    Color, Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Polygon, PolygonRing, Pt, Rgb, TextItem, WindingOrder,
};

pub(crate) fn render(d: &ReportDocument, pages: &[Page], font: &[u8]) -> ReportResult<Vec<u8>> {
    let parsed = ParsedFont::from_bytes(font, 0, &mut Vec::new())
        .ok_or_else(|| invalid("font_data", "font cannot be embedded in PDF"))?;
    let mut pdf = PdfDocument::new(&d.title);
    let handle = PdfFontHandle::External(pdf.add_font(&parsed));
    let metrics = FontMetrics::new(font)?;
    let mut result = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        let mut ops = Vec::new();
        rect(&mut ops, 0.0, 0.0, WIDTH, 8.0, "2563EB");
        for (n, line) in metrics
            .wrap(&page.title, 22.0, BODY_WIDTH)?
            .iter()
            .enumerate()
        {
            text(
                &mut ops,
                &handle,
                line,
                LEFT,
                42.0 + n as f64 * 26.0,
                22.0,
                "172033",
            );
        }
        match &page.body {
            PageBody::Text(lines) => {
                for (n, l) in lines.iter().enumerate() {
                    text(
                        &mut ops,
                        &handle,
                        l,
                        LEFT,
                        BODY_TOP + n as f64 * LINE,
                        12.0,
                        "172033",
                    );
                }
            }
            PageBody::Table { headers, rows } => {
                let cw = BODY_WIDTH / headers.len() as f64;
                let mut y = BODY_TOP - 12.0;
                for (n, row) in std::iter::once(headers).chain(rows.iter()).enumerate() {
                    let height = row_height(row);
                    rect(
                        &mut ops,
                        LEFT,
                        y,
                        BODY_WIDTH,
                        height,
                        if n == 0 {
                            "E8EEFB"
                        } else if n % 2 == 0 {
                            "F1F5F9"
                        } else {
                            "FFFFFF"
                        },
                    );
                    for (col, cell) in row.iter().enumerate() {
                        for (line_no, l) in cell.iter().enumerate() {
                            text(
                                &mut ops,
                                &handle,
                                l,
                                LEFT + col as f64 * cw + 8.0,
                                y + 18.0 + line_no as f64 * LINE,
                                11.0,
                                "172033",
                            );
                        }
                        line(
                            &mut ops,
                            LEFT + col as f64 * cw,
                            y,
                            LEFT + col as f64 * cw,
                            y + height,
                            "CBD5E1",
                        );
                    }
                    y += height;
                    line(&mut ops, LEFT, y, LEFT + BODY_WIDTH, y, "CBD5E1");
                }
            }
            PageBody::Chart(c) => chart(&mut ops, &handle, c, &metrics)?,
        }
        let footer = format!(
            "{} / {}  ·  {}  ·  revision {}",
            i + 1,
            pages.len(),
            d.document_id,
            d.revision
        );
        // The complete IDs are also in the provenance pages. Reject enormous
        // footer fields rather than silently truncate their visible binding.
        let footer_lines = metrics.wrap(&footer, 8.0, BODY_WIDTH)?;
        if footer_lines.len() > 3 {
            return Err(invalid(
                "document_id/revision",
                "page footer must fit within three lines",
            ));
        }
        for (n, l) in footer_lines.iter().enumerate() {
            text(
                &mut ops,
                &handle,
                l,
                LEFT,
                505.0 + n as f64 * 10.0,
                8.0,
                "525252",
            );
        }
        result.push(PdfPage::new(
            Mm((WIDTH * 25.4 / 72.0) as f32),
            Mm((HEIGHT * 25.4 / 72.0) as f32),
            ops,
        ));
    }
    let mut warnings = Vec::new();
    let bytes = pdf
        .with_pages(result)
        .save(&PdfSaveOptions::default(), &mut warnings);
    if bytes.is_empty() {
        return Err(invalid("pdf", "empty serialization"));
    }
    Ok(bytes)
}
fn point(x: f64, y: f64) -> Point {
    Point {
        x: Pt(x as f32),
        y: Pt((HEIGHT - y) as f32),
    }
}
fn color(hex: &str) -> Color {
    let v = u32::from_str_radix(hex, 16).unwrap_or(0);
    Color::Rgb(Rgb::new(
        ((v >> 16) & 255) as f32 / 255.0,
        ((v >> 8) & 255) as f32 / 255.0,
        (v & 255) as f32 / 255.0,
        None,
    ))
}
fn text(ops: &mut Vec<Op>, font: &PdfFontHandle, s: &str, x: f64, y: f64, size: f64, hex: &str) {
    ops.extend([
        Op::StartTextSection,
        Op::SetTextCursor { pos: point(x, y) },
        Op::SetFont {
            font: font.clone(),
            size: Pt(size as f32),
        },
        Op::SetFillColor { col: color(hex) },
        Op::ShowText {
            items: vec![TextItem::Text(s.into())],
        },
        Op::EndTextSection,
    ]);
}
fn line(ops: &mut Vec<Op>, x1: f64, y1: f64, x2: f64, y2: f64, hex: &str) {
    ops.extend([
        Op::SetOutlineColor { col: color(hex) },
        Op::SetOutlineThickness { pt: Pt(1.0) },
        Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: point(x1, y1),
                        bezier: false,
                    },
                    LinePoint {
                        p: point(x2, y2),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        },
    ]);
}
fn rect(ops: &mut Vec<Op>, x: f64, y: f64, w: f64, h: f64, hex: &str) {
    ops.extend([
        Op::SetFillColor { col: color(hex) },
        Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing {
                    points: vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
                        .into_iter()
                        .map(|(x, y)| LinePoint {
                            p: point(x, y),
                            bezier: false,
                        })
                        .collect(),
                }],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::NonZero,
            },
        },
    ]);
}
fn chart(
    ops: &mut Vec<Op>,
    font: &PdfFontHandle,
    c: &ReportChart,
    metrics: &FontMetrics,
) -> ReportResult<()> {
    // A measured two-column color key identifies every plotted series on this
    // page; the following data table supplies its complete observations.
    let mut legend_y = BODY_TOP;
    for (row, series) in c.series.chunks(2).enumerate() {
        let labels = series
            .iter()
            .map(|s| metrics.wrap(&s.name, 10.0, BODY_WIDTH / 2.0 - 36.0))
            .collect::<ReportResult<Vec<_>>>()?;
        let height = labels.iter().map(Vec::len).max().unwrap_or(1) as f64 * 12.0 + 6.0;
        if legend_y + height > 270.0 {
            return Err(invalid(
                "chart.series.name",
                "PDF chart legend exceeds the measured area",
            ));
        }
        for (col, lines) in labels.iter().enumerate() {
            let left = LEFT + col as f64 * BODY_WIDTH / 2.0;
            let color = COLORS[(row * 2 + col) % COLORS.len()];
            rect(ops, left, legend_y - 7.0, 16.0, 4.0, color);
            for (line_no, label) in lines.iter().enumerate() {
                text(
                    ops,
                    font,
                    label,
                    left + 24.0,
                    legend_y + line_no as f64 * 12.0,
                    10.0,
                    "172033",
                );
            }
        }
        legend_y += height;
    }
    let plot_top = legend_y + 16.0;
    let x = |i: usize| 100.0 + i as f64 / c.labels.len().saturating_sub(1).max(1) as f64 * 780.0;
    let y = |v: f64| 440.0 - (v - c.minimum) / (c.maximum - c.minimum) * (440.0 - plot_top);
    for i in 0..=4 {
        let v = c.minimum + (c.maximum - c.minimum) * i as f64 / 4.0;
        line(ops, 100.0, y(v), 880.0, y(v), "CBD5E1");
        text(ops, font, &v.to_string(), 40.0, y(v) + 4.0, 10.0, "525252");
    }
    if let Some(t) = c.target {
        line(ops, 100.0, y(t), 880.0, y(t), "172033");
        text(
            ops,
            font,
            &format!("Target {t}"),
            100.0,
            y(t) - 8.0,
            10.0,
            "172033",
        );
    }
    for (n, s) in c.series.iter().enumerate() {
        let mut last = None;
        for (i, v) in s.values.iter().enumerate() {
            if let Some(v) = v {
                let pos = (x(i), y(*v));
                if let Some((px, py)) = last {
                    line(ops, px, py, pos.0, pos.1, COLORS[n % 8]);
                }
                rect(ops, pos.0 - 2.0, pos.1 - 2.0, 4.0, 4.0, COLORS[n % 8]);
                last = Some(pos);
            } else {
                last = None;
            }
        }
    }
    for i in (0..c.labels.len()).step_by((c.labels.len() / 10).max(1)) {
        text(ops, font, &(i + 1).to_string(), x(i), 462.0, 10.0, "525252");
    }
    text(
        ops,
        font,
        "Observation number · full names, series and values in the following chart-data table",
        40.0,
        485.0,
        10.0,
        "525252",
    );
    Ok(())
}
