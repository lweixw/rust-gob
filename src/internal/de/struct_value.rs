use std::io::Cursor;

use serde;
use serde::de::{Deserializer, DeserializeSeed, IntoDeserializer, MapAccess, Visitor};
use serde::de::value::Error;

use ::internal::gob::Message;
use ::internal::types::{StructType, FieldType, Types};
use super::FieldValueDeserializer;

struct StructMapAccess<'t, 'de> where 'de: 't {
    def: &'t StructType,
    defs: &'t Types,
    field_no: i64,
    msg: &'t mut Message<Cursor<&'de [u8]>>
}

impl<'t, 'de> StructMapAccess<'t, 'de> {
    fn new(def: &'t StructType, defs: &'t Types, msg: &'t mut Message<Cursor<&'de [u8]>>) -> StructMapAccess<'t, 'de> {
        StructMapAccess {
            def, defs,
            field_no: -1,
            msg
        }
    }

    fn current_field(&self) -> Result<&'t FieldType, Error> {
        let field_no = self.field_no as usize;
        if field_no >= self.def.fields.len() {
            return Err(serde::de::Error::custom(format!("field number overflow ({}) on type {:?}", field_no, self.def)));
        }
        Ok(&self.def.fields[field_no])
    }
}

impl<'f, 'de> MapAccess<'de> for StructMapAccess<'f, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeSeed<'de>
    {
        let field_delta = self.msg.read_uint()?;

        if field_delta == 0 {
            return Ok(None);
        }

        self.field_no += field_delta as i64;
        let field = self.current_field()?;

        let de = <&str as IntoDeserializer>::into_deserializer(&field.name);
        let value = seed.deserialize(de).map_err(|err: Error| err)?;
        Ok(Some(value))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeSeed<'de>
    {
        let field = self.current_field()?;
        let de = FieldValueDeserializer::new(field.id, self.defs, &mut self.msg);
        seed.deserialize(de)
    }
}

pub(crate) struct StructValueDeserializer<'t, 'de> where 'de: 't {
    def: &'t StructType,
    defs: &'t Types,
    msg: &'t mut Message<Cursor<&'de [u8]>>
}

impl<'t, 'de> StructValueDeserializer<'t, 'de> {
    #[inline]
    pub(crate) fn new(def: &'t StructType, defs: &'t Types, msg: &'t mut Message<Cursor<&'de [u8]>>) -> StructValueDeserializer<'t, 'de> {
        StructValueDeserializer { def, defs, msg }
    }
}

impl<'t, 'de> Deserializer<'de> for StructValueDeserializer<'t, 'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_map(StructMapAccess::new(self.def, self.defs, self.msg))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}