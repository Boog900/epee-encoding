use epee_encoding::{EpeeObject, from_bytes, to_bytes};

#[derive(EpeeObject)]
struct AltName {
    #[epee_alt_name("val2")]
    val: u8,
    d: u64,
}

#[derive(EpeeObject)]
struct AltName2 {
    val2: u8,
    d: u64
}

#[test]
fn epee_alt_name() {
    let val2 = AltName2 {
        val2: 40,
        d: 30
    };
    let bytes = to_bytes(&val2).unwrap();

    let val: AltName = from_bytes(&bytes).unwrap();

    let bytes2 = to_bytes(&val).unwrap();

    assert_eq!(bytes, bytes2);
}
