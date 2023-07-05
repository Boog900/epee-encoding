use epee_encoding::{EpeeObject, from_bytes, to_bytes};

#[derive(EpeeObject)]
struct Child {
    val: u64,
    val2: Vec<u8>,
}

#[derive(EpeeObject)]
struct Parent {
    #[epee_flatten]
    child: Child,
    h: f64,
}

#[derive(EpeeObject)]
struct ParentChild {
    h: f64,
    val: u64,
    val2: Vec<u8>,
}


#[test]
fn epee_flatten() {
    let val2 = ParentChild {
        h: 38.9,
        val: 94,
        val2: vec![4, 5],
    };
    let bytes = to_bytes(&val2).unwrap();

    let val: Parent = from_bytes(&bytes).unwrap();

    assert_eq!(val.child.val2, val2.val2);
    assert_eq!(val.child.val, val2.val);
    assert_eq!(val.h, val2.h);
}

#[derive(EpeeObject, Default, Debug, PartialEq)]
struct Child1 {
    val: u64,
    val2: Vec<u8>,
}

#[derive(EpeeObject, Default, Debug, PartialEq)]
struct Child2 {
    buz: u16,
    fiz: String,
}

#[derive(EpeeObject, Default, Debug, PartialEq)]
struct Parent12 {
    #[epee_flatten]
    child1: Child1,
    #[epee_flatten]
    child2: Child2,
    h: f64,
}

#[test]
fn epee_double_flatten() {
    let val = Parent12::default();

    let bytes = to_bytes(&val).unwrap();

    let val1: Parent12 = from_bytes(&bytes).unwrap();

    assert_eq!(val, val1);
}