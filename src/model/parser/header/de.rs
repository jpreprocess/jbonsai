use std::ops::{AddAssign, MulAssign};

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

use super::error::DeserializeError;

pub type Result<T> = std::result::Result<T, DeserializeError>;

pub struct Deserializer<'de> {
    input: &'de str,
    context: DeserializeContext,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input,
            context: DeserializeContext::default(),
        }
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
        Err(DeserializeError::TrailingCharacters)
    }
}

#[derive(Debug, Clone, Default)]
struct DeserializeContext {
    contexts: Vec<char>,
}

impl DeserializeContext {
    fn current(&self) -> Option<char> {
        self.contexts.last().copied()
    }
    fn test(&self, c: char) -> bool {
        self.contexts.contains(&c)
    }
    fn test_noncurrent(&self, c: char) -> bool {
        self.contexts.len() > 0 && self.contexts[..self.contexts.len() - 1].contains(&c)
    }
    fn enter(&mut self, expect: char) {
        self.contexts.push(expect)
    }
    fn exit(&mut self) -> Option<char> {
        self.contexts.pop()
    }
}

impl<'de> Deserializer<'de> {
    fn peek_char(&self) -> Result<char> {
        self.input.chars().next().ok_or(DeserializeError::Eof)
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        match self.next_char()? {
            '0' => Ok(false),
            '1' => Ok(true),
            _ => Err(DeserializeError::ExpectedBool),
        }
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_char()? {
            ch @ '0'..='9' => T::from(ch as u8 - b'0'),
            _ => {
                return Err(DeserializeError::ExpectedInteger);
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
                None => Err(DeserializeError::Eof),
            }
        } else {
            self.str_until_expect()
        }
    }

    fn str_until_expect(&mut self) -> Result<&'de str> {
        let len = self
            .input
            .find(|c| self.context.test(c))
            .unwrap_or(self.input.len());

        let s = &self.input[..len];
        self.input = &self.input[len..];
        Ok(s)
    }

    /// Consume string until chars other than current delimiter appears.
    /// - `Some(true)`: Current delimiter found (sequence continues)
    /// - `Some(false)`: Current delimiter was not found, but other delimiters were found or the string reached EOF (sequence ends)
    /// - `None`: No delimiter found (syntax error)
    fn consume_delimiters(&mut self) -> Option<bool> {
        let Some(current) = self.context.current() else {
            // Delimiter is empty
            return None;
        };

        let Some(pos) = self.input.find(|c| c != current) else {
            // Input was all delimiter (EOF)
            // When EOF, clear buffer
            self.input = "";
            return Some(false);
        };
        let (prefix, rest) = self.input.split_at(pos);
        if prefix.contains(|c| self.context.test_noncurrent(c)) {
            // Other delimiters came earlier than current one
            return Some(false);
        }
        if prefix.is_empty() {
            match rest.chars().next() {
                // The first char was delimiter (end of sequence)
                Some(c) if self.context.test_noncurrent(c) => return Some(false),
                // The first char was not delimiter
                _ => return None,
            }
        }

        self.input = rest;
        Some(true)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.str_until_expect()?)
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 f32 f64 char
        bytes byte_buf unit unit_struct
        enum
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
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

    fn deserialize_option<V>(self, visitor: V) -> std::prelude::v1::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_char() {
            Ok('\n') | Err(DeserializeError::Eof) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
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
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        self.de.context.enter(self.delim);
        match self.de.consume_delimiters() {
            Some(false) => {
                self.de.context.exit();
                return Ok(None);
            }
            // No need to clear context if deserializing stops here
            None if !self.first => return Err(DeserializeError::ExpectedArrayComma),
            _ => (),
        }
        self.first = false;

        let res = seed.deserialize(&mut *self.de);
        self.de.context.exit();
        res.map(Some)
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
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        self.de.context.enter('\n');
        match self.de.consume_delimiters() {
            Some(false) => {
                self.de.context.exit();
                return Ok(None);
            }
            // No need to clear context if deserializing stops here
            None if !self.first => return Err(DeserializeError::ExpectedMapNewline),
            _ => (),
        }
        self.first = false;

        self.de.context.enter(':');
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.de.next_char()? != ':' {
            // No need to clear context if deserializing stops here
            return Err(DeserializeError::ExpectedMapColon);
        }

        self.de.context.exit();
        let res = seed.deserialize(&mut *self.de);
        self.de.context.exit();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::Deserializer;

    #[test]
    fn consume_delimiters() {
        {
            let mut de = Deserializer::from_str("\nFULL");
            de.context.enter('\n');
            assert_eq!(de.consume_delimiters(), Some(true));
            assert_eq!(de.input, "FULL");
        }
        {
            let mut de = Deserializer::from_str(",\nFULL");
            de.context.enter('\n');
            de.context.enter(',');
            assert_eq!(de.consume_delimiters(), Some(true));
            assert_eq!(de.input, "\nFULL");
        }
        {
            let mut de = Deserializer::from_str("\nFULL");
            de.context.enter('\n');
            de.context.enter(',');
            assert_eq!(de.consume_delimiters(), Some(false));
            assert_eq!(de.input, "\nFULL");
        }
        {
            let mut de = Deserializer::from_str("\nFULL");
            de.context.enter(',');
            assert_eq!(de.consume_delimiters(), None);
            assert_eq!(de.input, "\nFULL");
        }
    }
}
