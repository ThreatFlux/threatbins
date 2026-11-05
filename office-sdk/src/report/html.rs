use super::*;
use base64::Engine;
use layout::{COLORS, chart_context};
use std::fmt::Write;
fn e(s: &str) -> String {
    xml(s)
}
pub(crate) fn render(d: &ReportDocument, digest: &str, font: &[u8]) -> String {
    let mut h = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src data:; font-src data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>{}</style></head><body><main><header><p class=\"eyebrow\">Report · {}</p><h1>{}</h1><p>{}</p><dl><dt>Period</dt><dd>{}</dd><dt>Scope</dt><dd>{}</dd><dt>Comparison</dt><dd>{}</dd></dl></header>",
        e(&d.title),
        &format!(
            "{}@font-face{{font-family:Report;src:url(data:font/ttf;base64,{})}}body{{font-family:Report,system-ui,sans-serif}}",
            CSS,
            base64::engine::general_purpose::STANDARD.encode(font)
        ),
        e(&d.report_type),
        e(&d.title),
        e(&d.subtitle),
        e(&d.period),
        e(&d.scope),
        e(&d.comparison)
    );
    for section in &d.sections {
        write!(
            h,
            "<section id=\"{}\"><h2>{}</h2>",
            e(&section.id),
            e(&section.title)
        )
        .unwrap();
        for block in &section.blocks {
            match block {
                ReportBlock::Paragraph { text } => {
                    write!(h, "<p class=\"prose\">{}</p>", e(text)).unwrap();
                }
                ReportBlock::Table { table } => table_html(&mut h, table),
                ReportBlock::AnchoredTable { table, row_ids } => {
                    table_html_anchors(&mut h, table, Some(row_ids))
                }
                ReportBlock::Links { links } => {
                    h.push_str("<nav aria-label=\"Related report content\"><ul>");
                    for link in links {
                        write!(
                            h,
                            "<li><a href=\"#{}\">{}</a></li>",
                            e(&link.target_id),
                            e(&link.label)
                        )
                        .unwrap();
                    }
                    h.push_str("</ul></nav>");
                }
                ReportBlock::Chart { chart: c } => {
                    write!(
                        h,
                        "<figure><figcaption><h3>{}</h3><p>{}</p><p>{}</p></figcaption>",
                        e(&c.title),
                        e(&c.description),
                        e(&chart_context(c))
                    )
                    .unwrap();
                    h.push_str(&chart_svg(c));
                    h.push_str("</figure>");
                    let mut headers = vec!["Observation".into()];
                    headers.extend(c.series.iter().map(|s| s.name.clone()));
                    let rows = c
                        .labels
                        .iter()
                        .enumerate()
                        .map(|(i, l)| {
                            let mut r = vec![l.clone()];
                            r.extend(c.series.iter().map(|s| {
                                s.values[i].map_or_else(|| "Not measured".into(), |v| v.to_string())
                            }));
                            r
                        })
                        .collect();
                    table_html(
                        &mut h,
                        &ReportTable {
                            caption: "Chart data".into(),
                            headers,
                            rows,
                        },
                    );
                }
            }
        }
        h.push_str("</section>");
    }
    h.push_str("<section><h2>Definitions</h2><dl>");
    for v in &d.definitions {
        write!(h, "<dt>{}</dt><dd>{}</dd>", e(&v.term), e(&v.meaning)).unwrap();
    }
    h.push_str("</dl></section><section><h2>Limitations</h2>");
    for l in &d.limitations {
        write!(h, "<p>{}</p>", e(l)).unwrap();
    }
    write!(h,"</section><footer><h2>Provenance</h2><p>Document: {} · Revision: {}</p><p>Document SHA-256: {}</p><p>Evidence SHA-256: {}</p><p>Template: {}</p><ul>",e(&d.document_id),e(&d.revision),digest,e(&d.provenance.evidence_sha256),REPORT_TEMPLATE).unwrap();
    for s in &d.provenance.sources {
        write!(h, "<li>{} @ {}</li>", e(&s.id), e(&s.revision)).unwrap();
    }
    h.push_str("</ul></footer></main></body></html>");
    h
}
fn table_html(h: &mut String, t: &ReportTable) {
    table_html_anchors(h, t, None);
}
fn table_html_anchors(h: &mut String, t: &ReportTable, row_ids: Option<&[String]>) {
    h.push_str("<div class=\"table-scroll\"><table>");
    write!(h, "<caption>{}</caption><thead><tr>", e(&t.caption)).unwrap();
    for x in &t.headers {
        write!(h, "<th scope=\"col\">{}</th>", e(x)).unwrap();
    }
    h.push_str("</tr></thead><tbody>");
    for (index, row) in t.rows.iter().enumerate() {
        if let Some(id) = row_ids.and_then(|ids| ids.get(index)) {
            write!(h, "<tr id=\"{}\">", e(id)).unwrap();
        } else {
            h.push_str("<tr>");
        }
        for x in row {
            write!(h, "<td>{}</td>", e(x)).unwrap();
        }
        h.push_str("</tr>");
    }
    h.push_str("</tbody></table></div>");
}
fn chart_svg(c: &ReportChart) -> String {
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 880 340\" role=\"img\"><title>{}</title><desc>{}</desc>",
        e(&c.title),
        e(&c.description)
    );
    let y = |v: f64| 290.0 - (v - c.minimum) / (c.maximum - c.minimum) * 260.0;
    let x = |i: usize| 60.0 + i as f64 / (c.labels.len().saturating_sub(1).max(1)) as f64 * 780.0;
    for (i, b) in c.bands.iter().enumerate() {
        let end = c.bands.get(i + 1).map_or(c.maximum, |b| b.start);
        write!(
            s,
            "<rect x=\"60\" y=\"{}\" width=\"780\" height=\"{}\" fill=\"#{}\" opacity=\"0.08\"/>",
            y(end),
            y(b.start) - y(end),
            COLORS[i % 8]
        )
        .unwrap();
    }
    for i in 0..=4 {
        let v = c.minimum + (c.maximum - c.minimum) * i as f64 / 4.0;
        write!(s,"<path d=\"M60 {}H840\" stroke=\"#CBD5E1\"/><text x=\"2\" y=\"{}\" font-size=\"12\">{}</text>",y(v),y(v)+4.0,v).unwrap();
    }
    if let Some(t) = c.target {
        write!(s,"<path d=\"M60 {}H840\" stroke=\"#111827\" stroke-dasharray=\"6 4\"/><text x=\"60\" y=\"{}\" font-size=\"12\">Target {}</text>",y(t),y(t)-4.0,t).unwrap();
    }
    for (n, series) in c.series.iter().enumerate() {
        let mut last = None;
        for (i, v) in series.values.iter().enumerate() {
            if let Some(v) = v {
                let current = (x(i), y(*v));
                if let Some((lx, ly)) = last {
                    write!(s,"<path d=\"M{} {}L{} {}\" stroke=\"#{}\" stroke-width=\"2\" fill=\"none\"/>",lx,ly,current.0,current.1,COLORS[n%8]).unwrap();
                }
                write!(s,"<circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"#{}\"><title>{} · {}: {}</title></circle>",current.0,current.1,COLORS[n%8],e(&series.name),e(&c.labels[i]),v).unwrap();
                last = Some(current);
            } else {
                last = None;
            }
        }
    }
    s.push_str("</svg>");
    s
}
const CSS: &str = "*{box-sizing:border-box}body{margin:0;background:#f1f5f9;color:#172033;font:16px/1.6 system-ui,sans-serif}main{max-width:1100px;margin:32px auto;background:white;padding:48px}h1{font-size:36px;line-height:1.15}h2{font-size:25px;border-bottom:2px solid #2563eb;padding-bottom:8px}h3{font-size:19px}.eyebrow{font-weight:700;color:#1d4ed8}section{margin-top:36px}p,dd,td,th,li{overflow-wrap:anywhere;white-space:pre-wrap}dt{font-weight:700}dd{margin:0 0 12px}.prose{white-space:pre-wrap}table{width:100%;border-collapse:collapse;table-layout:fixed}caption{text-align:left;font-weight:700;padding:12px 0}th,td{text-align:left;vertical-align:top;border:1px solid #cbd5e1;padding:10px}th{background:#e8eefb}tbody tr:nth-child(even){background:#f8fafc}.table-scroll{overflow:auto}figure{margin:24px 0}svg{width:100%;height:auto}footer{font-size:12px;border-top:1px solid #cbd5e1;margin-top:36px;padding-top:20px}@media(max-width:600px){main{margin:0;padding:20px}h1{font-size:28px}th,td{padding:6px;font-size:13px}}@media print{body{background:white}main{padding:0;margin:0;max-width:none}thead{display:table-header-group}tr{break-inside:avoid}h2,h3,caption{break-after:avoid}svg{max-height:160mm}footer{break-inside:avoid}.table-scroll{overflow:visible}@page{size:A4;margin:18mm}}";
