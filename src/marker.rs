/// This module contains a [`Marker`] which is appended before each value to tell you the type.
use std_shims::io;
use std_shims::io::ErrorKind;

/// The inner marker just telling you the type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum InnerMarker {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    F64,
    String,
    Bool,
    Object,
}

/// A marker appended before Epee values which tell you the type of the field and if
/// its a sequence.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Marker {
    pub(crate) inner_marker: InnerMarker,
    pub(crate) is_seq: bool,
}

impl Marker {
    pub(crate) const fn new(inner_marker: InnerMarker) -> Self {
        Marker {
            inner_marker,
            is_seq: false,
        }
    }
    pub const fn into_seq(self) -> Self {
        if self.is_seq {
            panic!("Sequence of sequence not allowed!");
        }
        if matches!(self.inner_marker, InnerMarker::U8) {
            return Marker {
                inner_marker: InnerMarker::String,
                is_seq: false,
            };
        }

        Marker {
            inner_marker: self.inner_marker,
            is_seq: true,
        }
    }

    pub const fn as_u8(&self) -> u8 {
        let marker_val = match self.inner_marker {
            InnerMarker::I64 => 1,
            InnerMarker::I32 => 2,
            InnerMarker::I16 => 3,
            InnerMarker::I8 => 4,
            InnerMarker::U64 => 5,
            InnerMarker::U32 => 6,
            InnerMarker::U16 => 7,
            InnerMarker::U8 => 8,
            InnerMarker::F64 => 9,
            InnerMarker::String => 10,
            InnerMarker::Bool => 11,
            InnerMarker::Object => 12,
        };

        if self.is_seq {
            marker_val | 0x80
        } else {
            marker_val
        }
    }
}

impl TryFrom<u8> for Marker {
    type Error = io::Error;

    fn try_from(mut value: u8) -> Result<Self, Self::Error> {
        let is_seq = value & 0x80 > 0;

        if is_seq {
            value ^= 0x80;
        }

        let inner_marker = match value {
            1 => InnerMarker::I64,
            2 => InnerMarker::I32,
            3 => InnerMarker::I16,
            4 => InnerMarker::I8,
            5 => InnerMarker::U64,
            6 => InnerMarker::U32,
            7 => InnerMarker::U16,
            8 => InnerMarker::U8,
            9 => InnerMarker::F64,
            10 => InnerMarker::String,
            11 => InnerMarker::Bool,
            12 => InnerMarker::Object,
            _ => return Err(io::Error::new(ErrorKind::Other, "Unknown value Marker")),
        };

        Ok(Marker {
            inner_marker,
            is_seq,
        })
    }
}
