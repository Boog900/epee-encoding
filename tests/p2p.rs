use epee_encoding::{from_bytes, to_bytes, EpeeObject};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct SupportFlags(u32);

impl From<u32> for SupportFlags {
    fn from(value: u32) -> Self {
        SupportFlags(value)
    }
}

impl From<SupportFlags> for u32 {
    fn from(value: SupportFlags) -> Self {
        value.0
    }
}

#[derive(EpeeObject, Eq, PartialEq, Debug)]
pub struct BasicNodeData {
    pub my_port: u32,
    pub network_id: [u8; 16],
    pub peer_id: u64,
    #[epee_try_from_into(u32)]
    pub support_flags: SupportFlags,
}

#[derive(EpeeObject, Eq, PartialEq, Debug)]
pub struct HandshakeR {
    pub node_data: BasicNodeData,
}

#[test]
fn p2p_handshake() {
    let bytes = hex::decode("01110101010102010108096e6f64655f646174610c10076d795f706f727406a04600000a6e6574776f726b5f69640a401230f171610441611731008216a1a11007706565725f6964053eb3c096c4471c340d737570706f72745f666c61677306010000000c7061796c6f61645f646174610c181563756d756c61746976655f646966666963756c7479053951f7a79aab4a031b63756d756c61746976655f646966666963756c74795f746f7036340500000000000000000e63757272656e745f68656967687405fa092a00000000000c7072756e696e675f73656564068001000006746f705f69640a806cc497b230ba57a95edb370be8d6870c94e0992937c89b1def3a4cb7726d37ad0b746f705f76657273696f6e0810").unwrap();

    let val: HandshakeR = from_bytes(&bytes).unwrap();

    let bytes = to_bytes(&val).unwrap();

    assert_eq!(val, from_bytes(&bytes).unwrap());
}
