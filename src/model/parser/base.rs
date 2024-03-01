use std::ops::{Range, RangeFrom, RangeTo};

use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    combinator::{cut, map},
    error::{ErrorKind, ParseError},
    multi::separated_list0,
    sequence::{delimited, pair},
    IResult,
};

use crate::model::stream::Pattern;

const SEPARATOR_CHARS: &str = " \n";
const PATTERN_WILDCARD: &str = "*?";
const JPCOMMON_SYMBOLS: &str = "!#%&+-/:=@^_|";

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
        + nom::InputTakeAtPosition
        + nom::Offset
        + nom::AsBytes
        + nom::ParseTo<isize>
        + nom::ParseTo<usize>
        + nom::ParseTo<f64>
        + nom::Compare<&'static str>
        + for<'a> nom::Compare<&'a [u8]>
        + for<'a> nom::FindSubstring<&'a str>,
    <Self as nom::InputIter>::Item: nom::AsChar,
{
    fn parse_utf8(&self) -> Result<&str, std::str::Utf8Error>;

    fn parse_template<F, E>(self, cond: F) -> IResult<Self, Self, E>
    where
        F: Fn(char) -> bool,
        E: ParseError<Self>;
    fn parse_template1<F, E>(self, cond: F) -> IResult<Self, Self, E>
    where
        F: Fn(char) -> bool,
        E: ParseError<Self>;
    fn parse_ascii_to_string<E: ParseError<Self>>(&self) -> IResult<Self, String, E>;

    #[inline(always)]
    fn sp<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        Self::parse_template(self, |c| SEPARATOR_CHARS.contains(c))
    }
    #[inline(always)]
    fn sp1<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        Self::parse_template1(self, |c| SEPARATOR_CHARS.contains(c))
    }
    #[inline(always)]
    fn parse_identifier<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        Self::parse_template1(self, |c: char| {
            c.is_ascii() && (c.is_alphanumeric() || c == '_')
        })
    }
    #[inline(always)]
    fn parse_pattern<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        Self::parse_template1(self, |c: char| {
            c.is_ascii()
                && (c.is_alphanumeric()
                    || PATTERN_WILDCARD.contains(c)
                    || JPCOMMON_SYMBOLS.contains(c))
        })
    }
    #[inline(always)]
    fn parse_ascii<E: ParseError<Self>>(self) -> IResult<Self, Self, E> {
        Self::parse_template(self, |c: char| c.is_ascii() && c != '\n')
    }

    fn parse_pattern_list<E: ParseError<Self>>(self) -> IResult<Self, Vec<Pattern>, E> {
        use nom::character::complete::char;
        let parse_elem = move |s| {
            let (rest, s) = Self::parse_pattern(s)?;
            let (_, sstr) = Self::parse_ascii_to_string(&s)?;
            Ok((rest, sstr))
        };
        cut(separated_list0(
            pair(char(','), Self::sp),
            cut(map(
                alt((delimited(char('\"'), parse_elem, char('\"')), parse_elem)),
                |p| Pattern::from_pattern_string(p).unwrap(),
            )),
        ))(self)
    }
}

impl ParseTarget for &str {
    fn parse_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        Ok(self)
    }

    #[inline(always)]
    fn parse_template<'a, F, E>(self, cond: F) -> IResult<Self, Self, E>
    where
        F: Fn(char) -> bool,
        E: ParseError<Self>,
    {
        take_while(cond)(self)
    }
    #[inline(always)]
    fn parse_template1<'a, F, E>(self, cond: F) -> IResult<Self, Self, E>
    where
        F: Fn(char) -> bool,
        E: ParseError<Self>,
    {
        take_while1(cond)(self)
    }

    fn parse_ascii_to_string<E: ParseError<Self>>(&self) -> IResult<Self, String, E> {
        Self::parse_ascii(self).map(|(rest, result)| (rest, result.to_string()))
    }
}

impl ParseTarget for &[u8] {
    fn parse_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self)
    }

    #[inline(always)]
    fn parse_template<'a, F, E>(self, cond: F) -> IResult<Self, Self, E>
    where
        F: Fn(char) -> bool,
        E: ParseError<Self>,
    {
        take_while(|c| cond(c as char))(self)
    }
    #[inline(always)]
    fn parse_template1<'a, F, E>(self, cond: F) -> IResult<Self, Self, E>
    where
        F: Fn(char) -> bool,
        E: ParseError<Self>,
    {
        take_while1(|c| cond(c as char))(self)
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

    use crate::model::stream::Pattern;

    use super::ParseTarget;

    #[test]
    fn ascii() {
        assert_eq!(
            "hogehoge\"\nfugafuga".parse_ascii::<VerboseError<&str>>(),
            Ok(("\nfugafuga", "hogehoge\""))
        );
    }

    #[test]
    fn pattern() {
        assert_eq!(
            "*=d/A:*".parse_pattern::<VerboseError<&str>>(),
            Ok(("", "*=d/A:*"))
        );
        assert_eq!(
            "*^i-*".parse_pattern::<VerboseError<&str>>(),
            Ok(("", "*^i-*"))
        );
    }

    #[test]
    fn pattern_list() {
        assert_eq!(
            "\"*-sil+*\",\"*-pau+*\"".parse_pattern_list::<VerboseError<&str>>(),
            Ok((
                "",
                vec![
                    Pattern::from_pattern_string("*-sil+*").unwrap(),
                    Pattern::from_pattern_string("*-pau+*").unwrap(),
                ]
            ))
        );
    }
}
