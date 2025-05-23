//! References:
//! <https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Documents/Binary%20SII%20-%20Format.txt>
//! <https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Documents/Binary%20SII%20-%20Types.txt>

use std::collections::HashMap;
use std::str;

use nom::bytes::complete::{tag, take};
use nom::combinator::{fail, map};
use nom::multi::{count, many_till};
use nom::number::complete::{le_f32, le_i32, le_i64, le_u16, le_u32, le_u64, le_u8};
use nom::sequence::{pair, tuple};
use nom::Finish;
use nom::IResult;

use log::debug;

use crate::bsii_file::BsiiFile;
use crate::bsii_file::DataBlock;
use crate::bsii_file::DataValue;
use crate::bsii_file::Id;
use crate::bsii_file::Prototype;
use crate::bsii_file::ValuePrototype;

impl DataValue<'_> {
    pub fn is_array(&self) -> bool {
        matches!(
            self,
            DataValue::StringArray(_)
                | DataValue::EncodedStringArray(_)
                | DataValue::FloatArray(_)
                | DataValue::FloatVec3Array(_)
                | DataValue::Int32Vec3Array(_)
                | DataValue::FloatVec4Array(_)
                | DataValue::FloatVec8Array(_)
                | DataValue::Int32Array(_)
                | DataValue::UInt32Array(_)
                | DataValue::UInt16Array(_)
                | DataValue::Int64Array(_)
                | DataValue::UInt64Array(_)
                | DataValue::BoolArray(_)
                | DataValue::IdArray(_)
        )
    }

    pub fn get_array_length(&self) -> Option<usize> {
        match &self {
            DataValue::StringArray(array) => Some(array.len()),
            DataValue::EncodedStringArray(array) => Some(array.len()),
            DataValue::FloatArray(array) => Some(array.len()),
            DataValue::FloatVec3Array(array) => Some(array.len()),
            DataValue::Int32Vec3Array(array) => Some(array.len()),
            DataValue::FloatVec4Array(array) => Some(array.len()),
            DataValue::FloatVec8Array(array) => Some(array.len()),
            DataValue::Int32Array(array) => Some(array.len()),
            DataValue::UInt32Array(array) => Some(array.len()),
            DataValue::UInt16Array(array) => Some(array.len()),
            DataValue::Int64Array(array) => Some(array.len()),
            DataValue::UInt64Array(array) => Some(array.len()),
            DataValue::BoolArray(array) => Some(array.len()),
            DataValue::IdArray(array) => Some(array.len()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidHeader,
    InvalidInput,
    UnsupportedVersion,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidHeader => write!(f, "Invalid header"),
            ParseError::InvalidInput => write!(f, "Invalid input"),
            ParseError::UnsupportedVersion => write!(f, "Unsupported version"),
        }
    }
}

impl<'a> BsiiFile<'a> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        match bsii_parser(content).finish() {
            Ok((_, bsii_file)) => Ok(bsii_file),
            Err(error) => {
                if error.input.len() == content.len() {
                    // If the error input is the same as the original input, it means that the header
                    // is invalid
                    Err(ParseError::InvalidHeader)
                } else if error.input.len() == content.len() - 8
                    && error.code == nom::error::ErrorKind::Fail
                {
                    // We got a `fail` from checking the version
                    Err(ParseError::UnsupportedVersion)
                } else {
                    print!("Error: {:?}", error);
                    Err(ParseError::InvalidInput)
                }
            }
        }
    }

    pub(crate) fn get_prototype(&self, id: u32) -> Option<&Prototype<'a>> {
        self.prototypes.get(&id)
    }
}

