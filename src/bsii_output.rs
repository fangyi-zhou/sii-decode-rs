use std::{
    fmt::{self, Write},
    iter::zip,
};

use crate::bsii_file::{BsiiFile, DataBlock, DataValue, Id, Prototype, ValuePrototype};

/// Output the parsed BSII format into textual format
/// Reference: https://modding.scssoft.com/wiki/Documentation/Engine/Units
fn write_string<W: Write>(f: &mut W, data: &str) -> std::fmt::Result {
    // If the string consists of only digits, alphabetic characters, and underscores,
    // then it should be written without quotes
    if data.is_empty() {
        write!(f, "\"\"")
    } else if data.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        write!(f, "{}", data)
    } else {
        write!(f, "\"")?;
        for c in data.as_bytes() {
            if *c >= 32u8 && *c <= 127u8 {
                write!(f, "{}", *c as char)?;
            } else {
                write!(f, "\\x{:02x}", c)?;
            }
        }
        write!(f, "\"")
    }
}

fn write_encoded_string<W: Write>(f: &mut W, data: &str) -> std::fmt::Result {
    // If the value is empty, then it should be written as `""`
    if data.is_empty() {
        write!(f, "\"\"")
    } else {
        write!(f, "{}", data)
    }
}

fn write_u16<W: Write>(f: &mut W, data: &u16) -> std::fmt::Result {
    // If the value is max, then `nil` should written
    if *data == u16::MAX {
        write!(f, "nil")
    } else {
        write!(f, "{}", data)
    }
}

fn write_u32<W: Write>(f: &mut W, data: &u32) -> std::fmt::Result {
    // If the value is max, then `nil` should written
    if *data == u32::MAX {
        write!(f, "nil")
    } else {
        write!(f, "{}", data)
    }
}

fn write_u64<W: Write>(f: &mut W, data: &u64) -> std::fmt::Result {
    // If the value is max, then `nil` should written
    if *data == u64::MAX {
        write!(f, "nil")
    } else {
        write!(f, "{}", data)
    }
}

fn write_float<W: Write>(f: &mut W, data: &f32) -> std::fmt::Result {
    // Ref: https://github.com/TheLazyTomcat/SII_Decrypt/blob/d1cd7921d4667de895288c7227c58df43b63bd21/Source/SII_Decode_Utils.pas#L48
    if data.trunc() == *data && data.abs() <= 1e7 {
        write!(f, "{}", data)
    } else {
        // Rust doesn't like to print floats in hex format, so we need to convert it to u32 first
        write!(f, "&{:x}", unsafe {
            std::mem::transmute::<&f32, &u32>(data)
        })
    }
}

fn write_vec2<W: Write, T>(
    f: &mut W,
    data: &(T, T),
    format_fn: impl Fn(&mut W, &T) -> std::fmt::Result,
) -> std::fmt::Result {
    write!(f, "(")?;
    format_fn(f, &data.0)?;
    write!(f, ", ")?;
    format_fn(f, &data.1)?;
    write!(f, ")")
}

fn write_vec3<W: Write, T>(
    f: &mut W,
    data: &(T, T, T),
    format_fn: impl Fn(&mut W, &T) -> std::fmt::Result,
) -> std::fmt::Result {
    write!(f, "(")?;
    format_fn(f, &data.0)?;
    write!(f, ", ")?;
    format_fn(f, &data.1)?;
    write!(f, ", ")?;
    format_fn(f, &data.2)?;
    write!(f, ")")
}

fn write_float_vec4<W: Write>(
    f: &mut W,
    (f1, f2, f3, f4): &(f32, f32, f32, f32),
) -> std::fmt::Result {
    // https://github.com/TheLazyTomcat/SII_Decrypt/blob/d1cd7921d4667de895288c7227c58df43b63bd21/Source/ValueNodes/SII_Decode_ValueNode_00000018.pas#L96
    write!(f, "(")?;
    write_float(f, f1)?;
    write!(f, "; ")?;
    write_float(f, f2)?;
    write!(f, ", ")?;
    write_float(f, f3)?;
    write!(f, ", ")?;
    write_float(f, f4)?;
    write!(f, ")")
}

