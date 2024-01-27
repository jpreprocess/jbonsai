use nom::{error::ParseError, IResult};

use crate::model::parser::ParseTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderEntry<S> {
    key_main: S,
    key_sub: Option<S>,
    value: S,
}

impl<S: ParseTarget> HeaderEntry<S>
where
    <S as nom::InputIter>::Item: nom::AsChar,
    <S as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    pub fn new(((key_main, key_sub), value): ((S, Option<S>), S)) -> Self {
        Self {
            key_main,
            key_sub,
            value,
        }
    }
    pub fn parse_to_string<E: ParseError<S>>(&self) -> IResult<(), HeaderEntry<String>, E> {
        let (_, key_main) = self.key_main.parse_ascii_to_string()?;
        let key_sub = self
            .key_sub
            .as_ref()
            .map(|s| s.parse_ascii_to_string().map(|s| s.1))
            .transpose()?;
        let (_, value) = self.value.parse_ascii_to_string()?;
        Ok((
            (),
            HeaderEntry {
                key_main,
                key_sub,
                value,
            },
        ))
    }
    pub fn into_key_main(self) -> S {
        self.key_main
    }
    pub fn into_key_sub_or_main(self) -> S {
        self.key_sub.unwrap_or(self.key_main)
    }
    pub fn into_value(self) -> S {
        self.value
    }
}

impl HeaderEntry<String> {
    pub fn key_main(&self) -> &str {
        &self.key_main
    }
    pub fn key_sub(&self) -> Option<&str> {
        self.key_sub.as_deref()
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}
