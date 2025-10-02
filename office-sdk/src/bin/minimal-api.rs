use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio;
use warp::Filter;

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
    let result = create_mock_document(&req.format, &req.content, &req.title, &req.author);

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

fn create_mock_document(
    format: &str,
    content: &str,
    title: &Option<String>,
    author: &Option<String>,
) -> Result<Vec<u8>, String> {
    let title_binding = "Untitled Document".to_string();
    let author_binding = "Unknown Author".to_string();
    let title_str = title.as_ref().unwrap_or(&title_binding);
    let author_str = author.as_ref().unwrap_or(&author_binding);

    let mock_content = match format {
        "docx" => create_mock_docx(title_str, author_str, content),
        "pdf" => create_mock_pdf(title_str, author_str, content),
        "xlsx" => create_mock_xlsx(title_str, content),
        "pptx" => create_mock_pptx(title_str, author_str, content),
        "eml" => create_mock_eml(title_str, author_str, content),
        _ => return Err("Unsupported format".to_string()),
    };

    Ok(mock_content.into_bytes())
}

fn create_mock_docx(title: &str, author: &str, content: &str) -> String {
    format!(
        "MOCK DOCX DOCUMENT\n\
        ==================\n\
        Title: {}\n\
        Author: {}\n\
        Created: {}\n\n\
        Content:\n\
        --------\n\
        {}\n\n\
        [This is a mock DOCX document for testing the API. In a real implementation, \
        this would be a properly formatted Microsoft Word document with XML structure.]",
        title,
        author,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        content
    )
}

fn create_mock_pdf(title: &str, author: &str, content: &str) -> String {
    format!(
        "MOCK PDF DOCUMENT\n\
        =================\n\
        Title: {}\n\
        Author: {}\n\
        Created: {}\n\n\
        Content:\n\
        --------\n\
        {}\n\n\
        [This is a mock PDF document for testing the API. In a real implementation, \
        this would be a properly formatted PDF with binary structure and text layout.]",
        title,
        author,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        content
    )
}

fn create_mock_xlsx(title: &str, content: &str) -> String {
    let rows: Vec<&str> = content.lines().collect();
    let mut xlsx_content = format!(
        "MOCK XLSX SPREADSHEET\n\
        =====================\n\
        Title: {}\n\
        Created: {}\n\n\
        Worksheet: Sheet1\n\
        -----------------\n",
        title,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );

    for (i, row) in rows.iter().enumerate() {
        if content.contains(',') {
            // CSV-like content
            let cells: Vec<&str> = row.split(',').collect();
            xlsx_content.push_str(&format!("Row {}: {}\n", i + 1, cells.join(" | ")));
        } else {
            // Plain text
            xlsx_content.push_str(&format!("A{}: {}\n", i + 1, row));
        }
    }

    xlsx_content.push_str("\n[This is a mock XLSX document for testing the API. In a real implementation, \
                           this would be a properly formatted Excel file with binary structure and cell data.]");

    xlsx_content
}

fn create_mock_pptx(title: &str, author: &str, content: &str) -> String {
    let slides: Vec<&str> = content.split("\n---\n").collect();
    let mut pptx_content = format!(
        "MOCK PPTX PRESENTATION\n\
        ======================\n\
        Title: {}\n\
        Author: {}\n\
        Created: {}\n\n",
        title,
        author,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );

    pptx_content.push_str("Slide 1: Title Slide\n");
    pptx_content.push_str(&format!("  Title: {}\n", title));
    pptx_content.push_str(&format!("  Subtitle: Presented by {}\n\n", author));

    for (i, slide) in slides.iter().enumerate() {
        if !slide.trim().is_empty() {
            pptx_content.push_str(&format!("Slide {}: Content Slide\n", i + 2));
            let lines: Vec<&str> = slide.lines().collect();
            if !lines.is_empty() {
                pptx_content.push_str(&format!("  Title: {}\n", lines[0]));
                if lines.len() > 1 {
                    pptx_content.push_str("  Content:\n");
                    for line in &lines[1..] {
                        pptx_content.push_str(&format!("    • {}\n", line));
                    }
                }
            }
            pptx_content.push('\n');
        }
    }

    pptx_content.push_str("[This is a mock PPTX document for testing the API. In a real implementation, \
                           this would be a properly formatted PowerPoint file with XML structure and slide layouts.]");

    pptx_content
}

fn create_mock_eml(title: &str, author: &str, content: &str) -> String {
    format!(
        "MOCK EML EMAIL\n\
        ==============\n\
        From: {}\n\
        Subject: {}\n\
        Date: {}\n\
        To: recipient@example.com\n\n\
        Message Body:\n\
        -------------\n\
        {}\n\n\
        [This is a mock EML document for testing the API. In a real implementation, \
        this would be a properly formatted email message with RFC 822 headers and MIME structure.]",
        author,
        title,
        chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000"),
        content
    )
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
            "File not found".as_bytes().to_vec(),
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

async fn list_files(storage: FileStorage) -> Result<impl warp::Reply, Infallible> {
    let storage = storage.lock().unwrap();
    let file_count = storage.len();
    let total_size: usize = storage.values().map(|v| v.len()).sum();

    let stats = serde_json::json!({
        "file_count": file_count,
        "total_size": total_size,
        "files": storage.keys().collect::<Vec<_>>()
    });

    Ok(warp::reply::json(&ApiResponse::success(stats)))
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

    // List files endpoint
    let files = warp::path("files")
        .and(warp::get())
        .and(storage_filter.clone())
        .and_then(list_files);

    let routes = health
        .or(create)
        .or(download)
        .or(formats)
        .or(files)
        .with(cors);

    println!("🚀 Minimal Office SDK API Server starting on http://localhost:3030");
    println!("📋 Available endpoints:");
    println!("  GET  /health           - Health check");
    println!("  GET  /formats          - Get supported formats");
    println!("  POST /documents        - Create document (mock implementation)");
    println!("  GET  /download/<id>    - Download file");
    println!("  GET  /files            - List stored files");
    println!("\n📝 Example usage:");
    println!("  curl -X POST http://localhost:3030/documents \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(
        "    -d '{{\"format\":\"pdf\",\"content\":\"Hello World\",\"title\":\"Test Document\"}}'"
    );
    println!("\n⚠️  Note: This is a mock implementation for testing API structure.");
    println!("   Real document generation requires fixing the library compilation issues.");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
