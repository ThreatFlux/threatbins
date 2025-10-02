use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::Filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDocumentRequest {
    pub format: String,
    pub content: String,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConvertDocumentRequest {
    pub content: String,
    pub source_format: String,
    pub target_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchCreateRequest {
    pub requests: Vec<CreateDocumentRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileResponse {
    pub id: String,
    pub filename: String,
    pub size: usize,
    pub format: String,
    pub mime_type: String,
    pub download_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageStats {
    pub total_files: usize,
    pub current_size: usize,
    pub formats: HashMap<String, usize>,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    pub id: String,
    pub filename: String,
    pub format: String,
    pub size: usize,
    pub created_at: String,
}

type FileStorage = Arc<Mutex<HashMap<String, (Vec<u8>, FileResponse)>>>;

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
            let filename = format!("{}.{}",
                req.title.as_ref().unwrap_or(&"document".to_string()).replace(" ", "_").to_lowercase(),
                req.format
            );

            let mime_type = get_mime_type(&req.format);
            let created_at = chrono::Utc::now().to_rfc3339();

            let file_response = FileResponse {
                id: file_id.clone(),
                filename: filename.clone(),
                size: content.len(),
                format: req.format.clone(),
                mime_type,
                download_url: format!("/download/{}", file_id),
                created_at,
            };

            storage.lock().unwrap().insert(file_id.clone(), (content, file_response.clone()));

            Ok(warp::reply::json(&ApiResponse::success(file_response)))
        }
        Err(e) => Ok(warp::reply::json(&ApiResponse::<FileResponse>::error(e))),
    }
}

async fn convert_document(
    req: ConvertDocumentRequest,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let result = convert_mock_document(&req.content, &req.source_format, &req.target_format);

    match result {
        Ok(content) => {
            let file_id = uuid::Uuid::new_v4().to_string();
            let filename = format!("converted_document.{}", req.target_format);
            let mime_type = get_mime_type(&req.target_format);
            let created_at = chrono::Utc::now().to_rfc3339();

            let file_response = FileResponse {
                id: file_id.clone(),
                filename: filename.clone(),
                size: content.len(),
                format: req.target_format.clone(),
                mime_type,
                download_url: format!("/download/{}", file_id),
                created_at,
            };

            storage.lock().unwrap().insert(file_id.clone(), (content, file_response.clone()));

            Ok(warp::reply::json(&ApiResponse::success(file_response)))
        }
        Err(e) => Ok(warp::reply::json(&ApiResponse::<FileResponse>::error(e))),
    }
}

async fn batch_create(
    req: BatchCreateRequest,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let mut responses = Vec::new();

    for create_req in req.requests {
        let result = create_mock_document(&create_req.format, &create_req.content, &create_req.title, &create_req.author);

        match result {
            Ok(content) => {
                let file_id = uuid::Uuid::new_v4().to_string();
                let filename = format!("{}.{}",
                    create_req.title.as_ref().unwrap_or(&"document".to_string()).replace(" ", "_").to_lowercase(),
                    create_req.format
                );

                let mime_type = get_mime_type(&create_req.format);
                let created_at = chrono::Utc::now().to_rfc3339();

                let file_response = FileResponse {
                    id: file_id.clone(),
                    filename: filename.clone(),
                    size: content.len(),
                    format: create_req.format.clone(),
                    mime_type,
                    download_url: format!("/download/{}", file_id),
                    created_at,
                };

                storage.lock().unwrap().insert(file_id.clone(), (content, file_response.clone()));
                responses.push(file_response);
            }
            Err(e) => {
                return Ok(warp::reply::json(&ApiResponse::<Vec<FileResponse>>::error(e)));
            }
        }
    }

    Ok(warp::reply::json(&ApiResponse::success(responses)))
}

fn get_mime_type(format: &str) -> String {
    match format {
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
        "pdf" => "application/pdf".to_string(),
        "eml" => "message/rfc822".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn create_mock_document(format: &str, content: &str, title: &Option<String>, author: &Option<String>) -> Result<Vec<u8>, String> {
    let default_title = "Untitled Document".to_string();
    let default_author = "Unknown Author".to_string();
    let title_str = title.as_ref().unwrap_or(&default_title);
    let author_str = author.as_ref().unwrap_or(&default_author);

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

fn convert_mock_document(content: &str, source_format: &str, target_format: &str) -> Result<Vec<u8>, String> {
    let mock_content = format!(
        "CONVERTED DOCUMENT\n\
        ==================\n\
        Source Format: {}\n\
        Target Format: {}\n\
        Converted: {}\n\n\
        Original Content:\n\
        ----------------\n\
        {}\n\n\
        [This is a mock conversion for testing the API. In a real implementation, \
        this would properly convert between different document formats.]",
        source_format.to_uppercase(), target_format.to_uppercase(),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), content
    );

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
        title, author, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), content
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
        title, author, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), content
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
        title, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );

    for (i, row) in rows.iter().enumerate() {
        if content.contains(',') {
            let cells: Vec<&str> = row.split(',').collect();
            xlsx_content.push_str(&format!("Row {}: {}\n", i + 1, cells.join(" | ")));
        } else {
            xlsx_content.push_str(&format!("A{}: {}\n", i + 1, row));
        }
    }

    xlsx_content.push_str("\n[This is a mock XLSX document for testing the API.]");
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
        title, author, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
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

    pptx_content.push_str("[This is a mock PPTX document for testing the API.]");
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
        [This is a mock EML document for testing the API.]",
        author, title, chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000"), content
    )
}

