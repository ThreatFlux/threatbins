use crate::error::{OfficeError, Result};
use std::io::Read;
use zip::ZipArchive;

pub fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Result<String> {
    let mut entry = archive
        .by_name(entry_name)
        .map_err(|_| OfficeError::MissingFile(entry_name.to_string()))?;

    let mut content = String::new();
    entry.read_to_string(&mut content)?;
    Ok(content)
}

pub fn read_zip_entry_bytes<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Result<Vec<u8>> {
    let mut entry = archive
        .by_name(entry_name)
        .map_err(|_| OfficeError::MissingFile(entry_name.to_string()))?;

    let mut content = Vec::new();
    entry.read_to_end(&mut content)?;
    Ok(content)
}

pub fn validate_office_format<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    required_files: &[&str],
) -> Result<()> {
    let file_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    for required_file in required_files {
        if !file_names.iter().any(|name| name == required_file) {
            return Err(OfficeError::InvalidFormat(format!(
                "Missing required file: {}",
                required_file
            )));
        }
    }

    Ok(())
}
