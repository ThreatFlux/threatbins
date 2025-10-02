use office_sdk::OfficeDocument;
use office_sdk::docx::DocxDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = DocxDocument::new();

    doc.add_paragraph("Hello, World!")?;
    doc.add_paragraph("This is a paragraph created with the Office SDK.")?;
    doc.add_paragraph("You can add multiple paragraphs to build rich documents.")?;

    doc.save("example.docx")?;
    println!("DOCX document created successfully: example.docx");

    let loaded_doc = DocxDocument::from_file("example.docx")?;
    let text = loaded_doc.get_text()?;
    println!("Document text content:");
    println!("{}", text);

    Ok(())
}
