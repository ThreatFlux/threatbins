use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::Filter;

use office_sdk::OfficeDocument;
use office_sdk::docx::DocxDocument;
use office_sdk::eml::EmlDocument;
use office_sdk::pdf::PdfDocument;
use office_sdk::pptx::PptxDocument;
use office_sdk::xlsx::XlsxDocument;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDocumentRequest {
    pub format: String,
    pub content: String,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileResponse {
    pub filename: String,
    pub size: usize,
    pub format: String,
    pub download_url: String,
}

type FileStorage = Arc<Mutex<HashMap<String, Vec<u8>>>>;

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

async fn create_document(
    req: CreateDocumentRequest,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let result = match req.format.as_str() {
        "docx" => create_docx_document(&req.content, &req.title, &req.author),
        "pdf" => create_pdf_document(&req.content, &req.title, &req.author),
        "xlsx" => create_xlsx_document(&req.content, &req.title, &req.author),
        "pptx" => create_pptx_document(&req.content, &req.title, &req.author),
        "eml" => create_eml_document(&req.content, &req.title, &req.author),
        _ => Err("Unsupported format".to_string()),
    };

    match result {
        Ok(content) => {
            let file_id = uuid::Uuid::new_v4().to_string();
            storage
                .lock()
                .unwrap()
                .insert(file_id.clone(), content.clone());

            let filename = format!(
                "{}.{}",
                req.title
                    .unwrap_or_else(|| "document".to_string())
                    .replace(" ", "_")
                    .to_lowercase(),
                req.format
            );

            let response = FileResponse {
                filename,
                size: content.len(),
                format: req.format,
                download_url: format!("/download/{}", file_id),
            };

            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => Ok(warp::reply::json(&ApiResponse::<FileResponse>::error(e))),
    }
}

fn create_docx_document(
    content: &str,
    title: &Option<String>,
    author: &Option<String>,
) -> Result<Vec<u8>, String> {
    let mut doc = DocxDocument::new();

    if let Some(title) = title {
        doc.metadata.title = Some(title.clone());
    }
    if let Some(author) = author {
        doc.metadata.author = Some(author.clone());
    }

    for line in content.lines() {
        if !line.trim().is_empty() {
            doc.add_paragraph(line).map_err(|e| e.to_string())?;
        }
    }

    doc.to_bytes().map_err(|e| e.to_string())
}

fn create_pdf_document(
    content: &str,
    title: &Option<String>,
    author: &Option<String>,
) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new();

    if let Some(title) = title {
        doc.set_title(title);
    }
    if let Some(author) = author {
        doc.set_author(author);
    }

    doc.add_a4_page().map_err(|e| e.to_string())?;

    if let Some(title) = title {
        doc.add_heading(0, 1, title, 50.0, 750.0)
            .map_err(|e| e.to_string())?;
    }

    let mut y_position = 700.0;
    for line in content.lines() {
        if !line.trim().is_empty() {
            doc.add_text(0, line, 50.0, y_position, "Helvetica", 12.0)
                .map_err(|e| e.to_string())?;
            y_position -= 20.0;
        }
    }

    doc.to_bytes().map_err(|e| e.to_string())
}

fn create_xlsx_document(
    content: &str,
    title: &Option<String>,
    _author: &Option<String>,
) -> Result<Vec<u8>, String> {
    let mut doc = XlsxDocument::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut data: Vec<Vec<String>> = Vec::new();

    for line in lines {
        if content.contains(',') {
            let cells: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            data.push(cells);
        } else {
            data.push(vec![line.to_string()]);
        }
    }

    doc.add_worksheet("Sheet1", &data)
        .map_err(|e| e.to_string())?;
    doc.to_bytes().map_err(|e| e.to_string())
}

fn create_pptx_document(
    content: &str,
    title: &Option<String>,
    author: &Option<String>,
) -> Result<Vec<u8>, String> {
    let mut doc = PptxDocument::new();

    if let (Some(title), Some(author)) = (title, author) {
        doc.add_title_slide(title, author)
            .map_err(|e| e.to_string())?;
    }

    let slides: Vec<&str> = content.split("\n---\n").collect();
    for slide_content in slides {
        if !slide_content.trim().is_empty() {
            let lines: Vec<&str> = slide_content.lines().collect();
            let slide_title = if !lines.is_empty() { lines[0] } else { "Slide" };
            let slide_content_lines = if lines.len() > 1 { &lines[1..] } else { &[] };
            doc.add_slide(slide_title, slide_content_lines)
                .map_err(|e| e.to_string())?;
        }
    }

    doc.to_bytes().map_err(|e| e.to_string())
}

fn create_eml_document(
    content: &str,
    title: &Option<String>,
    author: &Option<String>,
) -> Result<Vec<u8>, String> {
    let mut doc = EmlDocument::new();

    if let Some(author) = author {
        doc.set_from(author, None).map_err(|e| e.to_string())?;
    }

    if let Some(title) = title {
        doc.set_subject(title);
    }

    doc.set_text_body(content);
    doc.to_bytes().map_err(|e| e.to_string())
}

async fn download_file(
    file_id: String,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let storage = storage.lock().unwrap();

    if let Some(content) = storage.get(&file_id) {
        Ok(warp::reply::with_header(
            content.clone(),
            "Content-Type",
            "application/octet-stream",
        ))
    } else {
        Ok(warp::reply::with_header(
            Vec::new(),
            "Content-Type",
            "text/plain",
        ))
    }
}

async fn get_supported_formats() -> Result<impl warp::Reply, Infallible> {
    let formats = vec!["docx", "pdf", "xlsx", "pptx", "eml"];
    Ok(warp::reply::json(&ApiResponse::success(formats)))
}

async fn health_check() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&ApiResponse::success("OK")))
}

#[tokio::main]
async fn main() {
    let file_storage: FileStorage = Arc::new(Mutex::new(HashMap::new()));

    let storage_filter = warp::any().map(move || file_storage.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST"]);

    // Health check endpoint
    let health = warp::path("health").and(warp::get()).and_then(health_check);

    // Create document endpoint
    let create = warp::path("documents")
        .and(warp::post())
        .and(warp::body::json())
        .and(storage_filter.clone())
        .and_then(create_document);

    // Download file endpoint
    let download = warp::path("download")
        .and(warp::path::param::<String>())
        .and(storage_filter.clone())
        .and_then(download_file);

    // Get supported formats endpoint
    let formats = warp::path("formats")
        .and(warp::get())
        .and_then(get_supported_formats);

    let routes = health.or(create).or(download).or(formats).with(cors);

    println!("🚀 Simple Office SDK API Server starting on http://localhost:3030");
    println!("📋 Available endpoints:");
    println!("  GET  /health           - Health check");
    println!("  GET  /formats          - Get supported formats");
    println!("  POST /documents        - Create document");
    println!("  GET  /download/<id>    - Download file");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
