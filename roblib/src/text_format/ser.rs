use crate::cmd::SEPARATOR;

use super::error::{Error, Result};
use serde::{
    ser::{self, SerializeSeq},
    Serialize,
};
use std::fmt::{self, Write};

pub struct Serializer<W: fmt::Write> {
    first: bool,
    writer: W,
}
impl<W: fmt::Write> fmt::Write for Serializer<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.first {
            self.writer.write_fmt(format_args!("{}", SEPARATOR))?;
        }
        self.writer.write_str(s)
    }
}

impl<W: fmt::Write> ser::Serializer for &mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write_char(if v { '1' } else { '0' })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        Ok(write!(self, "{v}")?)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        Ok(self.write_char(v)?)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        Ok(self.write_str(v)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for b in v {
            seq.serialize_element(b)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        Ok(self.write_char('0')?)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_char('1')?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
    ) -> Result<()> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        write!(self, "{variant_index}")?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            write!(self, "{}", len as u32)?;
            Ok(self)
        } else {
            Err(Error::UnsizedSeq)
        }
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        write!(self, "{variant_index}")?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if let Some(len) = len {
            write!(self, "{len}")?;
            Ok(self)
        } else {
            Err(Error::UnsizedMap)
        }
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        write!(self, "{variant_index}")?;
        Ok(self)
    }
}

impl<'a, W: fmt::Write> ser::SerializeSeq for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: fmt::Write> ser::SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: fmt::Write> ser::SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: fmt::Write> ser::SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: fmt::Write> ser::SerializeMap for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: fmt::Write> ser::SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: fmt::Write> ser::SerializeStructVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