async fn download_file(
    file_id: String,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let storage = storage.lock().unwrap();

    if let Some((content, file_info)) = storage.get(&file_id) {
        Ok(warp::reply::with_header(
            warp::reply::with_header(
                content.clone(),
                "Content-Type",
                file_info.mime_type.clone(),
            ),
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_info.filename),
        ))
    } else {
        Ok(warp::reply::with_header(
            warp::reply::with_header(
                "File not found".as_bytes().to_vec(),
                "Content-Type",
                "text/plain",
            ),
            "Content-Disposition",
            "inline",
        ))
    }
}

async fn get_supported_formats() -> Result<impl warp::Reply, Infallible> {
    let formats = vec!["docx", "pdf", "xlsx", "pptx", "eml"];
    Ok(warp::reply::json(&ApiResponse::success(formats)))
}

async fn health_check() -> Result<impl warp::Reply, Infallible> {
    let status = serde_json::json!({
        "status": "OK",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "service": "Office SDK API"
    });
    Ok(warp::reply::json(&ApiResponse::success(status)))
}

async fn get_storage_stats(storage: FileStorage) -> Result<impl warp::Reply, Infallible> {
    let storage = storage.lock().unwrap();
    let total_files = storage.len();
    let current_size: usize = storage.values().map(|(content, _)| content.len()).sum();

    let mut formats: HashMap<String, usize> = HashMap::new();
    let mut files: Vec<FileInfo> = Vec::new();

    for (_, (_, file_info)) in storage.iter() {
        *formats.entry(file_info.format.clone()).or_insert(0) += 1;
        files.push(FileInfo {
            id: file_info.id.clone(),
            filename: file_info.filename.clone(),
            format: file_info.format.clone(),
            size: file_info.size,
            created_at: file_info.created_at.clone(),
        });
    }

    let stats = StorageStats {
        total_files,
        current_size,
        formats,
        files,
    };

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
    let health = warp::path("health")
        .and(warp::get())
        .and_then(health_check);

    // Create document endpoint
    let create = warp::path("documents")
        .and(warp::post())
        .and(warp::body::json())
        .and(storage_filter.clone())
        .and_then(create_document);

    // Convert document endpoint
    let convert = warp::path("convert")
        .and(warp::post())
        .and(warp::body::json())
        .and(storage_filter.clone())
        .and_then(convert_document);

    // Batch create endpoint
    let batch = warp::path("batch")
        .and(warp::post())
        .and(warp::body::json())
        .and(storage_filter.clone())
        .and_then(batch_create);

    // Download file endpoint
    let download = warp::path("download")
        .and(warp::path::param::<String>())
        .and(storage_filter.clone())
        .and_then(download_file);

    // Get supported formats endpoint
    let formats = warp::path("formats")
        .and(warp::get())
        .and_then(get_supported_formats);

    // Get storage stats endpoint
    let stats = warp::path("stats")
        .and(warp::get())
        .and(storage_filter.clone())
        .and_then(get_storage_stats);

    let routes = health
        .or(create)
        .or(convert)
        .or(batch)
        .or(download)
        .or(formats)
        .or(stats)
        .with(cors);

    println!("🚀 Office SDK API Test Server starting on http://localhost:3030");
    println!("📋 Available endpoints:");
    println!("  GET  /health           - Health check");
    println!("  GET  /formats          - Get supported formats");
    println!("  GET  /stats            - Get storage statistics");
    println!("  POST /documents        - Create document");
    println!("  POST /convert          - Convert document");
    println!("  POST /batch            - Batch create documents");
    println!("  GET  /download/<id>    - Download file");
    println!("\n📝 Example usage:");
    println!("  curl -X POST http://localhost:3030/documents \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"format\":\"pdf\",\"content\":\"Hello World\",\"title\":\"Test Document\"}}'");
    println!("\n⚠️  Note: This is a mock implementation for testing API structure.");
    println!("   Real document generation requires the full SDK implementation.");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}