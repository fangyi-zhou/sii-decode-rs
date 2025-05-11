//! Parses an ScsC file and decrypts the content.
//!
//! References:
//! <https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Source/SII_Decrypt_Decryptor.pas>
//! <https://gitlab.com/jammerxd/sii-decryptsharp/-/blob/main/SIIDecryptSharp/SIIDecryptSharp/Decryptor.cs>
use std::io::{self, Read};

use aes::cipher::block_padding::{NoPadding, UnpadError};
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use flate2::read::ZlibDecoder;

/// Structure of a ScsC file
/// ScsC file is a binary file that contains encrypted and compressed data.
/// The file starts with a header "ScsC", followed by an HMAC, an IV, a size, and the data.
/// The data is encrypted using AES-256-CBC and compressed using zlib.
/// After decryption and decompression, the data might be in BSII format (binary
/// form) or SIIN format (textual form).
pub struct ScscFile<'a> {
    pub(crate) header: &'a [u8], // ScsC, size 4
    pub(crate) hmac: &'a [u8],   // size 32
    pub(crate) iv: &'a [u8],     // size 16
    pub(crate) size: u32,
    pub data: &'a [u8],
}

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

#[derive(Debug)]
pub enum ParseError {
    InvalidHeader,
    InvalidInput,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidHeader => write!(f, "Invalid header"),
            ParseError::InvalidInput => write!(f, "Invalid input"),
        }
    }
}

#[derive(Debug)]
pub enum DecodeError {
    DecryptionError(UnpadError),
    DecompressionError(io::Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::DecryptionError(err) => write!(f, "Decryption error: {}", err),
            DecodeError::DecompressionError(err) => write!(f, "Decompression error: {}", err),
        }
    }
}

impl From<UnpadError> for DecodeError {
    fn from(err: UnpadError) -> Self {
        DecodeError::DecryptionError(err)
    }
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> Self {
        DecodeError::DecompressionError(err)
    }
}

impl ScscFile<'_> {
    /// Decrypts the data and decompress the payload data
    pub fn decode(&self) -> Result<Vec<u8>, DecodeError> {
        let mut buf_decryption: Vec<u8> = vec![0; self.data.len()];
        // There shouldn't be any error when initializing the decryptor, since the key and IV are of fixed size.
        let cipher = Aes256CbcDec::new_from_slices(ENCRYPTION_KEY, self.iv).unwrap();
        cipher.decrypt_padded_b2b_mut::<NoPadding>(self.data, buf_decryption.as_mut())?;
        let mut buf_decompression: Vec<u8> = vec![0; self.size as usize];
        let mut decoder = ZlibDecoder::new(buf_decryption.as_slice());
        decoder.read_exact(&mut buf_decompression)?;
        Ok(buf_decompression)
    }
}

const ENCRYPTION_KEY: &[u8; 32] = &[
    0x2a, 0x5f, 0xcb, 0x17, 0x91, 0xd2, 0x2f, 0xb6, 0x02, 0x45, 0xb3, 0xd8, 0x36, 0x9e, 0xd0, 0xb2,
    0xc2, 0x73, 0x71, 0x56, 0x3f, 0xbf, 0x1f, 0x3c, 0x9e, 0xdf, 0x6b, 0x11, 0x82, 0x5a, 0x5d, 0x0a,
];

// TODO: Add tests for decode
