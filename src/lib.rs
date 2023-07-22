#![cfg_attr(not(feature = "std"), no_std)]
//! Epee Encoding
//!
//! This library contains the Epee binary format found in Monero, unlike other
//! crates this crate does not use serde.
//!
//! example without derive:
//! ```rust
//! use epee_encoding::{EpeeObject, EpeeObjectBuilder, read_epee_value, write_field, to_bytes, from_bytes};
//! use epee_encoding::io::{Read, Write};
//!
//! pub struct Test {
//!     val: u64
//! }
//!
//! #[derive(Default)]
//! pub struct __TestEpeeBuilder {
//!     val: Option<u64>,
//! }
//!
//! impl EpeeObjectBuilder<Test> for __TestEpeeBuilder {
//!     fn add_field<R: Read>(&mut self, name: &str, r: &mut R) -> epee_encoding::error::Result<bool> {
//!         match name {
//!             "val" => {self.val = Some(read_epee_value(r)?);}
//!             _ => return Ok(false),
//!         }
//!         Ok(true)
//!     }
//!
//!     fn finish(self) -> epee_encoding::error::Result<Test> {
//!         Ok(
//!             Test {
//!                 val: self.val.ok_or_else(|| epee_encoding::error::Error::Format("Required field was not found!"))?
//!             }
//!         )
//!     }
//! }
//!
//! impl EpeeObject for Test {
//!     type Builder = __TestEpeeBuilder;
//!
//!     fn number_of_fields(&self) -> u64 {
//!         1
//!     }
//!
//!     fn write_fields<W: Write>(&self, w: &mut W) -> epee_encoding::error::Result<()> {
//!        // write the fields
//!        write_field(&self.val, "val", w)
//!    }
//! }
//!
//!
//! let data = [1, 17, 1, 1, 1, 1, 2, 1, 1, 4, 3, 118, 97, 108, 5, 4, 0, 0, 0, 0, 0, 0, 0]; // the data to decode;
//! let val: Test = from_bytes(&data).unwrap();
//! let data = to_bytes(&val).unwrap();
//!
//!
//! ```
//!
//! example with derive:
//! ```ignore
//! use epee_encoding::{EpeeObject, from_bytes, to_bytes};
//!
//! #[derive(EpeeObject)]
//! struct Test2 {
//!     val: u64
//! }
//!
//!
//! let data = [1, 17, 1, 1, 1, 1, 2, 1, 1, 4, 3, 118, 97, 108, 5, 4, 0, 0, 0, 0, 0, 0, 0]; // the data to decode;
//! let val: Test2 = from_bytes(&data).unwrap();
//! let data = to_bytes(&val).unwrap();
//!
//! ```

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

pub mod error;
pub mod io;
pub mod marker;
mod value;
mod varint;

#[cfg(feature = "derive")]
pub use epee_encoding_derive::EpeeObject;

pub use error::*;
use io::*;
pub use marker::{InnerMarker, Marker};
pub use value::EpeeValue;
use varint::*;

/// Header that needs to be at the beginning of every binary blob that follows
/// this binary serialization format.
const HEADER: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01\x01";
/// The maximum length a byte array (marked as a string) can be.
const MAX_STRING_LEN_POSSIBLE: u64 = 2000000000;
/// The maximum depth of skipped objects.
const MAX_DEPTH_OF_SKIPPED_OBJECTS: u8 = 20;

/// A trait for an object that can build a type `T` from the epee format.
pub trait EpeeObjectBuilder<T>: Default + Sized {
    /// Called when a field names has been read no other bytes following the field
    /// name will have been read.
    ///
    /// Returns a bool if true then the field has been read otherwise the field is not
    /// needed and has not been read.
    fn add_field<R: Read>(&mut self, name: &str, r: &mut R) -> Result<bool>;

    /// Called when the number of fields has been read.
    fn finish(self) -> Result<T>;
}

/// A trait for an object that can be turned into epee bytes.
pub trait EpeeObject: Sized {
    type Builder: EpeeObjectBuilder<Self>;

    /// Returns the number of fields to be encoded.
    fn number_of_fields(&self) -> u64;

    /// write the objects fields into the writer.
    fn write_fields<W: Write>(&self, w: &mut W) -> Result<()>;
}

/// Read the object `T` from a byte array.
pub fn from_bytes<T: EpeeObject>(mut buf: &[u8]) -> Result<T> {
    read_head_object(&mut buf)
}

/// Turn the object into epee bytes.
pub fn to_bytes<T: EpeeObject>(val: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::<u8>::new();
    write_head_object(val, &mut buf)?;
    Ok(buf)
}

