/// Output the parsed BSII format into textual format
use crate::bsii_decode::BsiiFile;

pub fn bsii_to_siin(bsii: &BsiiFile) -> String {
    let mut output = String::new();
    output.push_str("SiiNunit\n");
    output.push_str("{\n");
    for data_block in &bsii.data_blocks {
        output.push_str(&data_block.id.to_string());
        output.push('\n');
    }
    output.push_str("}\n");
    output
}

mod tests {
    use super::*;
    // TODO
}
