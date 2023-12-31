use epee_encoding::{from_bytes, to_bytes, EpeeObject};

#[derive(EpeeObject)]
struct T {
    val: Option<u8>,
}

#[test]
fn optional_val_not_in_data() {
    let bytes: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01\x01\x00";
    let t: T = from_bytes(bytes).unwrap();
    let bytes2 = to_bytes(&t).unwrap();
    assert_eq!(bytes, bytes2);
    assert!(t.val.is_none());
}

#[test]
fn optional_val_in_data() {
    let bytes = [
        0x01, 0x11, 0x01, 0x1, 0x01, 0x01, 0x02, 0x1, 0x1, 0x04, 0x03, b'v', b'a', b'l', 0x08, 21,
    ];
    let t: T = from_bytes(&bytes).unwrap();
    let bytes2 = to_bytes(&t).unwrap();
    assert_eq!(bytes.as_slice(), bytes2.as_slice());
    assert_eq!(t.val.unwrap(), 21);
}
