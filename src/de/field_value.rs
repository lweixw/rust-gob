use std::io::Cursor;

use bytes::Buf;
use serde;
use serde::de::Visitor;
use serde::de::value::Error;

use ::gob::Message;
use ::types::{TypeId, TypeDefs, WireType};

use super::struct_value::StructValueDeserializer;
use super::slice_value::SliceValueDeserializer;
use super::array_value::ArrayValueDeserializer;
use super::map_value::MapValueDeserializer;
use super::complex_value::ComplexValueDeserializer;

pub(crate) struct FieldValueDeserializer<'t, 'de> where 'de: 't {
    type_id: TypeId,
    defs: &'t TypeDefs,
    msg: &'t mut Message<Cursor<&'de [u8]>>
}

impl<'t, 'de> FieldValueDeserializer<'t, 'de> {
    pub fn new(type_id: TypeId, defs: &'t TypeDefs, msg: &'t mut Message<Cursor<&'de [u8]>>) -> FieldValueDeserializer<'t, 'de> {
        FieldValueDeserializer {
            type_id, defs, msg
        }
    }

    fn deserialize_byte_slice(&mut self) -> Result<&'de [u8], Error> {
        let len = self.msg.read_bytes_len()?;
        let pos = self.msg.get_ref().position() as usize;
        self.msg.get_mut().advance(len);
        let bytes = &self.msg.get_ref().get_ref()[pos..pos+len];
        Ok(bytes)
    }

    fn deserialize_str_slice(&mut self) -> Result<&'de str, Error> {
        let bytes = self.deserialize_byte_slice()?;
        ::std::str::from_utf8(bytes)
            .map_err(|err| serde::de::Error::custom(err))
    }
}

impl<'t, 'de> serde::Deserializer<'de> for FieldValueDeserializer<'t, 'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match self.type_id {
            TypeId::BOOL => visitor.visit_bool(self.msg.read_bool()?),
            TypeId::INT => visitor.visit_i64(self.msg.read_int()?),
            TypeId::UINT => visitor.visit_u64(self.msg.read_uint()?),
            TypeId::FLOAT => visitor.visit_f64(self.msg.read_float()?),
            TypeId::BYTES => visitor.visit_borrowed_bytes(self.deserialize_byte_slice()?),
            TypeId::STRING => visitor.visit_borrowed_str(self.deserialize_str_slice()?),
            TypeId::COMPLEX => {
                ComplexValueDeserializer::new(self.msg).deserialize_any(visitor)
            },
            _ => {
                if let Some(wire_type) = self.defs.lookup(self.type_id) {
                    match wire_type {
                        &WireType::Struct(ref struct_type) => {
                            let de = StructValueDeserializer::new(struct_type, self.defs, self.msg);
                            de.deserialize_any(visitor)
                        },
                        &WireType::Slice(ref slice_type) => {
                            let de = SliceValueDeserializer::new(slice_type, self.defs, self.msg);
                            de.deserialize_any(visitor)
                        },
                        &WireType::Array(ref array_type) => {
                            let de = ArrayValueDeserializer::new(array_type, self.defs, self.msg);
                            de.deserialize_any(visitor)
                        },
                        &WireType::Map(ref map_type) => {
                            let de = MapValueDeserializer::new(map_type, self.defs, self.msg);
                            de.deserialize_any(visitor)
                        }
                    }
                } else {
                    Err(serde::de::Error::custom(format!("unknown type id {:?}", self.type_id)))
                }
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}