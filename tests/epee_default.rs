use epee_encoding::{from_bytes, to_bytes, EpeeObject};

#[derive(EpeeObject)]
pub struct Optional {
    val: u8,
    #[epee_default(-4)]
    optional_val: i32,
}

#[derive(EpeeObject)]
pub struct NotOptional {
    val: u8,
    optional_val: i32,
}

#[derive(EpeeObject, Default)]
pub struct NotPresent {
    val: u8,
}

#[test]
fn epee_default_does_not_encode() {
    let val = Optional {
        val: 1,
        optional_val: -4,
    };
    let bytes = to_bytes(&val).unwrap();

    assert!(from_bytes::<NotOptional>(&bytes).is_err());

    let val: Optional = from_bytes(&bytes).unwrap();
    assert_eq!(val.optional_val, -4);
    assert_eq!(val.val, 1);
}

#[test]
fn epee_non_default_does_encode() {
    let val = Optional {
        val: 8,
        optional_val: -3,
    };
    let bytes = to_bytes(&val).unwrap();

    assert!(from_bytes::<NotOptional>(&bytes).is_ok());

    let val: Optional = from_bytes(&bytes).unwrap();
    assert_eq!(val.optional_val, -3);
    assert_eq!(val.val, 8)
}

#[test]
fn epee_value_not_present_with_default() {
    let val = NotPresent { val: 76 };
    let bytes = to_bytes(&val).unwrap();

    assert!(from_bytes::<NotOptional>(&bytes).is_err());

    let val: Optional = from_bytes(&bytes).unwrap();
    assert_eq!(val.optional_val, -4);
    assert_eq!(val.val, 76)
}
