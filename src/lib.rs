#![no_std]
//! Epee Encoding
//!
//! This library contains the Epee binary format found in Monero, unlike other
//! crates this crate does not use serde, this is not because serde is bad but
//! because that there is an [unfixable problem](https://github.com/monero-rs/monero-epee-bin-serde/issues/49)
//! with using serde to encode in the Epee format.
//!
//! TODO: usage examples

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use std_shims::io;
use std_shims::io::{ErrorKind, Read, Write};

mod marker;
mod serialize;
mod value;
pub mod varint;
#[cfg(feature = "derive")]
pub use epee_encoding_derive::EpeeObject;

use marker::{InnerMarker, Marker};
use serialize::{read_byte, read_bytes, read_string};
use value::EpeeValue;
use varint::*;

/// Header that needs to be at the beginning of every binary blob that follows
/// this binary serialization format.
const HEADER: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01\x01";
/// The maximum length a byte array (marked as a string) can be.
const MAX_STRING_LEN_POSSIBLE: u64 = 2000000000;

pub trait EpeeObjectBuilder<T>: Default + Sized {
    fn add_field<R: Read>(&mut self, name: &str, r: &mut R) -> io::Result<()>;

    fn finish(self) -> io::Result<T>;
}

pub trait EpeeObject: Sized {
    type Builder: EpeeObjectBuilder<Self>;

    fn write<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

fn read_header<R: Read>(r: &mut R) -> io::Result<()> {
    let mut buf = [0; 9];
    r.read_exact(&mut buf)?;
    if buf != HEADER {
        return Err(io::Error::new(
            ErrorKind::Other,
            "Data does not contain header",
        ));
    }
    Ok(())
}

fn write_header<W: Write>(w: &mut W) -> io::Result<()> {
    w.write_all(HEADER)
}

pub fn from_bytes<T: EpeeObject>(mut buf: &[u8]) -> io::Result<T> {
    read_head_object(&mut buf)
}

pub fn to_bytes<T: EpeeObject>(val: &T) -> io::Result<Vec<u8>> {
    let mut buf = Vec::<u8>::new();
    write_head_object(val, &mut buf)?;
    Ok(buf)
}

fn write_head_object<T: EpeeObject, W: Write>(val: &T, w: &mut W) -> io::Result<()> {
    write_header(w)?;
    val.write(w)
}

fn read_head_object<T: EpeeObject, R: Read>(r: &mut R) -> io::Result<T> {
    read_header(r)?;
    read_object(r)
}

pub fn read_field_name<R: Read>(r: &mut R) -> io::Result<String> {
    let len = read_byte(r)?;
    read_string(r, len.into())
}

pub fn write_field_name<W: Write>(val: &str, w: &mut W) -> io::Result<()> {
    w.write(&[val.len().try_into().unwrap()])?;
    w.write_all(val.as_bytes())
}

pub fn write_field<T: EpeeValue, W: Write>(val: &T, field_name: &str, w: &mut W) -> io::Result<()> {
    write_field_name(field_name, w)?;
    write_epee_value(val, w)
}

pub fn read_object<T: EpeeObject, R: Read>(r: &mut R) -> io::Result<T> {
    let mut object_builder = T::Builder::default();

    let number_o_field = read_varint(r)?;
    // TODO: Size check numb of fields?

    for _ in 0..number_o_field {
        let field_name = read_field_name(r)?;

        object_builder.add_field(&field_name, r)?;
    }
    object_builder.finish()
}

pub fn read_marker<R: Read>(r: &mut R) -> io::Result<Marker> {
    Marker::try_from(read_byte(r)?)
}

pub fn read_epee_value<T: EpeeValue, R: Read>(r: &mut R) -> io::Result<T> {
    if read_marker(r)? != T::MARKER {
        return Err(io::Error::new(
            ErrorKind::Other,
            "Marker does match expected Marker",
        ));
    }

    T::read(r)
}

pub fn write_epee_value<T: EpeeValue, W: Write>(val: &T, w: &mut W) -> io::Result<()> {
    w.write_all(&[T::MARKER.as_u8()])?;
    val.write(w)
}

/// A helper object builder that just skips every field.
#[derive(Default)]
struct SkipObjectBuilder;

impl EpeeObjectBuilder<SkipObject> for SkipObjectBuilder {
    fn add_field<R: Read>(&mut self, _name: &str, r: &mut R) -> io::Result<()> {
        skip_epee_value(r)
    }

    fn finish(self) -> io::Result<SkipObject> {
        Ok(SkipObject)
    }
}

/// A helper object that just skips every field.
struct SkipObject;

impl EpeeObject for SkipObject {
    type Builder = SkipObjectBuilder;

    fn write<W: Write>(&self, _w: &mut W) -> io::Result<()> {
        panic!("This is a helper function to use when de-serialising")
    }
}

pub fn skip_epee_value<R: Read>(r: &mut R) -> io::Result<()> {
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
                Vec::<u8>::read(r)?;
            }
            InnerMarker::Object => {
                read_object::<SkipObject, _>(r)?;
            }
        };
    }
    Ok(())
}
