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
            Err(error) => {
                if error.input == content {
                    Err(ParseError::InvalidHeader)
                } else {
                    Err(ParseError::InvalidInput)
                }
            }
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
