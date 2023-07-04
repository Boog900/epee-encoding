
use crate::error::*;
use crate::io::*;

pub fn read_varint<R: Read>(reader: &mut R) -> Result<u64> {
    let vi_start = read_byte(reader)?;
    let len = match vi_start & 0b11 {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        _ => unreachable!(),
    };
    let mut vi = u64::from(vi_start >> 2);
    for i in 1..len {
        vi |= u64::from(read_byte(reader)?) << (((i - 1) * 8) + 6);
    }
    Ok(vi)
}

pub fn write_varint<W: Write>(number: u64, writer: &mut W) -> Result<()> {
    let size_marker = match number {
        ..=63 => 0,
        64..=16383 => 1,
        16384..=1073741823 => 2,
        _ => 3,
    };

    let number = (number << 2) | size_marker;

    // Although `as` is unsafe we just checked the length.
    match size_marker {
        0 => writer.write_all(&[number as u8]),
        1 => writer.write_all(&(number as u16).to_le_bytes()),
        2 => writer.write_all(&(number as u32).to_le_bytes()),
        3 => writer.write_all(&(number).to_le_bytes()),
        _ => unreachable!(),
    }
}

#[test]
fn test_varint() {
    let _buf = [(64 << 2)];
}