fn bsii_parser(input: &[u8]) -> IResult<&[u8], BsiiFile<'_>> {
    let (input, header) = tag("BSII")(input)?;
    let (input, version) = le_u32(input)?;
    if version == 1 {
        // Doesn't support version 1
        // It has differing behaviour with value type 0x19
        return fail(input);
    }
    let mut prototypes = HashMap::new();
    let mut data_blocks = Vec::new();
    // TODO: Rewrite the loop using combinators
    let mut loop_input = input;
    loop {
        // Peek block id
        let (peek_input, block_id) = le_u32(loop_input)?;
        if block_id == 0 {
            // Peek validity bit
            let (peek_input, validity) = take(1usize)(peek_input)?;
            if validity[0] == 0 {
                // let (peek_input, _) = eof(peek_input)?;
                return Ok((
                    peek_input,
                    BsiiFile {
                        header,
                        version,
                        prototypes,
                        data_blocks,
                    },
                ));
            } else {
                let (next_input, prototype) = prototype_parser(loop_input)?;
                debug!("Parsed prototype {}", prototype.name);
                prototypes.insert(prototype.id, prototype);
                loop_input = next_input;
            }
        } else {
            let (next_input, data_block) = data_block_parser(loop_input, &prototypes)?;
            debug!(
                "Parsed data block with prototype {}, ID {}",
                prototypes.get(&data_block.prototype_id).unwrap().name,
                data_block.id
            );
            data_blocks.push(data_block);
            loop_input = next_input;
        }
    }
}

fn str_parser(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, len) = le_u32(input)?;
    let (input, data) = take(len)(input)?;
    Ok((input, str::from_utf8(data).unwrap()))
}

const CHAR_ENCODINGS: [char; 37] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '_',
];

fn encoded_str_parser(input: &[u8]) -> IResult<&[u8], String> {
    let (input, encoded_data) = le_u64(input)?;
    let mut remaining = encoded_data & !(1u64 << 63);
    let mut chars: Vec<char> = Vec::new();
    while remaining > 0 {
        let last = remaining % 38;
        remaining /= 38;
        chars.push(CHAR_ENCODINGS[last as usize - 1]);
    }
    Ok((input, chars.into_iter().collect()))
}

fn value_prototype_parser(input: &[u8]) -> IResult<&[u8], ValuePrototype<'_>> {
    let (input, type_id) = le_u32(input)?;
    if type_id == 0 {
        fail(input)
    } else {
        let (input, name) = str_parser(input)?;
        let (input, enum_values) = if type_id == 0x37u32 {
            // parse enum values
            let (input, enum_values_length) = le_u32(input)?;
            let (input, enum_values_vec) =
                count(pair(le_u32, str_parser), enum_values_length as usize)(input)?;
            let enum_values = HashMap::from_iter(enum_values_vec);
            (input, Some(enum_values))
        } else {
            (input, None)
        };
        debug!("Parsed prototype value {} type_id {:x}", name, type_id);
        Ok((
            input,
            ValuePrototype {
                type_id,
                name,
                enum_values,
            },
        ))
    }
}

fn prototype_parser(input: &[u8]) -> IResult<&[u8], Prototype<'_>> {
    let (input, _) = tag("\0\0\0\0")(input)?;
    let (input, _) = tag(&[0x01])(input)?;
    let (input, id) = le_u32(input)?;
    let (input, name) = str_parser(input)?;
    let (input, (value_prototypes, _)) = many_till(value_prototype_parser, tag("\0\0\0\0"))(input)?;
    Ok((
        input,
        Prototype {
            id,
            name,
            value_prototypes,
        },
    ))
}

fn id_parser(input: &[u8]) -> IResult<&[u8], Id> {
    let (input, length) = le_u8(input)?;
    if length == 0xff {
        let (input, nameless_id) = le_u64(input)?;
        Ok((input, Id::Nameless(nameless_id)))
    } else {
        let (input, parts) = count(encoded_str_parser, length as usize)(input)?;
        Ok((input, Id::Named(parts)))
    }
}

