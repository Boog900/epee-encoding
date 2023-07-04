use alloc::vec::Vec;
/// This module contains a `sealed` [`EpeeValue`] trait and different impls for
/// the different possible base epee values.
use sealed::sealed;

use crate::io::*;
use crate::varint::*;
use crate::{EpeeObject, Error, InnerMarker, Marker, Result, MAX_STRING_LEN_POSSIBLE};

/// A trait for epee values, this trait is sealed as all possible epee values are
/// defined in the lib, to make an [`EpeeValue`] outside the lib you will need to
/// use the trait [`EpeeObject`].
#[sealed]
pub trait EpeeValue: Sized {
    const MARKER: Marker;

    fn read<R: Read>(r: &mut R) -> Result<Self>;

    fn write<W: Write>(&self, w: &mut W) -> Result<()>;
}

#[sealed]
impl<T: EpeeObject> EpeeValue for T {
    const MARKER: Marker = Marker::new(InnerMarker::Object);

    fn read<R: Read>(r: &mut R) -> Result<Self> {
        crate::read_object(r)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        self.write(w)
    }
}

#[sealed]
impl<T: EpeeObject> EpeeValue for Vec<T> {
    const MARKER: Marker = T::MARKER.into_seq();

    fn read<R: Read>(r: &mut R) -> Result<Self> {
        let len = read_varint(r)?;
        // TODO: check length
        let mut res = Vec::with_capacity(len.try_into().unwrap());
        for _ in 0..len {
            res.push(T::read(r)?);
        }
        Ok(res)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into().unwrap(), w)?;
        for item in self.iter() {
            item.write(w)?;
        }
        Ok(())
    }
}

macro_rules! epee_numb {
    ($numb:ty, $marker:ident) => {
        #[sealed]
        impl EpeeValue for $numb {
            const MARKER: Marker = Marker::new(InnerMarker::$marker);

            fn read<R: Read>(r: &mut R) -> Result<Self> {
                Ok(<$numb>::from_le_bytes(read_bytes(r)?))
            }

            fn write<W: Write>(&self, w: &mut W) -> Result<()> {
                w.write_all(&self.to_le_bytes())
            }
        }
    };
}

epee_numb!(i64, I64);
epee_numb!(i32, I32);
epee_numb!(i16, I16);
epee_numb!(i8, I8);
epee_numb!(u8, U8);
epee_numb!(u16, U16);
epee_numb!(u32, U32);
epee_numb!(u64, U64);
epee_numb!(f64, F64);

#[sealed]
impl EpeeValue for bool {
    const MARKER: Marker = Marker::new(InnerMarker::Bool);

    fn read<R: Read>(r: &mut R) -> Result<Self> {
        Ok(read_byte(r)? != 0)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&[if *self { 1 } else { 0 }])
    }
}

#[sealed]
impl EpeeValue for Vec<u8> {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<R: Read>(r: &mut R) -> Result<Self> {
        let len = read_varint(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("Byte array exceeded max length"));
        }

        read_var_bytes(r, len.try_into().unwrap())
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into().unwrap(), w)?;
        w.write_all(self)
    }
}

#[sealed]
impl<const N: usize> EpeeValue for [u8; N] {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<R: Read>(r: &mut R) -> Result<Self> {
        let len = read_varint(r)?;
        if len != N.try_into().unwrap() {
            return Err(Error::Format("Byte array has incorrect length"));
        }

        read_bytes(r)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into().unwrap(), w)?;
        w.write_all(self)
    }
}

macro_rules! epee_seq {
    ($val:ty) => {
        #[sealed]
        impl EpeeValue for Vec<$val> {
            const MARKER: Marker = <$val>::MARKER.into_seq();

            fn read<R: Read>(r: &mut R) -> Result<Self> {
                let len = read_varint(r)?;
                //TODO: SIZE CHECK
                let mut res = Vec::with_capacity(len.try_into().unwrap());
                for _ in 0..len {
                    res.push(<$val>::read(r)?);
                }
                Ok(res)
            }

            fn write<W: Write>(&self, w: &mut W) -> Result<()> {
                write_varint(self.len().try_into().unwrap(), w)?;
                for item in self.iter() {
                    item.write(w)?;
                }
                Ok(())
            }
        }
    };
}

epee_seq!(i64);
epee_seq!(i32);
epee_seq!(i16);
epee_seq!(i8);
epee_seq!(u64);
epee_seq!(u32);
epee_seq!(u16);
epee_seq!(f64);
epee_seq!(bool);

#[test]
fn t() {}
