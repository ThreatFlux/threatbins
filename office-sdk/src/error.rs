use thiserror::Error;

pub type Result<T> = std::result::Result<T, OfficeError>;

#[derive(Error, Debug)]
pub enum OfficeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::Error),

    #[error("XML deserialization error: {0}")]
    XmlSerialization(String),

    #[error("Invalid document format: {0}")]
    InvalidFormat(String),

    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Image processing error: {0}")]
    Image(#[from] image::error::ImageError),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Storage full - cannot store more files")]
    StorageFull,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