fn write_float_vec8<W: Write>(
    f: &mut W,
    (f1, f2, f3, f4, f5, f6, f7, f8): &(f32, f32, f32, f32, f32, f32, f32, f32),
) -> std::fmt::Result {
    // https://github.com/TheLazyTomcat/SII_Decrypt/blob/d1cd7921d4667de895288c7227c58df43b63bd21/Source/ValueNodes/SII_Decode_ValueNode_0000001A.pas#L124
    // https://github.com/TheLazyTomcat/SII_Decrypt/blob/d1cd7921d4667de895288c7227c58df43b63bd21/Source/ValueNodes/SII_Decode_ValueNode_00000019.pas#L57
    let coef = f4.trunc() as i32;
    let f1_ = f1 + (((coef & 0xfff) - 2048) << 9) as f32;
    let f3_ = f3 + ((((coef >> 12) & 0xfff) - 2048) << 9) as f32;
    write!(f, "(")?;
    write_float(f, &f1_)?;
    write!(f, ", ")?;
    write_float(f, f2)?;
    write!(f, ", ")?;
    write_float(f, &f3_)?;
    write!(f, ") (")?;
    write_float(f, f5)?;
    write!(f, "; ")?;
    write_float(f, f6)?;
    write!(f, ", ")?;
    write_float(f, f7)?;
    write!(f, ", ")?;
    write_float(f, f8)?;
    write!(f, ")")
}

fn write_scalar_data_value<W: Write>(
    f: &mut W,
    data: &DataValue<'_>,
    value_prototype: &ValuePrototype<'_>,
) -> std::fmt::Result {
    match data {
        DataValue::String(s) => write_string(f, s),
        DataValue::EncodedString(s) => write_encoded_string(f, s),
        DataValue::Float(float) => write_float(f, float),
        DataValue::FloatVec2(data) => write_vec2(f, data, |f, float| write_float(f, float)),
        DataValue::FloatVec3(data) => write_vec3(f, data, |f, float| write_float(f, float)),
        DataValue::FloatVec4(data) => write_float_vec4(f, data),
        DataValue::FloatVec8(data) => write_float_vec8(f, data),
        DataValue::Int32(i) => {
            write!(f, "{}", i)
        }
        DataValue::Int64(i) => {
            write!(f, "{}", i)
        }
        DataValue::Int32Vec3(data) => write_vec3(f, data, |f, i| write!(f, "{}", i)),
        DataValue::UInt16(u) => write_u16(f, u),
        DataValue::UInt32(u) => write_u32(f, u),
        DataValue::UInt64(u) => write_u64(f, u),
        DataValue::Id(id) => {
            write!(f, "{}", id)
        }
        DataValue::Bool(b) => {
            write!(f, "{}", b)
        }
        DataValue::Enum(e) => {
            let enum_string = value_prototype
                .enum_values
                .as_ref()
                .unwrap()
                .get(e)
                .unwrap();
            write_string(f, enum_string)
        }
        _ => {
            panic!("Unexpected data type {:?}", data);
        }
    }
}

fn write_vector_data_value_single<'a, W: Write, T>(
    f: &mut W,
    name: &'a str,
    data: &'a [T],
    format_fn: impl Fn(&mut W, &T) -> std::fmt::Result,
) -> std::fmt::Result {
    for (i, value) in data.iter().enumerate() {
        write!(f, "  {}[{}]: ", name, i)?;
        format_fn(f, value)?;
        writeln!(f)?;
    }
    Ok(())
}

