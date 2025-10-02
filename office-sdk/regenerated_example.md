Office SDK Documentation Welcome to the Office SDK - a comprehensive Rust library for handling Office document formats. Core Features The SDK supports multiple document formats: DOCX (Word documents) XLSX (Excel spreadsheets) PPTX (PowerPoint presentations) Markdown conversion (NEW!) Code Examples Here's how to create a DOCX document: use office_sdk::docx::DocxDocument;

# use office_sdk::OfficeDocument;

# let mut doc = DocxDocument::new();

# doc.add_paragraph("Hello, World!");

doc.save("document.docx")?; Markdown Conversion From Markdown to DOCX The library can convert markdown documents to DOCX format: let markdown_doc = MarkdownDocument::from_markdown(markdown_text)?;

# let docx_doc = markdown_doc.to_docx()?;

docx_doc.save("output.docx")?; From DOCX to Markdown Conversely, you can extract markdown from DOCX documents: let docx = DocxDocument::from_file("input.docx")?;

# let markdown = MarkdownDocument::from_docx(&docx)?;

let markdown_text = markdown.to_markdown(); Advanced Features The SDK provides comprehensive error handling and type safety. Key benefits: Memory-efficient ZIP processing Full OpenXML compatibility Extensive test coverage (97%+) Clean, idiomatic Rust APIs Ready to get started? Check out the examples and documentation!
