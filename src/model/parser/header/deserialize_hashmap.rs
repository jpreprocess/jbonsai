use std::collections::HashMap;

use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
    forward_to_deserialize_any,
};

use super::error::DeserializeError;

struct StrMapVisitor;
impl<'de> Visitor<'de> for StrMapVisitor {
    type Value = HashMap<&'de str, Vec<(&'de str, &'de str)>>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("StrMap")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut result = HashMap::new();

        while let Some((key, value)) = map.next_entry::<&str, &str>()? {
            if !key.ends_with(']') {
                continue;
            }
            let Some(start) = key.find('[') else {
                continue;
            };

            let key_sub = &key[start + 1..key.len() - 1];
            let key_main = &key[..start];

            result
                .entry(key_sub)
                .or_insert(Vec::new())
                .push((key_main, value));
        }

        Ok(result)
    }
}

struct MapDeserializer<'de> {
    inner: Vec<(&'de str, &'de str)>,
}
impl<'de> MapDeserializer<'de> {
    pub fn new(inner: Vec<(&'de str, &'de str)>) -> Self {
        MapDeserializer { inner }
    }
}
impl<'de> serde::de::Deserializer<'de> for &mut MapDeserializer<'de> {
    type Error = DeserializeError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(AlreadySeparated::new(self))
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum map struct identifier ignored_any
    }
}

struct AlreadySeparated<'a, 'de: 'a> {
    de: &'a MapDeserializer<'de>,
    index: usize,
}
impl<'a, 'de> AlreadySeparated<'a, 'de> {
    fn new(de: &'a MapDeserializer<'de>) -> Self {
        Self { de, index: 0 }
    }
}
impl<'de> MapAccess<'de> for AlreadySeparated<'_, 'de> {
    type Error = DeserializeError;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.index += 1;
        let Some((k, _)) = self.de.inner.get(self.index - 1) else {
            return Ok(None);
        };

        seed.deserialize(&mut super::de::Deserializer::from_str(k))
            .map(Some)
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let (_, v) = &self.de.inner[self.index - 1];

        seed.deserialize(&mut super::de::Deserializer::from_str(v))
    }
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<HashMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: 'de + Deserialize<'de> + std::fmt::Debug + Clone,
{
    deserializer
        .deserialize_map(StrMapVisitor)?
        .into_iter()
        .map(|(k, v)| {
            Ok((
                k.to_string(),
                T::deserialize(&mut MapDeserializer::new(v)).map_err(serde::de::Error::custom)?,
            ))
        })
        .collect()
}
