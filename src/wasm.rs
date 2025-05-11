use wasm_bindgen::{prelude::wasm_bindgen, JsError};

use crate::file_type::decode_until_siin;

#[wasm_bindgen]
pub fn decode(input: &[u8]) -> Result<String, JsError> {
    match decode_until_siin(input) {
        Ok(decoded) => {
            let decoded_str = String::from_utf8(decoded)
                .map_err(|_| JsError::new("Failed to convert to UTF-8"))?;
            Ok(decoded_str)
        }
        Err(err) => Err(JsError::new(&format!("Decoding error: {}", err))),
    }
}
