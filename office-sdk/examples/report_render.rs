//! cargo run --example report_render -- tests/fixtures/executive-report.json output/reports
use office_sdk::report::*;
use std::{env, fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let fixture = args
        .next()
        .unwrap_or_else(|| "tests/fixtures/executive-report.json".into());
    let out = PathBuf::from(args.next().unwrap_or_else(|| "output/reports".into()));
    let document: ReportDocument = serde_json::from_slice(&fs::read(fixture)?)?;
    fs::create_dir_all(&out)?;
    let digest = report_document_sha256(&document)?;
    for format in [ReportFormat::Pdf, ReportFormat::Html, ReportFormat::Pptx] {
        let mut options = ReportRenderOptions::new(format);
        options.expected_document_sha256 = Some(digest.clone());
        let result = render_report(&document, &options)?;
        fs::write(
            out.join(format!("report.{}", format.extension())),
            &result.bytes,
        )?;
        fs::write(
            out.join(format!("report.{}.manifest.json", format.extension())),
            serde_json::to_vec_pretty(&result.manifest)?,
        )?;
        println!(
            "{}: {} bytes, {:?} pages/slides, {}",
            format.extension(),
            result.manifest.byte_size,
            result.manifest.pages_or_slides,
            result.manifest.output_sha256
        );
    }
    Ok(())
}
