//! Parses an ScsC file and decrypts the content.
//!
//! References:
//! <https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Source/SII_Decrypt_Decryptor.pas>
//! <https://gitlab.com/jammerxd/sii-decryptsharp/-/blob/main/SIIDecryptSharp/SIIDecryptSharp/Decryptor.cs>
use std::io::Read;

use aes::cipher::block_padding::NoPadding;
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

impl ScscFile<'_> {
    /// Decrypts the data and decompress the payload data
    // TODO: Add error handling
    pub fn decode(&self) -> Vec<u8> {
        let mut buf_decryption: Vec<u8> = vec![0; self.data.len()];
        let cipher = Aes256CbcDec::new_from_slices(ENCRYPTION_KEY, self.iv).unwrap();
        cipher
            .decrypt_padded_b2b_mut::<NoPadding>(self.data, buf_decryption.as_mut())
            .unwrap();
        let mut buf_decompression: Vec<u8> = vec![0; self.size as usize];
        let mut decoder = ZlibDecoder::new(buf_decryption.as_slice());
        decoder.read_exact(&mut buf_decompression).unwrap();
        buf_decompression
    }
}

const ENCRYPTION_KEY: &[u8; 32] = &[
    0x2a, 0x5f, 0xcb, 0x17, 0x91, 0xd2, 0x2f, 0xb6, 0x02, 0x45, 0xb3, 0xd8, 0x36, 0x9e, 0xd0, 0xb2,
    0xc2, 0x73, 0x71, 0x56, 0x3f, 0xbf, 0x1f, 0x3c, 0x9e, 0xdf, 0x6b, 0x11, 0x82, 0x5a, 0x5d, 0x0a,
];

// TODO: Add tests for decode
