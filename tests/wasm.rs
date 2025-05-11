#[cfg(feature = "wasm")]
mod wasm_test {

    use sii_decode::wasm::decode;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_decode_with_siin() {
        // Given a SiiN file, the decoding should return the same content.
        let input = b"SiiN";
        let result = decode(input).unwrap();
        assert_eq!(result, "SiiN");
    }

    #[wasm_bindgen_test]
    fn test_decode_with_failure() {
        // Given an invalid file, the decoding should return an error.
        let input = b"Invalid data";
        decode(input).expect_err("Decoding error: Unknown file type");
    }
}
