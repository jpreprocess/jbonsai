use std::collections::HashMap;
use std::fmt::{self, Display};
use std::ops::{AddAssign, MulAssign};

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

pub type Result<T> = std::result::Result<T, Error>;

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

impl de::Error for Error {
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

pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_char(&self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_char()? {
            ch @ '0'..='9' => T::from(ch as u8 - b'0'),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        loop {
            match self.input.chars().next() {
                Some(ch @ '0'..='9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch as u8 - b'0');
                }
                _ => {
                    return Ok(int);
                }
            }
        }
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        if matches!(self.peek_char(), Ok('"')) {
            self.next_char()?;
            match self.input.find('"') {
                Some(len) => {
                    let s = &self.input[..len];
                    self.input = &self.input[len + 1..];
                    Ok(s)
                }
                None => Err(Error::Eof),
            }
        } else {
            let len = self
                .input
                .find([':', ',', '\n'])
                .unwrap_or(self.input.len());
            let s = &self.input[..len];
            self.input = &self.input[len..];
            Ok(s)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 f32 f64 char
        bytes byte_buf option unit unit_struct
        enum
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(List::new(self, ','))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(List::new(self, '-'))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(NewlineSeparated::new(self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct List<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
    delim: char,
}

impl<'a, 'de> List<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, delim: char) -> Self {
        List {
            de,
            first: true,
            delim,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for List<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if !self.first {
            match self.de.peek_char() {
                Ok(c) if c == self.delim => (),
                Ok('\n') => return Ok(None),
                Err(Error::Eof) => return Ok(None),
                _ => return Err(Error::ExpectedArrayComma),
            }
            self.de.next_char()?;
        }
        self.first = false;

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct NewlineSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> NewlineSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        NewlineSeparated { de, first: true }
    }
}

impl<'de, 'a> MapAccess<'de> for NewlineSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        dbg!(&self.de.input);
        let mut newline = false;
        loop {
            let char = self.de.peek_char();
            match char {
                Ok('\n') => newline = true,
                Err(Error::Eof) => return Ok(None),
                _ => break,
            }
            self.de.next_char()?;
        }
        if !self.first && !newline {
            return Err(Error::ExpectedMapNewline);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.de.next_char()? != ':' {
            return Err(Error::ExpectedMapColon);
        }
        seed.deserialize(&mut *self.de)
    }
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Test {
    fullcontext_version: String,
    gv_off_context: Vec<String>,
    sampling_frequency: usize,
    stream_win: Vec<(usize, usize)>,
    #[serde(flatten, with = "with")]
    test: HashMap<String, TestInner>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct TestInner {
    stream_pdf: (usize, usize),
}

#[test]
fn test_struct() {
    let j = r#"
FULLCONTEXT_VERSION:1.0
GV_OFF_CONTEXT:"*-sil+*","*-pau+*"
SAMPLING_FREQUENCY:48000
STREAM_WIN:40880-40885,40886-40900
STREAM_PDF[LF0]:788578-848853
"#;
    let expected = Test {
        fullcontext_version: "1.0".to_string(),
        gv_off_context: vec!["*-sil+*".to_owned(), "*-pau+*".to_owned()],
        sampling_frequency: 48000,
        stream_win: vec![(40880, 40885), (40886, 40900)],
        test: HashMap::new(),
    };
    assert_eq!(expected, from_str(j).unwrap());
}

mod with {
    use std::collections::HashMap;

    use serde::{de::Visitor, Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<HashMap<String, T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + std::fmt::Debug,
    {
        struct StrMapVisitor;
        impl<'de> Visitor<'de> for StrMapVisitor {
            type Value = Vec<(String, String)>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("StrMap")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut result = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    result.push(entry);
                }
                Ok(result)
            }
        }

        let s = deserializer.deserialize_map(StrMapVisitor)?;
        dbg!(s);
        todo!()
    }
}
