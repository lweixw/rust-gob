use std::io::Cursor;

use serde::{self, Deserialize};
use serde::de::Visitor;
use serde::de::value::Error;

use ::gob::Message;
use ::types::{TypeId, TypeDefs, WireType};

mod value_deserializer;
mod struct_deserializer;
mod slice_deserializer;

use self::value_deserializer::ValueDeserializer;
use self::struct_deserializer::StructDeserializer;

impl From<::gob::Error> for Error {
    fn from(err: ::gob::Error) -> Error {
        serde::de::Error::custom(format!("{:?}", err))
    }
}

pub struct Deserializer<'de> {
    msg: Message<Cursor<&'de [u8]>>
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Deserializer<'de> {
        Deserializer {
            msg: Message::new(Cursor::new(input))
        }
    }
}

impl<'de> serde::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let mut defs = TypeDefs::new();

        loop {
            let _len = self.msg.read_bytes_len()?;
            let type_id = self.msg.read_int()?;

            if type_id >= 0 {
                if let Some(&WireType::Struct(ref struct_type)) = defs.lookup(TypeId(type_id)) {
                    let de = StructDeserializer::new(struct_type, &defs, &mut self.msg);
                    return serde::de::Deserializer::deserialize_any(de, visitor);
                }

                if self.msg.read_uint()? != 0 {
                    return Err(serde::de::Error::custom(format!("neither a singleton nor a struct value")));
                }

                let de = ValueDeserializer::new(TypeId(type_id), &defs, &mut self.msg);
                return serde::de::Deserializer::deserialize_any(de, visitor);
            }

            let wire_type = {
                let de = ValueDeserializer::new(TypeId::WIRE_TYPE, &defs, &mut self.msg);
                WireType::deserialize(de)
            }?;

            if -type_id != wire_type.common().id.0 {
                return Err(serde::de::Error::custom(format!("type id mismatch")));
            }

            defs.insert(wire_type);
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
