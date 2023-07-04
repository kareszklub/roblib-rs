use crate::cmd::SEPARATOR;

use super::error::{Error, Result};
use serde::{
    de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess},
    Deserialize,
};

pub struct Deserializer<'de, I: Iterator<Item = &'de str>> {
    iter: I,
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer {
        iter: s.split(SEPARATOR),
    };

    let t = T::deserialize(&mut deserializer)?;
    if deserializer.iter.next().is_none() {
        Ok(t)
    } else {
        Err(Error::Trailing)
    }
}

impl<'de, I: Iterator<Item = &'de str>> Deserializer<'de, I> {
    fn next(&mut self) -> Result<I::Item> {
        match self.iter.next() {
            Some("") => Err(Error::Empty),
            Some(s) => Ok(s),
            None => Err(Error::MissingArgument),
        }
    }
}

impl<'de, 'a, I: Iterator<Item = &'de str>> de::Deserializer<'de> for &'a mut Deserializer<'de, I> {
    type Error = Error;

    fn deserialize_any<V>(self, _: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::DeserializeAny)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(match self.next()? {
            "0" => false,
            "1" => true,
            _ => return Err(Error::Parse("bool")),
        })
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.next()?.parse().map_err(|_| Error::Parse("i8"))?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.next()?.parse().map_err(|_| Error::Parse("i16"))?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.next()?.parse().map_err(|_| Error::Parse("i32"))?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.next()?.parse().map_err(|_| Error::Parse("i64"))?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.next()?.parse().map_err(|_| Error::Parse("u8"))?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.next()?.parse().map_err(|_| Error::Parse("u16"))?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.next()?.parse().map_err(|_| Error::Parse("u32"))?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.next()?.parse().map_err(|_| Error::Parse("u64"))?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.next()?.parse().map_err(|_| Error::Parse("f32"))?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.next()?.parse().map_err(|_| Error::Parse("f64"))?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let mut cs = self.next()?.chars();
        let c = cs.next().ok_or(Error::Empty)?;
        if cs.next().is_some() {
            return Err(Error::Trailing)?;
        }
        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.next()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.next()?.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.next()? {
            "0" => visitor.visit_none(),
            "1" => visitor.visit_some(self),
            _ => Err(Error::Parse("Option")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len: u32 = self.next()?.parse().map_err(|_| Error::Parse("u32"))?;
        visitor.visit_seq(Seq {
            de: self,
            left: len,
        })
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len: u32 = self.next()?.parse().map_err(|_| Error::Parse("u32"))?;
        visitor.visit_map(Seq {
            de: self,
            left: len,
        })
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(Seq {
            de: self,
            left: fields.len() as u32,
        })
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(Enum { de: self })
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::DeserializeAny)
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::DeserializeAny)
    }
}

struct Seq<'de: 'a, 'a, I: Iterator<Item = &'de str>> {
    de: &'a mut Deserializer<'de, I>,
    left: u32,
}

impl<'de, 'a, I: Iterator<Item = &'de str>> SeqAccess<'de> for Seq<'de, 'a, I> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.left > 0 {
            self.left -= 1;

            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left as usize)
    }
}

impl<'de, 'a, I: Iterator<Item = &'de str>> MapAccess<'de> for Seq<'de, 'a, I> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.left > 0 {
            self.left -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left as usize)
    }
}

struct Enum<'de: 'a, 'a, I: Iterator<Item = &'de str>> {
    de: &'a mut Deserializer<'de, I>,
}

impl<'de, 'a, I: Iterator<Item = &'de str>> EnumAccess<'de> for Enum<'de, 'a, I> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> std::result::Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let var: u32 = self.de.next()?.parse().map_err(|_| Error::Parse("u32"))?;
        let des: de::value::U32Deserializer<Error> = de::value::U32Deserializer::new(var);
        let v = seed.deserialize(des)?;
        Ok((v, self))
    }
}

impl<'de, 'a, I: Iterator<Item = &'de str>> VariantAccess<'de> for Enum<'de, 'a, I> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        serde::Deserializer::deserialize_tuple_struct(self.de, "Enum", len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        serde::Deserializer::deserialize_struct(self.de, "Enum", fields, visitor)
    }
}
