use epee_encoding::{from_bytes, to_bytes, EpeeObject};
use crate::t::HandshakeR;

mod t {
    use super::*;
    #[derive(EpeeObject, Eq, PartialEq, Debug)]
    pub struct BasicNodeData {
        pub my_port: u32,
        pub network_id: [u8; 16],
        pub peer_id: u64,
        pub support_flags: u32,
    }

    #[derive(EpeeObject, Eq, PartialEq, Debug)]
    pub struct HandshakeR {
        #[epee_alt_name("node_data")]
        pub node_daa: BasicNodeData,
        #[epee_default(0)]
        pub test: u8,
    }
}
#[derive(EpeeObject, Eq, PartialEq, Debug)]
struct Test {
    val: u64,
}

#[test]
fn decode() {
    let bytes = hex::decode("01110101010102010108096e6f64655f646174610c10076d795f706f727406a04600000a6e6574776f726b5f69640a401230f171610441611731008216a1a11007706565725f6964053eb3c096c4471c340d737570706f72745f666c61677306010000000c7061796c6f61645f646174610c181563756d756c61746976655f646966666963756c7479053951f7a79aab4a031b63756d756c61746976655f646966666963756c74795f746f7036340500000000000000000e63757272656e745f68656967687405fa092a00000000000c7072756e696e675f73656564068001000006746f705f69640a806cc497b230ba57a95edb370be8d6870c94e0992937c89b1def3a4cb7726d37ad0b746f705f76657273696f6e0810").unwrap();

    let ty: HandshakeR = from_bytes(&bytes).unwrap();

    let bytes = to_bytes(&ty).unwrap();

    assert_eq!(ty, from_bytes(&bytes).unwrap());
}

#[test]
fn t() {
    let bytes = hex::decode("011101010101020101140763726564697473050000000000000000046f7574738c04140668656967687405a100000000000000036b65790a802d392d0be38eb4699c17767e62a063b8d2f989ec15c80e5d2665ab06f8397439046d61736b0a805e8b863c5b267deda13f4bc5d5ec8e59043028380f2431bc8691c15c83e1fea404747869640a80c0646e065a33b849f0d9563673ca48eb0c603fe721dd982720dba463172c246f08756e6c6f636b65640b00067374617475730a084f4b08746f705f686173680a0009756e747275737465640b00").unwrap();
    let val: GetOutsResponse = from_bytes(&bytes).unwrap();
    println!("{:?}", val);
}

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
