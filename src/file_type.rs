//! Handles file types for SII files.

use std::borrow::Cow;

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
    /// Error when structured analysis is requested for textual SII.
    StructuredBsiiUnavailable,
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
            DecodeError::StructuredBsiiUnavailable => {
                write!(f, "Structured BSII analysis requires a binary BSII file")
            }
        }
    }
}

/// Given a supported file, decode until the binary BSII format is reached.
///
/// This is intended for structured analysis. Textual `SiiN` input is rejected
/// because it has already lost the prototype metadata needed to inspect blocks
/// and fields safely.
pub fn decode_until_bsii(file_content: &[u8]) -> Result<Cow<'_, [u8]>, DecodeError> {
    let file_type = detect_file_type(file_content).ok_or(DecodeError::UnknownFileType)?;
    info!("Obtained file type: {:?}", file_type);
    match file_type {
        FileType::Scsc => {
            let scsc_file = ScscFile::parse(file_content)?;
            let decoded_content = scsc_file.decode()?;
            match detect_file_type(&decoded_content).ok_or(DecodeError::UnknownFileType)? {
                FileType::Bsii => {
                    BsiiFile::parse(&decoded_content)?;
                    Ok(Cow::Owned(decoded_content))
                }
                FileType::Siin => Err(DecodeError::StructuredBsiiUnavailable),
                FileType::Scsc => unreachable!("Unexpected ScsC file after decoding"),
            }
        }
        FileType::Bsii => {
            BsiiFile::parse(file_content)?;
            Ok(Cow::Borrowed(file_content))
        }
        FileType::Siin => Err(DecodeError::StructuredBsiiUnavailable),
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

    fn minimal_bsii() -> &'static [u8] {
        &[
            b'B', b'S', b'I', b'I', // file signature
            0x02, 0x00, 0x00, 0x00, // format version
            0x00, 0x00, 0x00, 0x00, // block type
            0x00, // validity
        ]
    }

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

    #[test]
    fn decode_until_bsii_accepts_binary_bsii() {
        let decoded = decode_until_bsii(minimal_bsii()).unwrap();

        assert!(matches!(decoded, Cow::Borrowed(_)));
        assert_eq!(decoded.as_ref(), minimal_bsii());
    }

    #[test]
    fn decode_until_bsii_rejects_textual_siin() {
        let err = decode_until_bsii(b"SiiNunit\n{\n}\n").unwrap_err();

        assert!(matches!(err, DecodeError::StructuredBsiiUnavailable));
    }

    #[test]
    fn decode_until_bsii_preserves_header_errors() {
        let err = decode_until_bsii(b"Other").unwrap_err();

        assert!(matches!(err, DecodeError::UnknownFileType));
    }

    #[test]
    fn decode_until_siin_behavior_is_unchanged_for_supported_headers() {
        assert_eq!(
            decode_until_siin(b"SiiNunit\n{\n}\n").unwrap(),
            b"SiiNunit\n{\n}\n"
        );
        assert_eq!(
            String::from_utf8(decode_until_siin(minimal_bsii()).unwrap()).unwrap(),
            "SiiNunit\n{\n}\n"
        );
    }
}
