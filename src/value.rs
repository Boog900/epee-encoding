/// This module contains a `sealed` [`EpeeValue`] trait and different impls for
/// the different possible base epee values.
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

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

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self>;

    fn should_write(&self) -> bool {
        true
    }

    /// This is different than default field values and instead is the default
    /// value of a whole type.
    ///
    /// For example a `Vec` has a default value of a zero length vec as when a
    /// sequence has no entries it is not encoded.
    fn epee_default_value() -> Option<Self> {
        None
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()>;
}

#[sealed]
impl<T: EpeeObject> EpeeValue for T {
    const MARKER: Marker = Marker::new(InnerMarker::Object);

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let mut skipped_objects = 0;
        crate::read_object(r, &mut skipped_objects)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.number_of_fields(), w)?;
        self.write_fields(w)
    }
}

#[sealed]
impl<T: EpeeObject> EpeeValue for Vec<T> {
    const MARKER: Marker = T::MARKER.into_seq();

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if !marker.is_seq {
            return Err(Error::Format(
                "Marker is not sequence when a sequence was expected",
            ));
        }
        let len = read_varint(r)?;

        let individual_marker = Marker::new(marker.inner_marker.clone());

        let mut res = Vec::with_capacity(len.try_into()?);
        for _ in 0..len {
            res.push(T::read(r, &individual_marker)?);
        }
        Ok(res)
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn epee_default_value() -> Option<Self> {
        Some(Vec::new())
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into()?, w)?;
        for item in self.iter() {
            item.write(w)?;
        }
        Ok(())
    }
}

#[sealed]
impl<T: EpeeObject + Debug, const N: usize> EpeeValue for [T; N] {
    const MARKER: Marker = <T>::MARKER.into_seq();

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        let vec = Vec::<T>::read(r, marker)?;

        if vec.len() != N {
            return Err(Error::Format("Array has incorrect length"));
        }

        Ok(vec.try_into().unwrap())
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into()?, w)?;
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

            fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
                if marker != &Self::MARKER {
                    return Err(Error::Format("Marker does not match expected Marker"));
                }

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

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        Ok(read_byte(r)? != 0)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&[if *self { 1 } else { 0 }])
    }
}

#[sealed]
impl EpeeValue for Vec<u8> {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("Byte array exceeded max length"));
        }

        read_var_bytes(r, len.try_into()?)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into()?, w)?;
        w.write_all(self)
    }
}

#[sealed]
impl EpeeValue for String {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("String exceeded max length"));
        }

        read_string(r, len.try_into()?)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into()?, w)?;
        w.write_all(self.as_bytes())
    }
}

#[sealed]
impl<const N: usize> EpeeValue for [u8; N] {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint(r)?;
        if len != N.try_into()? {
            return Err(Error::Format("Byte array has incorrect length"));
        }

        read_bytes(r)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into()?, w)?;
        w.write_all(self)
    }
}

#[sealed]
impl<const N: usize> EpeeValue for Vec<[u8; N]> {
    const MARKER: Marker = <[u8; N]>::MARKER.into_seq();

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        if !marker.is_seq {
            return Err(Error::Format(
                "Marker is not sequence when a sequence was expected",
            ));
        }

        let len = read_varint(r)?;

        let individual_marker = Marker::new(marker.inner_marker.clone());

        let mut res = Vec::with_capacity(len.try_into()?);
        for _ in 0..len {
            res.push(<[u8; N]>::read(r, &individual_marker)?);
        }
        Ok(res)
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn epee_default_value() -> Option<Self> {
        Some(Vec::new())
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_varint(self.len().try_into()?, w)?;
        for item in self.iter() {
            item.write(w)?;
        }
        Ok(())
    }
}

macro_rules! epee_seq {
    ($val:ty) => {
        #[sealed]
        impl EpeeValue for Vec<$val> {
            const MARKER: Marker = <$val>::MARKER.into_seq();

            fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
                if !marker.is_seq {
                    return Err(Error::Format(
                        "Marker is not sequence when a sequence was expected",
                    ));
                }

                let len = read_varint(r)?;

                let individual_marker = Marker::new(marker.inner_marker.clone());

                let mut res = Vec::with_capacity(len.try_into()?);
                for _ in 0..len {
                    res.push(<$val>::read(r, &individual_marker)?);
                }
                Ok(res)
            }

            fn should_write(&self) -> bool {
                !self.is_empty()
            }

            fn epee_default_value() -> Option<Self> {
                Some(Vec::new())
            }

            fn write<W: Write>(&self, w: &mut W) -> Result<()> {
                write_varint(self.len().try_into()?, w)?;
                for item in self.iter() {
                    item.write(w)?;
                }
                Ok(())
            }
        }

        #[sealed]
        impl<const N: usize> EpeeValue for [$val; N] {
            const MARKER: Marker = <$val>::MARKER.into_seq();

            fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
                let vec = Vec::<$val>::read(r, marker)?;

                if vec.len() != N {
                    return Err(Error::Format("Array has incorrect length"));
                }

                Ok(vec.try_into().unwrap())
            }

            fn write<W: Write>(&self, w: &mut W) -> Result<()> {
                write_varint(self.len().try_into()?, w)?;
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
epee_seq!(Vec<u8>);
epee_seq!(String);

#[sealed]
impl<T: EpeeValue> EpeeValue for Option<T> {
    const MARKER: Marker = T::MARKER;

    fn read<R: Read>(r: &mut R, marker: &Marker) -> Result<Self> {
        Ok(Some(T::read(r, marker)?))
    }

    fn should_write(&self) -> bool {
        match self {
            Some(t) => t.should_write(),
            None => false,
        }
    }

    fn epee_default_value() -> Option<Self> {
        Some(None)
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        match self {
            Some(t) => t.write(w)?,
            None => panic!("Can't write an Option::None value, this should be handled elsewhere"),
        }
        Ok(())
    }
}
