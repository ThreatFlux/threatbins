use crate::error::{OfficeError, Result};
use quick_xml::events::{BytesRef, BytesText};
use std::io::Read;
use zip::ZipArchive;

/// Decodes a text event and maps malformed XML into the SDK error type.
pub(crate) fn decode_xml_text(text: &BytesText<'_>) -> Result<String> {
    quick_xml::escape::unescape(text.as_ref())
        .map(|text| text.into_owned())
        .map_err(|error| OfficeError::XmlParsing(error.into()))
}

/// Resolves a predefined or numeric XML reference emitted as a separate event.
pub(crate) fn decode_xml_reference(reference: &BytesRef<'_>) -> Result<String> {
    let escaped = format!("&{};", reference.as_ref());
    quick_xml::escape::unescape(&escaped)
        .map(|text| text.into_owned())
        .map_err(|error| OfficeError::XmlParsing(error.into()))
}

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
        .map(|i| Ok(archive.by_index(i)?.name()?.into_owned()))
        .collect::<Result<_>>()?;

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
