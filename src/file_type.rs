//! Handles file types for SII files.

use log::info;

use crate::bsii_parse;
use crate::scsc_file;
use crate::{bsii_file::BsiiFile, scsc_file::ScscFile};

/// FileType enum representing different file types.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    /// A binary file containing compressed and encrypted data.
    Scsc,
    /// A binary file of SII data format.
    Bsii,
    /// A text file of SII data format.
    Siin,
}

/// Detects the file type based on the header of the file.
/// If the file type is not recognized, it returns None.
pub fn detect_file_type(file_content: &[u8]) -> Option<FileType> {
    if file_content.len() < 4 {
        return None;
    }
    if file_content[0..4] == *"ScsC".as_bytes() {
        Some(FileType::Scsc)
    } else if file_content[0..4] == *"BSII".as_bytes() {
        Some(FileType::Bsii)
    } else if file_content[0..4] == *"SiiN".as_bytes() {
        Some(FileType::Siin)
    } else {
        None
    }
}

#[derive(Debug)]
pub enum DecodeError {
    /// Error when the file type is not recognized.
    UnknownFileType,
    /// Error when Scsc file parsing fails.
    ScscParse(scsc_file::ParseError),
    /// Error when Scsc file decoding fails.
    ScscDecode(scsc_file::DecodeError),
    /// Error when BSII file parsing fails.
    BsiiParse(bsii_parse::ParseError),
}

impl From<scsc_file::ParseError> for DecodeError {
    fn from(err: scsc_file::ParseError) -> Self {
        DecodeError::ScscParse(err)
    }
}

impl From<scsc_file::DecodeError> for DecodeError {
    fn from(err: scsc_file::DecodeError) -> Self {
        DecodeError::ScscDecode(err)
    }
}

impl From<bsii_parse::ParseError> for DecodeError {
    fn from(err: bsii_parse::ParseError) -> Self {
        DecodeError::BsiiParse(err)
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnknownFileType => write!(f, "Unknown file type"),
            DecodeError::ScscParse(err) => write!(f, "Scsc parse error: {}", err),
            DecodeError::ScscDecode(err) => write!(f, "Scsc decode error: {}", err),
            DecodeError::BsiiParse(err) => write!(f, "BSII parse error: {}", err),
        }
    }
}

/// Given a supported file, decode until the textual SII format is reached.
pub fn decode_until_siin(file_content: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let content = file_content;
    let file_type = detect_file_type(file_content).ok_or(DecodeError::UnknownFileType)?;
    info!("Obtained file type: {:?}", file_type);
    match file_type {
        FileType::Scsc => {
            let scsc_file = ScscFile::parse(content)?;
            let decoded_content = scsc_file.decode()?;
            match detect_file_type(&decoded_content).ok_or(DecodeError::UnknownFileType)? {
                FileType::Siin => Ok(decoded_content),
                FileType::Bsii => {
                    let bsii_file = BsiiFile::parse(&decoded_content)?;
                    Ok(bsii_file.to_siin().as_bytes().to_vec())
                }
                _ => unreachable!("Unexpected file type after decoding"),
            }
        }
        FileType::Bsii => {
            let bsii_file = BsiiFile::parse(content)?;
            Ok(bsii_file.to_siin().into())
        }
        FileType::Siin => Ok(content.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        let scsc_header = b"ScsC";
        let bsii_header = b"BSII";
        let siin_header = b"SiiN";
        let empty_file = b"";
        let other_header = b"Other";

        assert_eq!(detect_file_type(scsc_header), Some(FileType::Scsc));
        assert_eq!(detect_file_type(bsii_header), Some(FileType::Bsii));
        assert_eq!(detect_file_type(siin_header), Some(FileType::Siin));
        assert_eq!(detect_file_type(empty_file), None);
        assert_eq!(detect_file_type(other_header), None);
    }
}
