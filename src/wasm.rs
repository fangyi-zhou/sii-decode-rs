use wasm_bindgen::{prelude::wasm_bindgen, JsError};

use crate::file_type::decode_until_siin;

#[wasm_bindgen]
pub fn decode(input: &[u8]) -> Result<String, JsError> {
    if let Some(result) = decode_until_siin(input) {
        Ok(String::from_utf8(result).unwrap())
    } else {
        Err(JsError::new("Failed to process input"))
    }
}