fn write_vector_data_value<W: Write>(
    f: &mut W,
    data: &DataValue<'_>,
    value_prototype: &ValuePrototype<'_>,
) -> std::fmt::Result {
    match data {
        DataValue::StringArray(strings) => {
            write_vector_data_value_single(f, value_prototype.name, strings, |f, s| {
                write_string(f, s)
            })
        }
        DataValue::EncodedStringArray(strings) => {
            write_vector_data_value_single(f, value_prototype.name, strings, |f, s| {
                write_encoded_string(f, s)
            })
        }
        DataValue::IdArray(ids) => {
            write_vector_data_value_single(f, value_prototype.name, ids, |f, id| {
                write!(f, "{}", id)
            })
        }
        DataValue::FloatArray(floats) => {
            write_vector_data_value_single(f, value_prototype.name, floats, write_float)
        }
        DataValue::FloatVec3Array(floatvecs) => {
            write_vector_data_value_single(f, value_prototype.name, floatvecs, |f, data| {
                write_vec3(f, data, |f, float| write_float(f, float))
            })
        }
        DataValue::FloatVec4Array(floatvecs) => {
            write_vector_data_value_single(f, value_prototype.name, floatvecs, |f, data| {
                write_float_vec4(f, data)
            })
        }
        DataValue::FloatVec8Array(floatvecs) => {
            write_vector_data_value_single(f, value_prototype.name, floatvecs, write_float_vec8)
        }
        DataValue::Int32Array(ints) => {
            write_vector_data_value_single(f, value_prototype.name, ints, |f, i| write!(f, "{}", i))
        }
        DataValue::Int32Vec3Array(intvecs) => {
            write_vector_data_value_single(f, value_prototype.name, intvecs, |f, data| {
                write_vec3(f, data, |f, i| write!(f, "{}", i))
            })
        }
        DataValue::Int64Array(ints) => {
            write_vector_data_value_single(f, value_prototype.name, ints, |f, i| write!(f, "{}", i))
        }
        DataValue::UInt16Array(uints) => {
            write_vector_data_value_single(f, value_prototype.name, uints, write_u16)
        }
        DataValue::UInt32Array(uints) => {
            write_vector_data_value_single(f, value_prototype.name, uints, write_u32)
        }
        DataValue::UInt64Array(uints) => {
            write_vector_data_value_single(f, value_prototype.name, uints, write_u64)
        }
        DataValue::BoolArray(bools) => {
            write_vector_data_value_single(f, value_prototype.name, bools, |f, b| {
                write!(f, "{}", b)
            })
        }
        _ => {
            eprintln!("Unexpected data type {:?}", data);
            Ok(())
        }
    }
}

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
        write!(f, "  {}: ", value_prototype.name)?;
        if data.is_array() {
            // First write the length of the array
            writeln!(f, "{}", data.get_array_length().unwrap())?;
            write_vector_data_value(f, data, value_prototype)?;
        } else {
            // Write the scalar value
            write_scalar_data_value(f, data, value_prototype)?;
            writeln!(f)?;
        }
    }
    writeln!(f, "}}")?;
    Ok(())
}

fn write_bsii<W: Write>(f: &mut W, bsii: &BsiiFile) -> std::fmt::Result {
    writeln!(f, "SiiNunit")?;
    writeln!(f, "{{")?;
    for data_block in &bsii.data_blocks {
        let prototype = bsii.get_prototype(data_block.prototype_id).unwrap();
        write_data_block(f, data_block, prototype)?;
    }
    writeln!(f, "}}")?;
    Ok(())
}

impl BsiiFile<'_> {
    /// Write the BSII file to a SIIN string
    pub fn to_siin(&self) -> String {
        let mut output = String::new();
        write_bsii(&mut output, self).unwrap();
        output
    }
}

impl fmt::Display for Id {
    // https://github.com/TheLazyTomcat/SII_Decrypt/blob/d1cd7921d4667de895288c7227c58df43b63bd21/Source/SII_Decode_Utils.pas#L183
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Id::Nameless(id) => {
                write!(f, "_nameless")?;
                if *id == 0 {
                    write!(f, ".0")?;
                } else {
                    let mut bit_shift = 48;
                    let mut has_first_part = false;
                    while bit_shift >= 0 {
                        let mask = 0xffff << bit_shift;
                        let part = ((*id & mask) >> bit_shift) as u16;
                        bit_shift -= 16;
                        if !has_first_part && part != 0 {
                            has_first_part = true;
                            write!(f, ".{:x}", part)?;
                        } else if has_first_part {
                            write!(f, ".{:04x}", part)?;
                        }
                    }
                }
                Ok(())
            }
            Id::Named(parts) => {
                if parts.is_empty() {
                    write!(f, "null")
                } else {
                    write!(f, "{}", parts.join("."))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_print_test() {
        let id = Id::Nameless(0x0807060504030201u64);
        assert_eq!(id.to_string(), "_nameless.807.0605.0403.0201");
        let id = Id::Nameless(0x060504030201u64);
        assert_eq!(id.to_string(), "_nameless.605.0403.0201");
        let id = Id::Nameless(0);
        assert_eq!(id.to_string(), "_nameless.0");
        let id = Id::Named(vec!["first".to_string(), "second".to_string()]);
        assert_eq!(id.to_string(), "first.second");
        let id = Id::Named(vec![]);
        assert_eq!(id.to_string(), "null");
    }
}