fn read_header<R: Read>(r: &mut R) -> Result<()> {
    let mut buf = [0; 9];
    r.read_exact(&mut buf)?;
    if buf != HEADER {
        return Err(Error::Format("Data does not contain header"));
    }
    Ok(())
}

fn write_header<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(HEADER)
}

fn write_head_object<T: EpeeObject, W: Write>(val: &T, w: &mut W) -> Result<()> {
    write_header(w)?;
    val.write(w)
}

fn read_head_object<T: EpeeObject, R: Read>(r: &mut R) -> Result<T> {
    read_header(r)?;
    let mut skipped_objects = 0;
    read_object(r, &mut skipped_objects)
}

fn read_field_name<R: Read>(r: &mut R) -> Result<String> {
    let len = read_byte(r)?;
    read_string(r, len.into())
}

fn write_field_name<W: Write>(val: &str, w: &mut W) -> Result<()> {
    w.write(&[val.len().try_into()?])?;
    w.write_all(val.as_bytes())
}

/// Write an epee field.
pub fn write_field<T: EpeeValue, W: Write>(val: &T, field_name: &str, w: &mut W) -> Result<()> {
    if val.should_write() {
        write_field_name(field_name, w)?;
        write_epee_value(val, w)?;
    }
    Ok(())
}

fn read_object<T: EpeeObject, R: Read>(r: &mut R, skipped_objects: &mut u8) -> Result<T> {
    let mut object_builder = T::Builder::default();

    let number_o_field = read_varint(r)?;
    // TODO: Size check numb of fields?

    for _ in 0..number_o_field {
        let field_name = read_field_name(r)?;

        if !object_builder.add_field(&field_name, r)? {
            skip_epee_value(r, skipped_objects)?;
        }
    }
    object_builder.finish()
}

/// Read a marker from the [`Read`], this function should only be used for
/// custom serialisation based on the marker otherwise just use [`read_epee_value`].
pub fn read_marker<R: Read>(r: &mut R) -> Result<Marker> {
    Marker::try_from(read_byte(r)?)
}

/// Read an epee value from the stream, an epee value is the part after the key
/// including the marker.
pub fn read_epee_value<T: EpeeValue, R: Read>(r: &mut R) -> Result<T> {
    let marker = read_marker(r)?;
    T::read(r, &marker)
}

/// Write an epee value to the stream, an epee value is the part after the key
/// including the marker.
fn write_epee_value<T: EpeeValue, W: Write>(val: &T, w: &mut W) -> Result<()> {
    w.write_all(&[T::MARKER.as_u8()])?;
    val.write(w)
}

/// A helper object builder that just skips every field.
#[derive(Default)]
struct SkipObjectBuilder;

impl EpeeObjectBuilder<SkipObject> for SkipObjectBuilder {
    fn add_field<R: Read>(&mut self, _name: &str, _r: &mut R) -> Result<bool> {
        Ok(false)
    }

    fn finish(self) -> Result<SkipObject> {
        Ok(SkipObject)
    }
}

/// A helper object that just skips every field.
struct SkipObject;

impl EpeeObject for SkipObject {
    type Builder = SkipObjectBuilder;

    fn number_of_fields(&self) -> u64 {
        panic!("This is a helper function to use when de-serialising")
    }

    fn write_fields<W: Write>(&self, _w: &mut W) -> Result<()> {
        panic!("This is a helper function to use when de-serialising")
    }
}

/// Skip an epee value, should be used when you do not need the value
/// stored at a key.
fn skip_epee_value<R: Read>(r: &mut R, skipped_objects: &mut u8) -> Result<()> {
    let marker = read_marker(r)?;
    let mut len = 1;
    if marker.is_seq {
        len = read_varint(r)?;
    }
    for _ in 0..len {
        match marker.inner_marker {
            InnerMarker::I64 | InnerMarker::U64 | InnerMarker::F64 => {
                read_bytes::<_, 8>(r)?;
            }
            InnerMarker::I32 | InnerMarker::U32 => {
                read_bytes::<_, 4>(r)?;
            }
            InnerMarker::I16 | InnerMarker::U16 => {
                read_bytes::<_, 2>(r)?;
            }
            InnerMarker::I8 | InnerMarker::U8 | InnerMarker::Bool => {
                read_bytes::<_, 1>(r)?;
            }
            InnerMarker::String => {
                Vec::<u8>::read(r, &marker)?;
            }
            InnerMarker::Object => {
                *skipped_objects += 1;
                if *skipped_objects > MAX_DEPTH_OF_SKIPPED_OBJECTS {
                    return Err(Error::Format("Depth of skipped objects exceeded maximum"));
                }
                read_object::<SkipObject, _>(r, skipped_objects)?;
                *skipped_objects -= 1;
            }
        };
    }
    Ok(())
}