fn value_parser(input: &[u8], type_id: u32) -> IResult<&[u8], DataValue<'_>> {
    match type_id {
        0x01u32 => {
            // string
            map(str_parser, DataValue::String)(input)
        }
        0x02u32 => {
            // array of string
            let (input, size) = le_u32(input)?;
            map(count(str_parser, size as usize), DataValue::StringArray)(input)
        }
        0x03u32 => {
            // encoded string
            map(encoded_str_parser, DataValue::EncodedString)(input)
        }
        0x04u32 => {
            // array of encoded string
            let (input, size) = le_u32(input)?;
            map(
                count(encoded_str_parser, size as usize),
                DataValue::EncodedStringArray,
            )(input)
        }
        0x05u32 => {
            // float
            map(le_f32, DataValue::Float)(input)
        }
        0x06u32 => {
            // array of float
            let (input, size) = le_u32(input)?;
            map(count(le_f32, size as usize), DataValue::FloatArray)(input)
        }
        0x07u32 => {
            // vec2 of float
            map(pair(le_f32, le_f32), DataValue::FloatVec2)(input)
        }
        0x09u32 => {
            // vec3 of float
            map(tuple((le_f32, le_f32, le_f32)), DataValue::FloatVec3)(input)
        }
        0x11u32 => {
            // vec3 of int32
            map(tuple((le_i32, le_i32, le_i32)), DataValue::Int32Vec3)(input)
        }
        0x12u32 => {
            // array of vec3 of int32
            let (input, size) = le_u32(input)?;
            map(
                count(tuple((le_i32, le_i32, le_i32)), size as usize),
                DataValue::Int32Vec3Array,
            )(input)
        }
        0x18u32 => {
            // array of vec4 of float
            let (input, size) = le_u32(input)?;
            map(
                count(tuple((le_f32, le_f32, le_f32, le_f32)), size as usize),
                DataValue::FloatVec4Array,
            )(input)
        }
        0x19u32 => {
            // vec8 of float
            map(
                tuple((
                    le_f32, le_f32, le_f32, le_f32, le_f32, le_f32, le_f32, le_f32,
                )),
                DataValue::FloatVec8,
            )(input)
        }
        0x1au32 => {
            // array of vec8 of float
            let (input, size) = le_u32(input)?;
            map(
                count(
                    tuple((
                        le_f32, le_f32, le_f32, le_f32, le_f32, le_f32, le_f32, le_f32,
                    )),
                    size as usize,
                ),
                DataValue::FloatVec8Array,
            )(input)
        }
        0x25u32 => {
            // int32
            map(le_i32, DataValue::Int32)(input)
        }
        0x26u32 => {
            // array of int32
            let (input, size) = le_u32(input)?;
            map(count(le_i32, size as usize), DataValue::Int32Array)(input)
        }
        0x27u32 | 0x2fu32 => {
            // uint32
            map(le_u32, DataValue::UInt32)(input)
        }
        0x28u32 => {
            // array of uint32
            let (input, size) = le_u32(input)?;
            map(count(le_u32, size as usize), DataValue::UInt32Array)(input)
        }
        0x2bu32 => {
            // uint16
            map(le_u16, DataValue::UInt16)(input)
        }
        0x2cu32 => {
            // array of uint16
            let (input, size) = le_u32(input)?;
            map(count(le_u16, size as usize), DataValue::UInt16Array)(input)
        }
        0x31u32 => {
            // int64
            map(le_i64, DataValue::Int64)(input)
        }
        0x32u32 => {
            // array of int64
            let (input, size) = le_u32(input)?;
            map(count(le_i64, size as usize), DataValue::Int64Array)(input)
        }
        0x33u32 => {
            // uint64
            map(le_u64, DataValue::UInt64)(input)
        }
        0x34u32 => {
            // array of uint64
            let (input, size) = le_u32(input)?;
            map(count(le_u64, size as usize), DataValue::UInt64Array)(input)
        }
        0x35u32 => {
            // bool
            map(le_u8, |v| DataValue::Bool(v != 0))(input)
        }
        0x36u32 => {
            // array of bool
            let (input, size) = le_u32(input)?;
            map(
                count(map(le_u8, |v| v != 0), size as usize),
                DataValue::BoolArray,
            )(input)
        }
        0x37u32 => {
            // enum
            map(le_u32, DataValue::Enum)(input)
        }
        0x39u32 | 0x3bu32 | 0x3du32 => {
            // id
            map(id_parser, DataValue::Id)(input)
        }
        0x3au32 | 0x3cu32 => {
            // array of id
            let (input, length) = le_u32(input)?;
            map(count(id_parser, length as usize), DataValue::IdArray)(input)
        }
        _ => todo!("Implement value parse for type_id {:x}", type_id),
    }
}

