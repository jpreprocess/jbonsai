use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    Message(String),

    Eof,
    ExpectedInteger,
    ExpectedString,
    ExpectedArrayComma,
    ExpectedMapColon,
    ExpectedMapNewline,
    TrailingCharacters,
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            _ => todo!(),
        }
    }
}

impl std::error::Error for Error {}
