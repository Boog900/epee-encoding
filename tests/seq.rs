use epee_encoding::{from_bytes, EpeeObject};

#[derive(EpeeObject)]
struct ObjSeq {
    seq: Vec<ObjSeq>,
}

#[derive(EpeeObject)]
struct ValSeq {
    seq: Vec<i64>,
}

#[test]
fn seq_with_zero_len_can_have_any_marker() {
    let mut data = [
        0x01, 0x11, 0x01, 0x1, 0x01, 0x01, 0x02, 0x1, 0x1, 0x04, 0x03, b's', b'e', b'q',
    ]
    .to_vec();
    for marker in 1..13 {
        data.push(0x80 | marker);
        data.push(0);

        assert!(from_bytes::<ObjSeq>(&data).is_ok());

        assert!(from_bytes::<ValSeq>(&data).is_ok());

        data.drain(14..);
    }
}

#[test]
fn seq_with_non_zero_len_must_have_correct_marker() {
    let mut data = [
        0x01, 0x11, 0x01, 0x1, 0x01, 0x01, 0x02, 0x1, 0x1, 0x04, 0x03, b's', b'e', b'q',
    ]
    .to_vec();
    for marker in 2..13 {
        // 1 is the marker for i64
        data.push(0x80 | marker);
        data.push(0x04); // varint length of 1
        data.extend_from_slice(&1_i64.to_le_bytes());

        assert!(from_bytes::<ValSeq>(&data).is_err());

        data.drain(14..);
    }

    data.push(0x80 + 1);
    data.push(0x04); // varint length
    data.extend_from_slice(&1_i64.to_le_bytes());
    (from_bytes::<ValSeq>(&data).unwrap());
}
