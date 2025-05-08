//! Handles file types for SII files.

use log::info;

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

/// Given a supported file, decode until the textual SII format is reached.
pub fn decode_until_siin(file_content: &[u8]) -> Option<Vec<u8>> {
    let content = file_content;
    let file_type = detect_file_type(file_content)?;
    info!("Obtained file type: {:?}", file_type);
    match file_type {
        FileType::Scsc => {
            let scsc_file = ScscFile::parse(content).unwrap();
            let decoded_content = scsc_file.decode();
            match detect_file_type(&decoded_content)? {
                FileType::Siin => Some(decoded_content),
                FileType::Bsii => {
                    let bsii_file = BsiiFile::parse(&decoded_content).unwrap();
                    Some(bsii_file.to_siin().as_bytes().to_vec())
                }
                _ => unreachable!("Unexpected file type after decoding"),
            }
        }
        FileType::Bsii => {
            let bsii_file = BsiiFile::parse(content).unwrap();
            Some(bsii_file.to_siin().into())
        }
        FileType::Siin => Some(content.to_vec()),
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
