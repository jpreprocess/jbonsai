use std::fmt::{self, Display};

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    Message(String),

    Eof,
    ExpectedBool,
    ExpectedInteger,
    ExpectedString,
    ExpectedArrayComma,
    ExpectedMapColon,
    ExpectedMapNewline,
    TrailingCharacters,
}

impl serde::de::Error for DeserializeError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializeError::Message(msg.to_string())
    }
}

impl Display for DeserializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeserializeError::Message(msg) => formatter.write_str(msg),
            DeserializeError::Eof => formatter.write_str("unexpected end of input"),
            _ => todo!(),
        }
    }
}
