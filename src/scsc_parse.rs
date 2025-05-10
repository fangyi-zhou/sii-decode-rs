use crate::scsc_file::{ParseError, ScscFile};

use nom::bytes::complete::{tag, take};
use nom::combinator::rest;
use nom::number::complete::le_u32;
use nom::Finish;
use nom::IResult;

impl<'a> ScscFile<'a> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        match scsc_parser(content).finish() {
            Ok((_, scsc_file)) => Ok(scsc_file),
            Err(error) => match error.code {
                nom::error::ErrorKind::Tag => Err(ParseError::InvalidHeader),
                _ => Err(ParseError::InvalidInput),
            },
        }
    }
}

fn scsc_parser(input: &[u8]) -> IResult<&[u8], ScscFile<'_>> {
    let (input, header) = tag("ScsC")(input)?;
    let (input, hmac) = take(32usize)(input)?;
    let (input, iv) = take(16usize)(input)?;
    let (input, size) = le_u32(input)?;
    let (input, data) = rest(input)?;
    Ok((
        input,
        ScscFile {
            header,
            hmac,
            iv,
            size,
            data,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let test_data: &[u8] = &[
            0x53, 0x63, 0x73, 0x43, // ScsC header
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x30, 0x31, 0x32, 0x33,
            0x34, 0x35, 0x36, 0x37, // 32 bytes HMAC
            0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55,
            0x56, 0x57, // 16 bytes IV
            0x04, 0x00, 0x00, 0x00, // size
            0xde, 0xad, 0xbe, 0xef, // some data
        ];
        match scsc_parser(test_data) {
            Ok((input, scscfile)) => {
                assert_eq!(input, &[]);
                assert_eq!(scscfile.header, &[0x53, 0x63, 0x73, 0x43]);
                assert_eq!(
                    scscfile.hmac,
                    &[
                        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x10, 0x11, 0x12, 0x13,
                        0x14, 0x15, 0x16, 0x17, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
                        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                    ]
                );
                assert_eq!(
                    scscfile.iv,
                    &[
                        0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x50, 0x51, 0x52, 0x53,
                        0x54, 0x55, 0x56, 0x57,
                    ]
                );
                assert_eq!(scscfile.size, 4u32);
                assert_eq!(scscfile.data, &[0xde, 0xad, 0xbe, 0xef]);
            }
            Err(err) => panic!("Failed to parse, {}", err),
        }
    }
}
