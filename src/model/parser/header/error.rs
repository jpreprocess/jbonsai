use std::fmt::{self, Display};

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    Message(String),

    Eof,
    ExpectedBool,
    ExpectedInteger,
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
            DeserializeError::ExpectedBool => formatter.write_str("expected bool (0 or 1)"),
            DeserializeError::ExpectedInteger => formatter.write_str("expected integer value"),
            DeserializeError::ExpectedArrayComma => {
                formatter.write_str("expected comma as an array delimiter")
            }
            DeserializeError::ExpectedMapColon => {
                formatter.write_str("expected colon as map delimiter")
            }
            DeserializeError::ExpectedMapNewline => {
                formatter.write_str("expected newline as map delimiter")
            }
            DeserializeError::TrailingCharacters => {
                formatter.write_str("some characters were not consumed")
            }
        }
    }
}
