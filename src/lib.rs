#![deny(warnings)]

extern crate bytes;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_schema;
extern crate slice_deque;
extern crate smallvec;

mod internal;
mod schema;

pub mod error;

pub mod de;
pub mod ser;

pub use error::Error;

pub use de::{Deserializer, StreamDeserializer};
pub use ser::StreamSerializer;
