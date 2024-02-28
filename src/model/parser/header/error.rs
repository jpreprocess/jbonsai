use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error("{0}")]
    Message(String),

    #[error("unexpected end of input")]
    Eof,
    #[error("expected bool (0 or 1)")]
    ExpectedBool,
    #[error("expected integer value")]
    ExpectedInteger,
    #[error("expected comma as an array delimiter")]
    ExpectedArrayComma,
    #[error("expected colon as map delimiter")]
    ExpectedMapColon,
    #[error("expected newline as map delimiter")]
    ExpectedMapNewline,
    #[error("some characters were not consumed")]
    TrailingCharacters,
}

impl serde::de::Error for DeserializeError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializeError::Message(msg.to_string())
    }
}
