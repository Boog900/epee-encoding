/// Helper functions when working with [`Read`] and [`Write`](io::Write).
use alloc::string::String;
use alloc::vec;

use std_shims::io;
use std_shims::io::{ErrorKind, Read};
use std_shims::vec::Vec;

pub(crate) fn read_bytes<R: Read, const N: usize>(r: &mut R) -> io::Result<[u8; N]> {
    let mut res = [0; N];
    r.read_exact(&mut res)?;
    Ok(res)
}

pub(crate) fn read_var_bytes<R: Read>(r: &mut R, len: usize) -> io::Result<Vec<u8>> {
    let mut res = vec![0; len];
    r.read_exact(&mut res)?;
    Ok(res)
}

pub(crate) fn read_byte<R: Read>(r: &mut R) -> io::Result<u8> {
    Ok(read_bytes::<_, 1>(r)?[0])
}

pub(crate) fn read_string<R: Read>(r: &mut R, len: usize) -> io::Result<String> {
    String::from_utf8(read_var_bytes(r, len)?)
        .map_err(|_| io::Error::new(ErrorKind::Other, "Invalid string"))
}
