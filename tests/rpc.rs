use epee_encoding::{from_bytes, to_bytes, EpeeObject};

#[derive(EpeeObject, Clone, Debug, PartialEq)]
struct BaseResponse {
    credits: u64,
    status: String,
    top_hash: String,
    untrusted: bool,
}

#[derive(EpeeObject, Clone, Debug, PartialEq)]
struct GetOIndexesResponse {
    base: BaseResponse,
    o_indexes: Vec<u64>,
}

#[derive(EpeeObject, Clone, Debug, PartialEq)]
struct GetOutsResponse {
    #[epee_flatten]
    base: BaseResponse,
    outs: Vec<OutKey>,
}

#[derive(EpeeObject, Clone, Copy, Debug, PartialEq)]
struct OutKey {
    height: u64,
    key: [u8; 32],
    mask: [u8; 32],
    txid: [u8; 32],
    unlocked: bool,
}

#[test]
fn rpc_get_outs_response() {
    let bytes = hex::decode("011101010101020101140763726564697473050000000000000000046f7574738c04140668656967687405a100000000000000036b65790a802d392d0be38eb4699c17767e62a063b8d2f989ec15c80e5d2665ab06f8397439046d61736b0a805e8b863c5b267deda13f4bc5d5ec8e59043028380f2431bc8691c15c83e1fea404747869640a80c0646e065a33b849f0d9563673ca48eb0c603fe721dd982720dba463172c246f08756e6c6f636b65640b00067374617475730a084f4b08746f705f686173680a0009756e747275737465640b00").unwrap();
    let val: GetOutsResponse = from_bytes(&bytes).unwrap();
    let bytes = to_bytes(&val).unwrap();

    assert_eq!(val, from_bytes(&bytes).unwrap());
}



#[derive(EpeeObject)]
struct Test2 {
    val: u64
}

fn main() {
    let data = [1, 17, 1, 1, 1, 1, 2, 1, 1, 4, 3, 118, 97, 108, 5, 4, 0, 0, 0, 0, 0, 0, 0]; // the data to decode;
    let val: Test2 = from_bytes(&data).unwrap();
    let data = to_bytes(&val).unwrap();
}