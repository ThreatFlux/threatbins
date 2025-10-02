use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::Filter;

// Only import what we need to avoid compilation issues
use office_sdk::OfficeDocument;

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
    // For now, just create dummy content to test the API structure
    let result = match req.format.as_str() {
        "docx" => create_dummy_document("docx", &req.content, &req.title),
        "pdf" => create_dummy_document("pdf", &req.content, &req.title),
        "xlsx" => create_dummy_document("xlsx", &req.content, &req.title),
        "pptx" => create_dummy_document("pptx", &req.content, &req.title),
        "eml" => create_dummy_document("eml", &req.content, &req.title),
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

fn create_dummy_document(
    format: &str,
    content: &str,
    title: &Option<String>,
) -> Result<Vec<u8>, String> {
    // Create a simple mock document for testing
    let mock_content = format!(
        "Mock {} Document\n\nTitle: {}\nContent: {}\n\nThis is a placeholder document for testing the API.",
        format.to_uppercase(),
        title.as_ref().unwrap_or(&"Untitled".to_string()),
        content
    );

    Ok(mock_content.into_bytes())
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

    println!("🚀 Standalone Office SDK API Server starting on http://localhost:3030");
    println!("📋 Available endpoints:");
    println!("  GET  /health           - Health check");
    println!("  GET  /formats          - Get supported formats");
    println!("  POST /documents        - Create document");
    println!("  GET  /download/<id>    - Download file");
    println!("  GET  /files            - List stored files");
    println!("\n📝 Example usage:");
    println!("  curl -X POST http://localhost:3030/documents \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(
        "    -d '{{\"format\":\"pdf\",\"content\":\"Hello World\",\"title\":\"Test Document\"}}'"
    );

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
