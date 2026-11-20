use super::*;
use crate::{
    OfficeDocument,
    pptx::{PptxDocument, SlideSize},
    xlsx::XlsxDocument,
};
use layout::*;
use section::{Frame, Item, Slide};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    io::{Cursor, Read, Write},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
pub(super) const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
pub(super) const TABLE_GRAPHIC_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/table";
pub(super) const TABLE_PREVIEW_SHAPE_NAME: &str = "Report table preview";

struct ChartPart<'a> {
    slide: usize,
    number: usize,
    relation: String,
    chart: &'a ReportChart,
    show_categories: bool,
    native_legend: bool,
}

fn emu(v: f64) -> i64 {
    (v * 12700.0).round() as i64
}

pub(crate) fn render(
    d: &ReportDocument,
    pages: &[Page],
    font: &[u8],
    family: &str,
) -> ReportResult<Vec<u8>> {
    let mut deck = PptxDocument::new();
    deck.set_title(&d.title);
    deck.set_author("ThreatFlux report SDK");
    deck.set_slide_size(SlideSize::Widescreen16x9);
    let metrics = FontMetrics::new(font)?;
    let mut charts = Vec::new();
    let mut navigation = Vec::new();
    for (i, p) in pages.iter().enumerate() {
        let id = deck.add_slide("", &[])?;
        deck.add_slide_notes(&id, &p.notes)?;
        let mut shapes = String::new();
        shapes.push_str(&textbox(
            2,
            &metrics.wrap(&p.title, 22.0, BODY_WIDTH)?,
            LEFT,
            24.0,
            BODY_WIDTH,
            88.0,
            22.0,
            "172033",
        ));
        match &p.body {
            PageBody::Links(lines) => {
                for (n, link) in lines.iter().enumerate() {
                    let relation = format!("rIdNavigation{}", n + 1);
                    let shape=textbox(10+n as u32,&[link.text.clone()],LEFT,BODY_TOP-12.0+n as f64*LINE,BODY_WIDTH,LINE,12.0,"2563EB")
                        .replace("</a:rPr>",&format!("<a:hlinkClick r:id=\"{relation}\" action=\"ppaction://hlinksldjump\"/></a:rPr>"));
                    shapes.push_str(&shape);
                    let (target, _) = destination(pages, &link.target_id)?;
                    navigation.push((i + 1, relation, target + 1));
                }
            }
            PageBody::Text(lines) => {
                shapes.push_str(&textbox(
                    3,
                    lines,
                    LEFT,
                    BODY_TOP - 12.0,
                    BODY_WIDTH,
                    BODY_HEIGHT,
                    12.0,
                    "172033",
                ));
            }
            PageBody::Table { headers, rows } => shapes.push_str(&table_xml(headers, rows)),
            PageBody::Chart(c) => {
                let relation = "rIdChart".to_owned();
                shapes.push_str(&chart_frame(
                    3,
                    charts.len() + 1,
                    c,
                    &relation,
                    Frame {
                        x: LEFT,
                        y: BODY_TOP - 12.0,
                        width: BODY_WIDTH,
                        height: BODY_HEIGHT,
                    },
                ));
                charts.push(ChartPart {
                    slide: i + 1,
                    number: i + 1,
                    relation,
                    chart: c,
                    show_categories: true,
                    native_legend: true,
                });
            }
        }
        let footer = metrics.wrap(
            &format!(
                "{} / {} · {} · revision {}",
                i + 1,
                pages.len(),
                d.document_id,
                d.revision
            ),
            8.0,
            BODY_WIDTH,
        )?;
        if footer.len() > 2 {
            return Err(invalid(
                "document_id/revision",
                "slide footer must fit within two lines",
            ));
        }
        shapes.push_str(&textbox(
            4, &footer, LEFT, 500.0, BODY_WIDTH, 36.0, 8.0, "525252",
        ));
        deck.slides.insert(id, slide(&shapes));
    }
    finish(&deck, charts, navigation, family, None)
}

