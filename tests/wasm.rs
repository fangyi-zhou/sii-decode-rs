use sii_decode::wasm::decode;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_decode_with_Siin() {
    let input = b"SiiN";
    let result = decode(input).unwrap();
    assert_eq!(result, "SiiN");
}
