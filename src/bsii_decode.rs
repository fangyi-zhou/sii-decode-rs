// Refs:
// https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Documents/Binary%20SII%20-%20Format.txt
// https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Documents/Binary%20SII%20-%20Types.txt

use std::collections::HashMap;
use std::str;

use nom::bytes::complete::{tag, take};
use nom::combinator::{fail, peek};
use nom::multi::{count, many_till};
use nom::number::complete::{le_u32, le_u64};
use nom::sequence::{pair, preceded};
use nom::IResult;

use log::info;

pub struct BsiiFile<'a> {
    header: &'a [u8], // BSII,
    version: u32,
    prototypes: HashMap<u32, Prototype<'a>>,
    data_blocks: Vec<DataBlock<'a>>,
}

struct Prototype<'a> {
    // valid prototypes only
    id: u32,
    name: &'a str,
    value_prototypes: Vec<ValuePrototype<'a>>,
}

struct ValuePrototype<'a> {
    type_id: u32,
    name: &'a str,
    // enum values are only used when type_id is 0x37
    enum_values: Option<HashMap<u32, &'a str>>,
}

struct DataBlock<'a> {
    type_id: u32,
    id: u64,
    data: Vec<&'a [u8]>, // TODO: Decode values
}

impl<'a> BsiiFile<'a> {
    pub fn from_content(content: &'a [u8]) -> Self {
        let (_, bsii_file) = bsii_parser(content).unwrap();
        bsii_file
    }
}

fn bsii_parser(input: &[u8]) -> IResult<&[u8], BsiiFile<'_>> {
    let (input, header) = tag("BSII")(input)?;
    let (input, version) = le_u32(input)?;
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
                let (next_input, prototype) = prototype_parse(loop_input)?;
                info!("Parsed prototype {}", prototype.name);
                prototypes.insert(prototype.id, prototype);
                loop_input = next_input;
            }
        } else {
            let (next_input, data_block) = data_block_parse(loop_input, &prototypes)?;
            info!("Parsed data block {:X}", data_block.id);
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
        info!("Parsed prototype value {}", name);
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

fn prototype_parse(input: &[u8]) -> IResult<&[u8], Prototype<'_>> {
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

fn value_parse(input: &[u8], type_id: u32) -> IResult<&[u8], &[u8]> {
    // TODO: Parse into values instead of u8 slices
    match type_id {
        0x05u32 => {
            // single
            take(4usize)(input)
        }
        0x25u32 => {
            // int32
            take(4usize)(input)
        }
        0x34u32 => {
            // array of uint64
            let (input, size) = peek(le_u32)(input)?;
            take(size as usize * 8 + 4usize)(input)
        }
        0x36u32 => {
            // array of ByteBool
            let (input, size) = peek(le_u32)(input)?;
            take(size as usize + 4usize)(input)
        }
        _ => todo!("Implement value parse for type_id {}", type_id),
    }
}

fn data_block_parse<'a, 'b>(
    input: &'a [u8],
    prototypes: &'b HashMap<u32, Prototype<'b>>,
) -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, type_id) = le_u32(input)?;
    if type_id == 0 {
        // this is a prototype block, not a data block
        fail(input)
    } else {
        let prototype = prototypes.get(&type_id).unwrap();
        let (input, id) = preceded(tag(&[0xff]), le_u64)(input)?;
        // TODO: Try to rewrite the code below in combinators
        let mut data: Vec<&'a [u8]> = Vec::new();
        let mut loop_input = input;
        for value in &prototype.value_prototypes {
            let (next_input, value) = value_parse(loop_input, value.type_id)?;
            loop_input = next_input;
            data.push(value);
        }
        Ok((loop_input, DataBlock { type_id, id, data }))
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
        match str_parser(&test_str) {
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
        match prototype_parse(test_prototype) {
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
        match prototype_parse(test_prototype) {
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
            name: &"first_structure",
            value_prototypes: vec![
                ValuePrototype {
                    type_id: 37,
                    name: &"int32_field",
                    enum_values: None,
                },
                ValuePrototype {
                    type_id: 54,
                    name: &"bytebool_array_field",
                    enum_values: None,
                },
                ValuePrototype {
                    type_id: 52,
                    name: &"empty_uint64_array_field",
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
        match data_block_parse(test_data_block, &prototypes) {
            Ok((input, data_block)) => {
                assert_eq!(input, &[]);
                assert_eq!(data_block.type_id, 1);
                assert_eq!(data_block.id, 0x0807060504030201u64);
                assert_eq!(data_block.data.len(), 3);
                assert_eq!(data_block.data[0], &[0xff, 0xff, 0xff, 0xff]);
                assert_eq!(
                    data_block.data[1],
                    &[0x03, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]
                );
                assert_eq!(data_block.data[2], &[0x00, 0x00, 0x00, 0x00]);
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn data_block_parse_works_2() {
        let prototype = Prototype {
            id: 2,
            name: &"last",
            value_prototypes: vec![ValuePrototype {
                type_id: 5,
                name: &"single_field",
                enum_values: None,
            }],
        };
        let prototypes = HashMap::from([(2, prototype)]);
        let test_data_block: &[u8] = &[
            0x02, 0x00, 0x00, 0x00, // block type
            0xFF, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF, // block ID
            0x00, 0x00, 0x80, 0x3F, // single value
        ];
        match data_block_parse(test_data_block, &prototypes) {
            Ok((input, data_block)) => {
                assert_eq!(input, &[]);
                assert_eq!(data_block.type_id, 2);
                assert_eq!(data_block.id, 0xfffefdfcfbfaf9f8u64);
                assert_eq!(data_block.data.len(), 1);
                assert_eq!(data_block.data[0], &[0x00, 0x00, 0x80, 0x3f]);
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }

    #[test]
    fn it_works() {
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
}