pub(super) fn render_sections(
    document: &ReportDocument,
    slides: &[Slide<'_>],
    family: &str,
    binding: &str,
) -> ReportResult<Vec<u8>> {
    let mut deck = PptxDocument::new();
    deck.set_title(&document.title);
    deck.set_author("ThreatFlux report SDK");
    deck.set_slide_size(SlideSize::Widescreen16x9);
    deck.set_keywords(binding);

    let mut targets = BTreeMap::new();
    for (index, slide) in slides.iter().enumerate() {
        for target in std::iter::once(slide.id).chain(slide.row_ids.iter().copied()) {
            targets.insert(target, index + 1);
        }
    }
    let mut charts = Vec::new();
    let mut navigation = Vec::new();
    for (index, slide_layout) in slides.iter().enumerate() {
        let slide_no = index + 1;
        let id = deck.add_slide("", &[])?;
        deck.add_slide_notes(&id, &slide_layout.notes)?;
        let mut shapes = String::new();
        let mut previews = slide_layout.table_previews.iter();
        for (item_index, (frame, item)) in slide_layout.items.iter().enumerate() {
            let shape = item_index as u32 + 2;
            match item {
                Item::Text {
                    lines,
                    size,
                    target,
                    color,
                } => {
                    let mut text = text_frame(
                        shape,
                        lines,
                        *frame,
                        *size,
                        color,
                        section::line_height(*size),
                        None,
                    );
                    if let Some(target) = target {
                        let relation = format!("rIdNavigation{shape}");
                        let dest = *targets
                            .get(target)
                            .ok_or_else(|| invalid("links.target_id", "target has no slide"))?;
                        text = text.replace("</a:rPr>", &format!(r#"<a:hlinkClick r:id="{relation}" action="ppaction://hlinksldjump"/></a:rPr>"#));
                        navigation.push((slide_no, relation, dest));
                    }
                    shapes.push_str(&text);
                }
                Item::Table(table) => {
                    shapes.push_str(&table_frame(
                        shape,
                        *frame,
                        &table.headers,
                        &table.rows,
                        &table.widths,
                        (table.size, section::line_height(table.size), table.inset),
                        |row| table.row_height(row),
                    ));
                }
                Item::TablePreview(lines) => {
                    let binding = previews
                        .next()
                        .ok_or_else(|| invalid("pptx", "missing table preview binding"))?;
                    shapes.push_str(&text_frame(
                        shape,
                        lines,
                        *frame,
                        section::TABLE_PREVIEW_SIZE,
                        "172033",
                        section::line_height(section::TABLE_PREVIEW_SIZE),
                        Some(binding),
                    ));
                }
                Item::Chart {
                    chart,
                    show_categories,
                    native_legend,
                    ..
                } => {
                    let number = charts.len() + 1;
                    let relation = format!("rIdChart{number}");
                    shapes.push_str(&chart_frame(shape, number, chart, &relation, *frame));
                    charts.push(ChartPart {
                        slide: slide_no,
                        number,
                        relation,
                        chart,
                        show_categories: *show_categories,
                        native_legend: *native_legend,
                    });
                }
            }
        }
        if previews.next().is_some() {
            return Err(invalid("pptx", "unused table preview binding"));
        }
        deck.slides.insert(id, slide(&shapes));
    }
    finish(&deck, charts, navigation, family, Some(slides))
}

fn finish(
    deck: &PptxDocument,
    charts: Vec<ChartPart<'_>>,
    navigation: Vec<(usize, String, usize)>,
    family: &str,
    sections: Option<&[Slide<'_>]>,
) -> ReportResult<Vec<u8>> {
    // Use the SDK's presentation, master and layout package for both templates.
    let mut parts = read_parts(&deck.to_bytes()?)?;
    for (source, relation, target) in navigation {
        append_slide_relationship(
            &mut parts,
            source,
            &relation,
            "slide",
            &format!("slide{target}.xml"),
        )?;
    }
    if let Some(sections) = sections {
        for (index, slide) in sections.iter().enumerate() {
            let path = format!("ppt/notesSlides/notesSlide{}.xml", index + 1);
            let data = parts
                .get_mut(&path)
                .ok_or_else(|| invalid("pptx", "missing notes"))?;
            let mut notes = std::str::from_utf8(data)
                .map_err(|_| invalid("pptx", "invalid notes encoding"))?
                .to_owned();
            let start = notes
                .find("<a:p>")
                .ok_or_else(|| invalid("pptx", "missing notes body"))?;
            let end = notes
                .find("</a:p>")
                .ok_or_else(|| invalid("pptx", "missing notes body"))?;
            let paragraphs = slide.notes.split('\n').map(|line| format!(
                r#"<a:p><a:r><a:rPr lang="en-US"/><a:t xml:space="preserve">{}</a:t></a:r></a:p>"#,
                xml(line)
            )).collect::<String>();
            notes.replace_range(start..end + "</a:p>".len(), &paragraphs);
            *data = notes.into_bytes();
        }
    }
    let mut types = String::from_utf8(parts.remove("[Content_Types].xml").unwrap())
        .map_err(|_| invalid("pptx", "content types encoding"))?;
    if let Some(sections) = sections {
        add_notes_master(&mut parts, sections.len())?;
        types = types.replace("</Types>", r#"<Override PartName="/ppt/notesMasters/notesMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"/></Types>"#);
    }
    if !charts.is_empty() {
        types=types.replace("</Types>","<Default Extension=\"xlsx\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\"/></Types>");
    }
    for part in charts {
        let ChartPart {
            slide,
            number,
            relation,
            chart,
            show_categories,
            native_legend,
        } = part;
        types=types.replace("</Types>",&format!("<Override PartName=\"/ppt/charts/chart{number}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.drawingml.chart+xml\"/></Types>"));
        append_slide_relationship(
            &mut parts,
            slide,
            &relation,
            "chart",
            &format!("../charts/chart{number}.xml"),
        )?;
        parts.insert(
            format!("ppt/charts/chart{number}.xml"),
            chart_xml(chart, show_categories, native_legend).into_bytes(),
        );
        parts.insert(format!("ppt/charts/_rels/chart{number}.xml.rels"),format!("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdWorkbook\" Type=\"{REL}/package\" Target=\"../embeddings/chart{number}.xlsx\"/></Relationships>").into_bytes());
        parts.insert(
            format!("ppt/embeddings/chart{number}.xlsx"),
            workbook(chart)?,
        );
    }
    parts.insert("[Content_Types].xml".into(), types.into_bytes());
    if let Some(app) = parts.get_mut("docProps/app.xml") {
        *app=format!("<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\"><Application>ThreatFlux report SDK</Application><PresentationFormat>On-screen Show (16:9)</PresentationFormat><Slides>{}</Slides><Notes>{}</Notes></Properties>",deck.slides.len(),deck.notes.len()).into_bytes();
    }
    for (name, data) in &mut parts {
        if name.ends_with(".xml") {
            let content = String::from_utf8_lossy(data).replace(
                "typeface=\"Noto Sans\"",
                &format!("typeface=\"{}\"", xml(family)),
            );
            *data = content.into_bytes();
        }
    }
    write_parts(parts)
}
fn add_notes_master(parts: &mut BTreeMap<String, Vec<u8>>, count: usize) -> ReportResult<()> {
    const PKG_REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    let master = r#"<p:notesMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
<p:sp><p:nvSpPr><p:cNvPr id="2" name="Speaker notes"/><p:cNvSpPr/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="914400"/><a:ext cx="5943600" cy="7315200"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody></p:sp>
</p:spTree></p:cSld>
<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
</p:notesMaster>"#;
    parts.insert(
        "ppt/notesMasters/notesMaster1.xml".into(),
        master.as_bytes().to_vec(),
    );
    parts.insert("ppt/notesMasters/_rels/notesMaster1.xml.rels".into(), format!(
        r#"<Relationships xmlns="{PKG_REL}"><Relationship Id="rIdTheme" Type="{REL}/theme" Target="../theme/theme1.xml"/></Relationships>"#
    ).into_bytes());
    replace_part(
        parts,
        "ppt/presentation.xml",
        "</p:sldMasterIdLst>",
        r#"</p:sldMasterIdLst><p:notesMasterIdLst><p:notesMasterId r:id="rIdNotesMaster"/></p:notesMasterIdLst>"#,
    )?;
    replace_part(
        parts,
        "ppt/_rels/presentation.xml.rels",
        "</Relationships>",
        &format!(
            r#"<Relationship Id="rIdNotesMaster" Type="{REL}/notesMaster" Target="notesMasters/notesMaster1.xml"/></Relationships>"#
        ),
    )?;
    let link = format!(
        r#"<Relationship Id="rIdNotesMaster" Type="{REL}/notesMaster" Target="../notesMasters/notesMaster1.xml"/></Relationships>"#
    );
    for n in 1..=count {
        replace_part(
            parts,
            &format!("ppt/notesSlides/_rels/notesSlide{n}.xml.rels"),
            "</Relationships>",
            &link,
        )?;
    }
    Ok(())
}

fn replace_part(
    parts: &mut BTreeMap<String, Vec<u8>>,
    path: &str,
    from: &str,
    to: &str,
) -> ReportResult<()> {
    let bytes = parts
        .get_mut(path)
        .ok_or_else(|| invalid("pptx", "missing generated XML part"))?;
    let content =
        std::str::from_utf8(bytes).map_err(|_| invalid("pptx", "invalid generated XML"))?;
    if !content.contains(from) {
        return Err(invalid("pptx", "missing generated XML element"));
    }
    *bytes = content.replacen(from, to, 1).into_bytes();
    Ok(())
}

fn append_slide_relationship(
    parts: &mut BTreeMap<String, Vec<u8>>,
    source: usize,
    id: &str,
    kind: &str,
    target: &str,
) -> ReportResult<()> {
    let path = format!("ppt/slides/_rels/slide{source}.xml.rels");
    replace_part(
        parts,
        &path,
        "</Relationships>",
        &format!(
            r#"<Relationship Id="{id}" Type="{REL}/{kind}" Target="{target}"/></Relationships>"#
        ),
    )
}

fn chart_frame(
    id: u32,
    number: usize,
    chart: &ReportChart,
    relation: &str,
    frame: Frame,
) -> String {
    format!(
        r#"<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{id}" name="Editable chart {number}" descr="{}"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></p:xfrm><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="{REL}" r:id="{relation}"/></a:graphicData></a:graphic></p:graphicFrame>"#,
        xml(&chart.description),
        emu(frame.x),
        emu(frame.y),
        emu(frame.width),
        emu(frame.height)
    )
}

fn slide(shapes: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"{REL}\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>{shapes}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"
    )
}
fn paragraphs(lines: &[String], size: f64, color: &str, spacing: f64) -> String {
    lines.iter().map(|l|format!("<a:p><a:pPr marL=\"0\" indent=\"0\"><a:lnSpc><a:spcPts val=\"{}\"/></a:lnSpc></a:pPr><a:r><a:rPr lang=\"en-US\" sz=\"{}\"><a:solidFill><a:srgbClr val=\"{color}\"/></a:solidFill><a:latin typeface=\"Noto Sans\"/></a:rPr><a:t xml:space=\"preserve\">{}</a:t></a:r><a:endParaRPr sz=\"{}\"/></a:p>",(spacing * 100.0).round() as u32,(size*100.0)as u32,xml(l),(size*100.0)as u32)).collect()
}
fn textbox(
    id: u32,
    lines: &[String],
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    size: f64,
    color: &str,
) -> String {
    text_frame(
        id,
        lines,
        Frame {
            x,
            y,
            width: w,
            height: h,
        },
        size,
        color,
        if size <= 12.0 { 16.0 } else { 26.0 },
        None,
    )
}

fn text_frame(
    id: u32,
    lines: &[String],
    frame: Frame,
    size: f64,
    color: &str,
    spacing: f64,
    preview_binding: Option<&str>,
) -> String {
    let metadata = match preview_binding {
        Some(binding) => format!(
            r#"name="{TABLE_PREVIEW_SHAPE_NAME}" descr="{}""#,
            xml(binding)
        ),
        None => format!(r#"name="Report text {id}""#),
    };
    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" {metadata}/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/><a:ln><a:noFill/></a:ln></p:spPr><p:txBody><a:bodyPr wrap=\"none\" lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\"/><a:lstStyle/>{}</p:txBody></p:sp>",
        emu(frame.x),
        emu(frame.y),
        emu(frame.width),
        emu(frame.height),
        paragraphs(lines, size, color, spacing)
    )
}
fn table_xml(headers: &[Vec<String>], rows: &[Vec<Vec<String>>]) -> String {
    let width = BODY_WIDTH / headers.len() as f64;
    let height = row_height(headers) + rows.iter().map(|r| row_height(r)).sum::<f64>();
    table_frame(
        3,
        Frame {
            x: LEFT,
            y: BODY_TOP - 12.0,
            width: BODY_WIDTH,
            height,
        },
        headers,
        rows,
        &vec![width; headers.len()],
        (11.0, 16.0, 8.0),
        row_height,
    )
}

fn table_frame(
    id: u32,
    frame: Frame,
    headers: &[Vec<String>],
    rows: &[Vec<Vec<String>>],
    widths: &[f64],
    (size, spacing, inset): (f64, f64, f64),
    height: impl Fn(&[Vec<String>]) -> f64,
) -> String {
    let vertical_inset = if inset == 8.0 { 6.0 } else { inset };
    let mut s = format!(
        "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"{id}\" name=\"Editable report table\"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></p:xfrm><a:graphic><a:graphicData uri=\"{TABLE_GRAPHIC_URI}\"><a:tbl><a:tblPr firstRow=\"1\" bandRow=\"1\"/><a:tblGrid>",
        emu(frame.x),
        emu(frame.y),
        emu(frame.width),
        emu(frame.height)
    );
    for width in widths {
        write!(s, "<a:gridCol w=\"{}\"/>", emu(*width)).unwrap();
    }
    s.push_str("</a:tblGrid>");
    for (n, row) in std::iter::once(headers)
        .chain(rows.iter().map(Vec::as_slice))
        .enumerate()
    {
        write!(s, "<a:tr h=\"{}\">", emu(height(row))).unwrap();
        for cell in row {
            write!(s,"<a:tc><a:txBody><a:bodyPr lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\"/><a:lstStyle/>{}</a:txBody><a:tcPr marL=\"{h}\" marR=\"{h}\" marT=\"{v}\" marB=\"{v}\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:tcPr></a:tc>",paragraphs(cell,size,"172033",spacing),if n==0{"E8EEFB"}else if n%2==0{"F1F5F9"}else{"FFFFFF"}, h=emu(inset), v=emu(vertical_inset)).unwrap();
        }
        s.push_str("</a:tr>");
    }
    s.push_str("</a:tbl></a:graphicData></a:graphic></p:graphicFrame>");
    s
}
fn chart_series(c: &ReportChart) -> Vec<ReportSeries> {
    let mut v = c.series.clone();
    if let Some(t) = c.target {
        v.push(ReportSeries {
            name: format!("Target {t}"),
            values: vec![Some(t); c.labels.len()],
        });
    }
    v
}
fn chart_xml(c: &ReportChart, show_categories: bool, native_legend: bool) -> String {
    let legend = if native_legend {
        r#"<c:legend><c:legendPos val="b"/><c:layout/><c:overlay val="0"/></c:legend>"#
    } else {
        ""
    };
    let label_position = if show_categories { "low" } else { "none" };
    let series = chart_series(c);
    let mut s = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"{REL}\"><c:date1904 val=\"0\"/><c:lang val=\"en-US\"/><c:chart><c:autoTitleDeleted val=\"1\"/><c:plotArea><c:layout/><c:lineChart><c:grouping val=\"standard\"/>"
    );
    for (n, series) in series.iter().enumerate() {
        let col = (b'B' + n as u8) as char;
        write!(s,"<c:ser><c:idx val=\"{n}\"/><c:order val=\"{n}\"/><c:tx><c:strRef><c:f>Sheet1!${col}$1</c:f><c:strCache><c:ptCount val=\"1\"/><c:pt idx=\"0\"><c:v>{}</c:v></c:pt></c:strCache></c:strRef></c:tx><c:spPr><a:ln w=\"25400\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:ln></c:spPr><c:marker><c:symbol val=\"circle\"/><c:size val=\"5\"/></c:marker><c:cat><c:strRef><c:f>Sheet1!$A$2:$A${}</c:f><c:strCache><c:ptCount val=\"{}\"/>",xml(&series.name),if n<c.series.len(){COLORS[n%8]}else{"172033"},c.labels.len()+1,c.labels.len()).unwrap();
        for (i, l) in c.labels.iter().enumerate() {
            write!(s, "<c:pt idx=\"{i}\"><c:v>{}</c:v></c:pt>", xml(l)).unwrap();
        }
        write!(s,"</c:strCache></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!${col}$2:${col}${}</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val=\"{}\"/>",c.labels.len()+1,c.labels.len()).unwrap();
        for (i, v) in series.values.iter().enumerate() {
            if let Some(v) = v {
                write!(s, "<c:pt idx=\"{i}\"><c:v>{v}</c:v></c:pt>").unwrap();
            }
        }
        s.push_str("</c:numCache></c:numRef></c:val><c:smooth val=\"0\"/></c:ser>");
    }
    write!(s,"<c:axId val=\"100\"/><c:axId val=\"200\"/></c:lineChart><c:catAx><c:axId val=\"100\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"b\"/><c:tickLblPos val=\"{label_position}\"/><c:crossAx val=\"200\"/><c:crosses val=\"autoZero\"/><c:auto val=\"1\"/><c:lblAlgn val=\"ctr\"/><c:lblOffset val=\"100\"/><c:tickLblSkip val=\"{}\"/></c:catAx><c:valAx><c:axId val=\"200\"/><c:scaling><c:orientation val=\"minMax\"/><c:max val=\"{}\"/><c:min val=\"{}\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"l\"/><c:majorGridlines/><c:numFmt formatCode=\"General\" sourceLinked=\"0\"/><c:tickLblPos val=\"nextTo\"/><c:crossAx val=\"100\"/><c:crosses val=\"autoZero\"/><c:crossBetween val=\"between\"/></c:valAx></c:plotArea>{legend}<c:plotVisOnly val=\"1\"/><c:dispBlanksAs val=\"gap\"/></c:chart><c:externalData r:id=\"rIdWorkbook\"><c:autoUpdate val=\"0\"/></c:externalData></c:chartSpace>",(c.labels.len()/10).max(1),c.maximum,c.minimum).unwrap();
    s
}
fn workbook(c: &ReportChart) -> ReportResult<Vec<u8>> {
    let series = chart_series(c);
    let mut s = String::from(
        "<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData><row r=\"1\">",
    );
    s.push_str(&string_cell("A1", "Observation"));
    for (i, v) in series.iter().enumerate() {
        s.push_str(&string_cell(
            &format!("{}1", (b'B' + i as u8) as char),
            &v.name,
        ));
    }
    s.push_str("</row>");
    for (i, label) in c.labels.iter().enumerate() {
        let row = i + 2;
        write!(
            s,
            "<row r=\"{row}\">{}",
            string_cell(&format!("A{row}"), label)
        )
        .unwrap();
        for (n, series) in series.iter().enumerate() {
            if let Some(v) = series.values[i] {
                write!(
                    s,
                    "<c r=\"{}{row}\"><v>{v}</v></c>",
                    (b'B' + n as u8) as char
                )
                .unwrap();
            }
        }
        s.push_str("</row>");
    }
    s.push_str("</sheetData></worksheet>");
    let mut workbook = XlsxDocument::new();
    workbook.worksheets.insert("sheet1".into(), s);
    Ok(workbook.to_bytes()?)
}
fn string_cell(id: &str, value: &str) -> String {
    format!(
        "<c r=\"{id}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
        xml(value)
    )
}
fn read_parts(bytes: &[u8]) -> ReportResult<BTreeMap<String, Vec<u8>>> {
    let mut z = ZipArchive::new(Cursor::new(bytes))?;
    let mut parts = BTreeMap::new();
    for i in 0..z.len() {
        let mut entry = z.by_index(i)?;
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        parts.insert(entry.name()?.into_owned(), data);
    }
    Ok(parts)
}
fn write_parts(parts: BTreeMap<String, Vec<u8>>) -> ReportResult<Vec<u8>> {
    let mut z = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in parts {
        z.start_file(
            name,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
        )?;
        z.write_all(&data)?;
    }
    Ok(z.finish()?.into_inner())
}
