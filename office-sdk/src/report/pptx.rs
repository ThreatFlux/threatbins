use super::*;
use crate::{
    OfficeDocument,
    pptx::{PptxDocument, SlideSize},
    xlsx::XlsxDocument,
};
use layout::*;
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    io::{Cursor, Read, Write},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
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
                let n = charts.len() + 1;
                shapes.push_str(&format!("<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"3\" name=\"Editable chart {}\" descr=\"{}\"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></p:xfrm><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:r=\"{REL}\" r:id=\"rIdChart\"/></a:graphicData></a:graphic></p:graphicFrame>",n,xml(&c.description),emu(LEFT),emu(BODY_TOP-12.0),emu(BODY_WIDTH),emu(BODY_HEIGHT)));
                charts.push((i + 1, c));
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
    // Reuse the existing SDK's presentation/master/layout/notes packaging; add
    // native report parts to that package rather than rasterizing the slides.
    let mut parts = read_parts(&deck.to_bytes()?)?;
    let mut types = String::from_utf8(parts.remove("[Content_Types].xml").unwrap())
        .map_err(|_| invalid("pptx", "content types encoding"))?;
    if !charts.is_empty() {
        types=types.replace("</Types>","<Default Extension=\"xlsx\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\"/></Types>");
    }
    for (slide_no, c) in charts {
        let chart_no = slide_no;
        types=types.replace("</Types>",&format!("<Override PartName=\"/ppt/charts/chart{chart_no}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.drawingml.chart+xml\"/></Types>"));
        let path = format!("ppt/slides/_rels/slide{slide_no}.xml.rels");
        let rels=String::from_utf8(parts.remove(&path).unwrap()).map_err(|_|invalid("pptx","slide relationships encoding"))?.replace("</Relationships>",&format!("<Relationship Id=\"rIdChart\" Type=\"{REL}/chart\" Target=\"../charts/chart{chart_no}.xml\"/></Relationships>"));
        parts.insert(path, rels.into_bytes());
        parts.insert(
            format!("ppt/charts/chart{chart_no}.xml"),
            chart_xml(c).into_bytes(),
        );
        parts.insert(format!("ppt/charts/_rels/chart{chart_no}.xml.rels"),format!("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdWorkbook\" Type=\"{REL}/package\" Target=\"../embeddings/chart{chart_no}.xlsx\"/></Relationships>").into_bytes());
        parts.insert(format!("ppt/embeddings/chart{chart_no}.xlsx"), workbook(c)?);
    }
    parts.insert("[Content_Types].xml".into(), types.into_bytes());
    if let Some(app) = parts.get_mut("docProps/app.xml") {
        *app=format!("<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\"><Application>ThreatFlux report SDK</Application><PresentationFormat>On-screen Show (16:9)</PresentationFormat><Slides>{}</Slides><Notes>{}</Notes></Properties>",pages.len(),pages.len()).into_bytes();
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
fn slide(shapes: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"{REL}\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>{shapes}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"
    )
}
fn paragraphs(lines: &[String], size: f64, color: &str) -> String {
    lines.iter().map(|l|format!("<a:p><a:pPr marL=\"0\" indent=\"0\"><a:lnSpc><a:spcPts val=\"{}\"/></a:lnSpc></a:pPr><a:r><a:rPr lang=\"en-US\" sz=\"{}\"><a:solidFill><a:srgbClr val=\"{color}\"/></a:solidFill><a:latin typeface=\"Noto Sans\"/></a:rPr><a:t xml:space=\"preserve\">{}</a:t></a:r><a:endParaRPr sz=\"{}\"/></a:p>",if size<=12.0{1600}else{2600},(size*100.0)as u32,xml(l),(size*100.0)as u32)).collect()
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
    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"Report text {id}\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/><a:ln><a:noFill/></a:ln></p:spPr><p:txBody><a:bodyPr wrap=\"none\" lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\"/><a:lstStyle/>{}</p:txBody></p:sp>",
        emu(x),
        emu(y),
        emu(w),
        emu(h),
        paragraphs(lines, size, color)
    )
}
fn table_xml(headers: &[Vec<String>], rows: &[Vec<Vec<String>>]) -> String {
    let width = BODY_WIDTH / headers.len() as f64;
    let height = row_height(headers) + rows.iter().map(|r| row_height(r)).sum::<f64>();
    let mut s = format!(
        "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"3\" name=\"Editable report table\"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></p:xfrm><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\"><a:tbl><a:tblPr firstRow=\"1\" bandRow=\"1\"/><a:tblGrid>",
        emu(LEFT),
        emu(BODY_TOP - 12.0),
        emu(BODY_WIDTH),
        emu(height)
    );
    for _ in headers {
        s.push_str(&format!("<a:gridCol w=\"{}\"/>", emu(width)));
    }
    s.push_str("</a:tblGrid>");
    for (n, row) in std::iter::once(headers)
        .chain(rows.iter().map(Vec::as_slice))
        .enumerate()
    {
        write!(s, "<a:tr h=\"{}\">", emu(row_height(row))).unwrap();
        for cell in row {
            write!(s,"<a:tc><a:txBody><a:bodyPr lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\"/><a:lstStyle/>{}</a:txBody><a:tcPr marL=\"101600\" marR=\"101600\" marT=\"76200\" marB=\"76200\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:tcPr></a:tc>",paragraphs(cell,11.0,"172033"),if n==0{"E8EEFB"}else if n%2==0{"F1F5F9"}else{"FFFFFF"}).unwrap();
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
fn chart_xml(c: &ReportChart) -> String {
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
    write!(s,"<c:axId val=\"100\"/><c:axId val=\"200\"/></c:lineChart><c:catAx><c:axId val=\"100\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"b\"/><c:tickLblPos val=\"low\"/><c:crossAx val=\"200\"/><c:crosses val=\"autoZero\"/><c:auto val=\"1\"/><c:lblAlgn val=\"ctr\"/><c:lblOffset val=\"100\"/><c:tickLblSkip val=\"{}\"/></c:catAx><c:valAx><c:axId val=\"200\"/><c:scaling><c:orientation val=\"minMax\"/><c:max val=\"{}\"/><c:min val=\"{}\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"l\"/><c:majorGridlines/><c:numFmt formatCode=\"General\" sourceLinked=\"0\"/><c:tickLblPos val=\"nextTo\"/><c:crossAx val=\"100\"/><c:crosses val=\"autoZero\"/><c:crossBetween val=\"between\"/></c:valAx></c:plotArea><c:legend><c:legendPos val=\"b\"/><c:layout/><c:overlay val=\"0\"/></c:legend><c:plotVisOnly val=\"1\"/><c:dispBlanksAs val=\"gap\"/></c:chart><c:externalData r:id=\"rIdWorkbook\"><c:autoUpdate val=\"0\"/></c:externalData></c:chartSpace>",(c.labels.len()/10).max(1),c.maximum,c.minimum).unwrap();
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
