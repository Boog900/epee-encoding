/// This module contains `Read` and `Write` traits for no-std,
///
/// This was taken from std-shims which is licensed under MIT and
/// Copyright (c) 2023 Luke Parker.
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

use crate::{Error, Result};

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let read = self.read(buf)?;
        if read != buf.len() {
            Err(Error::IO("Reader ran out of bytes"))?;
        }
        Ok(())
    }
}

impl Read for &[u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read = buf.len();
        if self.len() < buf.len() {
            read = self.len();
        }
        buf[..read].copy_from_slice(&self[..read]);
        *self = &self[read..];
        Ok(read)
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.write(buf)? != buf.len() {
            Err(Error::IO("Writer ran out of bytes"))?;
        }
        Ok(())
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend(buf);
        Ok(buf.len())
    }
}

pub(crate) fn read_bytes<R: Read, const N: usize>(r: &mut R) -> Result<[u8; N]> {
    let mut res = [0; N];
    r.read_exact(&mut res)?;
    Ok(res)
}

pub(crate) fn read_var_bytes<R: Read>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut res = vec![0; len];
    r.read_exact(&mut res)?;
    Ok(res)
}

pub(crate) fn read_byte<R: Read>(r: &mut R) -> Result<u8> {
    Ok(read_bytes::<_, 1>(r)?[0])
}

pub(crate) fn read_string<R: Read>(r: &mut R, len: usize) -> Result<String> {
    String::from_utf8(read_var_bytes(r, len)?).map_err(|_| Error::Format("Invalid string"))
}
