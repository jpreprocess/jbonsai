use std::ops::{Range, RangeFrom, RangeTo};

use nom::{
    bytes::complete::{take_while, take_while1},
    error::{ErrorKind, ParseError},
    IResult,
};

pub trait ParseTarget
where
    Self: Sized
        + Clone
        + nom::Slice<Range<usize>>
        + nom::Slice<RangeFrom<usize>>
        + nom::Slice<RangeTo<usize>>
        + nom::InputIter
        + nom::InputLength
        + nom::InputTake
        + nom::Compare<&'static str>,
{
    fn parse_identifier<E: ParseError<Self>>(self) -> IResult<Self, Self, E>;
    fn parse_ascii<E: ParseError<Self>>(self) -> IResult<Self, Self, E>;
    fn parse_ascii_to_string<E: ParseError<Self>>(&self) -> IResult<Self, String, E>;
}

impl ParseTarget for &str {
    fn parse_identifier<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        take_while1(|c: char| c.is_alphanumeric() || c == '_')(self)
    }

    fn parse_ascii<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        take_while(|c: char| c.is_ascii() && c != '\n')(self)
    }

    fn parse_ascii_to_string<E: ParseError<Self>>(&self) -> IResult<Self, String, E> {
        Self::parse_ascii(self).map(|(rest, result)| (rest, result.to_string()))
    }
}

impl ParseTarget for &[u8] {
    fn parse_identifier<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        take_while1(|c: u8| (c as char).is_alphanumeric() || c == b'_')(self)
    }

    fn parse_ascii<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        take_while(|c: u8| (c as char).is_ascii() && c != b'\n')(self)
    }

    fn parse_ascii_to_string<E: ParseError<Self>>(&self) -> IResult<Self, String, E> {
        Self::parse_ascii(self).and_then(|(rest, result)| {
            match String::from_utf8(result.to_vec()) {
                Ok(s) => Ok((rest, s)),
                Err(_) => Err(nom::Err::Failure(E::from_error_kind(
                    result,
                    ErrorKind::Char,
                ))),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::ParseTarget;

    #[test]
    fn ascii() {
        assert_eq!(
            "hogehoge\"\nfugafuga".parse_ascii::<VerboseError<&str>>(),
            Ok(("\nfugafuga", "hogehoge\""))
        );
    }
}
