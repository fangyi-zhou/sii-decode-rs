use std::{fmt::Write, iter::zip};

/// Output the parsed BSII format into textual format
/// Reference: https://modding.scssoft.com/wiki/Documentation/Engine/Units
use crate::bsii_decode::{BsiiFile, DataBlock, Prototype};

fn write_data_block<W: Write>(
    f: &mut W,
    data_block: &DataBlock,
    prototype: &Prototype,
) -> std::fmt::Result {
    writeln!(f, "{} : {} {{", prototype.name, data_block.id)?;
    assert_eq!(
        data_block.data.len(),
        prototype.value_prototypes.len(),
        "Data blocks should have the same length as the protytypes"
    );
    for (data, value_prototype) in zip(&data_block.data, &prototype.value_prototypes) {
        // TODO: Format data according to the type
        writeln!(f, "  {}: {:?}", value_prototype.name, data)?;
    }
    writeln!(f, "}}")?;
    Ok(())
}

fn write_bsii<W: Write>(f: &mut W, bsii: &BsiiFile) -> std::fmt::Result {
    writeln!(f, "SiiNunit")?;
    writeln!(f, "{{")?;
    for data_block in &bsii.data_blocks {
        let prototype = bsii.get_prototype(data_block.type_id).unwrap();
        write_data_block(f, data_block, prototype)?;
    }
    writeln!(f, "}}")?;
    Ok(())
}

pub fn bsii_to_siin(bsii: &BsiiFile) -> String {
    let mut output = String::new();
    write_bsii(&mut output, bsii).unwrap();
    output
}

mod tests {
    use super::*;
    // TODO
}
