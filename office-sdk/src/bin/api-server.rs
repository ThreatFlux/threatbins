use office_sdk::files::{CreateRequest, DocumentFormat, FileInput, FileMetadata, FilesAPI};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::Filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDocumentRequest {
    pub format: DocumentFormat,
    pub content: String,
    pub metadata: Option<FileMetadata>,
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConvertDocumentRequest {
    pub content: Vec<u8>,
    pub source_format: DocumentFormat,
    pub target_format: DocumentFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchCreateRequest {
    pub requests: Vec<CreateRequest>,
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
    pub format: DocumentFormat,
    pub mime_type: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageStats {
    pub total_files: usize,
    pub current_size: usize,
    pub max_size: usize,
    pub formats: HashMap<String, usize>,
}

type SharedFilesAPI = Arc<Mutex<FilesAPI>>;
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
    api: SharedFilesAPI,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let api = api.lock().unwrap();

    let metadata = req.metadata.unwrap_or_default();

    let result = if let Some(template) = req.template {
        match req.format {
            DocumentFormat::Docx => {
                api.builders
                    .docx
                    .with_template(&template, &req.content, metadata)
            }
            DocumentFormat::Xlsx => {
                api.builders
                    .xlsx
                    .with_template(&template, &req.content, metadata)
            }
            DocumentFormat::Pptx => {
                api.builders
                    .pptx
                    .with_template(&template, &req.content, metadata)
            }
            DocumentFormat::Pdf => {
                api.builders
                    .pdf
                    .with_template(&template, &req.content, metadata)
            }
            DocumentFormat::Eml => {
                api.builders
                    .eml
                    .with_template(&template, &req.content, metadata)
            }
            DocumentFormat::Markdown => {
                api.builders
                    .markdown
                    .with_template(&template, &req.content, metadata)
            }
        }
    } else {
        api.create_document_with_metadata(req.format, &req.content, metadata)
    };

    match result {
        Ok(file_result) => {
            let file_id = uuid::Uuid::new_v4().to_string();
            storage
                .lock()
                .unwrap()
                .insert(file_id.clone(), file_result.content.clone());

            let response = FileResponse {
                filename: file_result.filename.clone(),
                size: file_result.size(),
                format: file_result.format.clone(),
                mime_type: file_result.mime_type.clone(),
                download_url: format!("/download/{}", file_id),
            };

            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => Ok(warp::reply::json(&ApiResponse::<FileResponse>::error(
            e.to_string(),
        ))),
    }
}

async fn convert_document(
    req: ConvertDocumentRequest,
    api: SharedFilesAPI,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let api = api.lock().unwrap();

    let input = FileInput::Content {
        data: req.content,
        format: req.source_format,
    };

    match api.convert_document(input, req.target_format) {
        Ok(file_result) => {
            let file_id = uuid::Uuid::new_v4().to_string();
            storage
                .lock()
                .unwrap()
                .insert(file_id.clone(), file_result.content.clone());

            let response = FileResponse {
                filename: file_result.filename.clone(),
                size: file_result.size(),
                format: file_result.format.clone(),
                mime_type: file_result.mime_type.clone(),
                download_url: format!("/download/{}", file_id),
            };

            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => Ok(warp::reply::json(&ApiResponse::<FileResponse>::error(
            e.to_string(),
        ))),
    }
}

async fn batch_create(
    req: BatchCreateRequest,
    api: SharedFilesAPI,
    storage: FileStorage,
) -> Result<impl warp::Reply, Infallible> {
    let api = api.lock().unwrap();

    match api.batch_create(req.requests) {
        Ok(results) => {
            let mut responses = Vec::new();

            for file_result in results {
                let file_id = uuid::Uuid::new_v4().to_string();
                storage
                    .lock()
                    .unwrap()
                    .insert(file_id.clone(), file_result.content.clone());

                responses.push(FileResponse {
                    filename: file_result.filename.clone(),
                    size: file_result.size(),
                    format: file_result.format.clone(),
                    mime_type: file_result.mime_type.clone(),
                    download_url: format!("/download/{}", file_id),
                });
            }

            Ok(warp::reply::json(&ApiResponse::success(responses)))
        }
        Err(e) => Ok(warp::reply::json(&ApiResponse::<Vec<FileResponse>>::error(
            e.to_string(),
        ))),
    }
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
    let formats = office_sdk::files::FilesAPI::get_supported_formats();
    Ok(warp::reply::json(&ApiResponse::success(formats)))
}

async fn get_storage_stats(api: SharedFilesAPI) -> Result<impl warp::Reply, Infallible> {
    let api = api.lock().unwrap();
    let stats = api.storage.get_storage_stats();

    let mut formats = HashMap::new();
    for (format, count) in stats.files_by_format {
        formats.insert(format!("{:?}", format), count);
    }

    let response = StorageStats {
        total_files: stats.total_files,
        current_size: stats.current_size,
        max_size: stats.max_size,
        formats,
    };

    Ok(warp::reply::json(&ApiResponse::success(response)))
}

async fn health_check() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&ApiResponse::success("OK")))
}

#[tokio::main]
async fn main() {
    let files_api = Arc::new(Mutex::new(FilesAPI::new()));
    let file_storage: FileStorage = Arc::new(Mutex::new(HashMap::new()));

    let api_filter = warp::any().map(move || files_api.clone());
    let storage_filter = warp::any().map(move || file_storage.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    // Health check endpoint
    let health = warp::path("health").and(warp::get()).and_then(health_check);

    // Create document endpoint
    let create = warp::path("documents")
        .and(warp::post())
        .and(warp::body::json())
        .and(api_filter.clone())
        .and(storage_filter.clone())
        .and_then(create_document);

    // Convert document endpoint
    let convert = warp::path("convert")
        .and(warp::post())
        .and(warp::body::json())
        .and(api_filter.clone())
        .and(storage_filter.clone())
        .and_then(convert_document);

    // Batch create endpoint
    let batch = warp::path("batch")
        .and(warp::post())
        .and(warp::body::json())
        .and(api_filter.clone())
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
        .and(api_filter.clone())
        .and_then(get_storage_stats);

    let routes = health
        .or(create)
        .or(convert)
        .or(batch)
        .or(download)
        .or(formats)
        .or(stats)
        .with(cors);

    println!("🚀 Office SDK API Server starting on http://localhost:3030");
    println!("📋 Available endpoints:");
    println!("  GET  /health           - Health check");
    println!("  GET  /formats          - Get supported formats");
    println!("  GET  /stats            - Get storage statistics");
    println!("  POST /documents        - Create document");
    println!("  POST /convert          - Convert document");
    println!("  POST /batch            - Batch create documents");
    println!("  GET  /download/<id>    - Download file");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