fn data_block_parser<'a, 'b>(
    input: &'a [u8],
    prototypes: &'b HashMap<u32, Prototype<'b>>,
) -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, prototype_id) = le_u32(input)?;
    if prototype_id == 0 {
        // this is a prototype block, not a data block
        fail(input)
    } else {
        let prototype = prototypes.get(&prototype_id).unwrap();
        let (input, id) = id_parser(input)?;
        // TODO: Try to rewrite the code below in combinators
        let mut data: Vec<DataValue<'a>> = Vec::new();
        let mut loop_input = input;
        for value in &prototype.value_prototypes {
            let (next_input, value) = value_parser(loop_input, value.type_id)?;
            loop_input = next_input;
            data.push(value);
        }
        Ok((
            loop_input,
            DataBlock {
                prototype_id,
                id,
                data,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_parser_works() {
        let test_str: &[u8] = &[
            0x0F, 0x00, 0x00, 0x00, // length of following string,
            0x66, 0x69, 0x72, 0x73, 0x74, 0x5F, 0x73, 0x74, 0x72, 0x75, 0x63, 0x74, 0x75, 0x72,
            0x65, // data
        ];
        match str_parser(test_str) {
            Ok((input, parsed_str)) => {
                assert_eq!(input, &[]);
                assert_eq!(parsed_str, "first_structure");
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn value_prototype_parser_works() {
        let test_value_prototype: &[u8] = &[
            0x25, 0x00, 0x00, 0x00, // value type
            0x0B, 0x00, 0x00, 0x00, // length of following string
            0x69, 0x6E, 0x74, 0x33, 0x32, 0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
        ];
        match value_prototype_parser(test_value_prototype) {
            Ok((input, value_prototype)) => {
                assert_eq!(input, &[]);
                assert_eq!(value_prototype.type_id, 0x25u32);
                assert_eq!(value_prototype.name, "int32_field")
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn prototype_parse_works() {
        let test_prototype: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, // block type
            0x01, // validity
            0x01, 0x00, 0x00, 0x00, // structure ID
            0x0F, 0x00, 0x00, 0x00, // length of following string,
            0x66, 0x69, 0x72, 0x73, 0x74, 0x5F, 0x73, 0x74, 0x72, 0x75, 0x63, 0x74, 0x75, 0x72,
            0x65, // structure name
            0x25, 0x00, 0x00, 0x00, // value type
            0x0B, 0x00, 0x00, 0x00, // length of following string
            0x69, 0x6E, 0x74, 0x33, 0x32, 0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
            0x36, 0x00, 0x00, 0x00, // value type
            0x14, 0x00, 0x00, 0x00, // length of following string
            0x62, 0x79, 0x74, 0x65, 0x62, 0x6F, 0x6F, 0x6C, 0x5F, 0x61, 0x72, 0x72, 0x61, 0x79,
            0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
            0x34, 0x00, 0x00, 0x00, // value type
            0x18, 0x00, 0x00, 0x00, // length of following string
            0x65, 0x6D, 0x70, 0x74, 0x79, 0x5F, 0x75, 0x69, 0x6E, 0x74, 0x36, 0x34, 0x5F, 0x61,
            0x72, 0x72, 0x61, 0x79, 0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
            0x00, 0x00, 0x00, 0x00, // value type
        ];
        match prototype_parser(test_prototype) {
            Ok((input, prototype)) => {
                assert_eq!(input, &[]);
                assert_eq!(prototype.id, 0x01u32);
                assert_eq!(prototype.name, "first_structure");
                assert_eq!(prototype.value_prototypes.len(), 3);
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn prototype_parse_works_2() {
        let test_prototype: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, // block type
            0x01, // validity
            0x02, 0x00, 0x00, 0x00, // structure ID
            0x04, 0x00, 0x00, 0x00, // length of following string
            0x6C, 0x61, 0x73, 0x74, // structure name
            0x05, 0x00, 0x00, 0x00, // value type
            0x0C, 0x00, 0x00, 0x00, // length of following string
            0x73, 0x69, 0x6E, 0x67, 0x6C, 0x65, 0x5F, 0x66, 0x69, 0x65, 0x6C,
            0x64, // value name
            0x00, 0x00, 0x00, 0x00, // value type
        ];
        match prototype_parser(test_prototype) {
            Ok((input, prototype)) => {
                assert_eq!(input, &[]);
                assert_eq!(prototype.id, 0x02u32);
                assert_eq!(prototype.name, "last");
                assert_eq!(prototype.value_prototypes.len(), 1);
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn data_block_parse_works() {
        let prototype = Prototype {
            id: 1,
            name: "first_structure",
            value_prototypes: vec![
                ValuePrototype {
                    type_id: 37,
                    name: "int32_field",
                    enum_values: None,
                },
                ValuePrototype {
                    type_id: 54,
                    name: "bytebool_array_field",
                    enum_values: None,
                },
                ValuePrototype {
                    type_id: 52,
                    name: "empty_uint64_array_field",
                    enum_values: None,
                },
            ],
        };
        let prototypes = HashMap::from([(1, prototype)]);
        let test_data_block: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // block type
            0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // block ID
            0xFF, 0xFF, 0xFF, 0xFF, // Int32 value
            0x03, 0x00, 0x00, 0x00, // length of the following array
            0x00, 0x01, 0x00, // array of ByteBool
            0x00, 0x00, 0x00, 0x00, // length of the following array
        ];
        match data_block_parser(test_data_block, &prototypes) {
            Ok((input, data_block)) => {
                assert_eq!(input, &[]);
                assert_eq!(data_block.prototype_id, 1);
                assert_eq!(data_block.id, Id::Nameless(0x0807060504030201u64));
                assert_eq!(data_block.data.len(), 3);
                assert_eq!(data_block.data[0], DataValue::Int32(-1));
                assert_eq!(
                    data_block.data[1],
                    DataValue::BoolArray(vec![false, true, false])
                );
                assert_eq!(data_block.data[2], DataValue::UInt64Array(vec![]));
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn data_block_parse_works_2() {
        let prototype = Prototype {
            id: 2,
            name: "last",
            value_prototypes: vec![ValuePrototype {
                type_id: 5,
                name: "single_field",
                enum_values: None,
            }],
        };
        let prototypes = HashMap::from([(2, prototype)]);
        let test_data_block: &[u8] = &[
            0x02, 0x00, 0x00, 0x00, // block type
            0xFF, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF, // block ID
            0x00, 0x00, 0x80, 0x3F, // single value
        ];
        match data_block_parser(test_data_block, &prototypes) {
            Ok((input, data_block)) => {
                assert_eq!(input, &[]);
                assert_eq!(data_block.prototype_id, 2);
                assert_eq!(data_block.id, Id::Nameless(0xfffefdfcfbfaf9f8u64));
                assert_eq!(data_block.data.len(), 1);
                assert_eq!(data_block.data[0], DataValue::Float(1.0f32));
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn bsii_parser_works() {
        // From https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Documents/Binary%20SII%20-%20Format.txt
        let test_data: &[u8] = &[
            0x42, 0x53, 0x49, 0x49, // file signature
            0x02, 0x00, 0x00, 0x00, // format version
            0x00, 0x00, 0x00, 0x00, // block type
            0x01, // validity
            0x01, 0x00, 0x00, 0x00, // structure ID
            0x0F, 0x00, 0x00, 0x00, // length of following string,
            0x66, 0x69, 0x72, 0x73, 0x74, 0x5F, 0x73, 0x74, 0x72, 0x75, 0x63, 0x74, 0x75, 0x72,
            0x65, // structure name
            0x25, 0x00, 0x00, 0x00, // value type
            0x0B, 0x00, 0x00, 0x00, // length of following string
            0x69, 0x6E, 0x74, 0x33, 0x32, 0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
            0x36, 0x00, 0x00, 0x00, // value type
            0x14, 0x00, 0x00, 0x00, // length of following string
            0x62, 0x79, 0x74, 0x65, 0x62, 0x6F, 0x6F, 0x6C, 0x5F, 0x61, 0x72, 0x72, 0x61, 0x79,
            0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
            0x34, 0x00, 0x00, 0x00, // value type
            0x18, 0x00, 0x00, 0x00, // length of following string
            0x65, 0x6D, 0x70, 0x74, 0x79, 0x5F, 0x75, 0x69, 0x6E, 0x74, 0x36, 0x34, 0x5F, 0x61,
            0x72, 0x72, 0x61, 0x79, 0x5F, 0x66, 0x69, 0x65, 0x6C, 0x64, // value name
            0x00, 0x00, 0x00, 0x00, // value type
            0x00, 0x00, 0x00, 0x00, // block type
            0x01, // validity
            0x02, 0x00, 0x00, 0x00, // structure ID
            0x04, 0x00, 0x00, 0x00, // length of following string
            0x6C, 0x61, 0x73, 0x74, // structure name
            0x05, 0x00, 0x00, 0x00, // value type
            0x0C, 0x00, 0x00, 0x00, // length of following string
            0x73, 0x69, 0x6E, 0x67, 0x6C, 0x65, 0x5F, 0x66, 0x69, 0x65, 0x6C,
            0x64, // value name
            0x00, 0x00, 0x00, 0x00, // value type
            0x01, 0x00, 0x00, 0x00, // block type
            0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // block ID
            0xFF, 0xFF, 0xFF, 0xFF, // Int32 value
            0x03, 0x00, 0x00, 0x00, // length of the following array
            0x00, 0x01, 0x00, // array of ByteBool
            0x00, 0x00, 0x00, 0x00, // length of the following array
            0x02, 0x00, 0x00, 0x00, // block type
            0xFF, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF, // block ID
            0x00, 0x00, 0x80, 0x3F, // single value
            0x00, 0x00, 0x00, 0x00, // block type
            0x00, // validity
        ];
        match bsii_parser(test_data) {
            Ok((input, bsiifile)) => {
                assert_eq!(input, &[]);
                assert_eq!(bsiifile.header, &[0x42, 0x53, 0x49, 0x49]);
                assert_eq!(bsiifile.version, 2u32);
                assert_eq!(bsiifile.prototypes.len(), 2);
                assert_eq!(bsiifile.data_blocks.len(), 2);
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn bsii_parser_raises_error_on_invalid_header() {
        let test_data: &[u8] = &[0x42, 0x53, 0x49, 0x00];
        match BsiiFile::parse(test_data) {
            Ok(_) => panic!("Should have raised an error"),
            Err(ParseError::InvalidHeader) => {}
            Err(err) => panic!("Should have raised an InvalidHeader error, got {:?}", err),
        }
    }

    #[test]
    fn bsii_parser_raises_error_on_unsupported_version() {
        // Version 1 is not supported
        let test_data: &[u8] = &[0x42, 0x53, 0x49, 0x49, 0x01, 0x00, 0x00, 0x00];
        match BsiiFile::parse(test_data) {
            Ok(_) => panic!("Should have raised an error"),
            Err(ParseError::UnsupportedVersion) => {}
            Err(err) => panic!(
                "Should have raised an UnsupportedVersion error, got {:?}",
                err
            ),
        }
    }

    #[test]
    fn bsii_parser_raises_error_on_invalid_input() {
        // File is incomplete
        let test_data: &[u8] = &[0x42, 0x53, 0x49, 0x49, 0x02, 0x00, 0x00, 0x00];
        match BsiiFile::parse(test_data) {
            Ok(_) => panic!("Should have raised an error"),
            Err(ParseError::InvalidInput) => {}
            Err(err) => panic!("Should have raised an InvalidInput error, got {:?}", err),
        }
    }
}
