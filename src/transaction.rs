use pallas::codec::minicbor;
use pallas::ledger::addresses::Address;                  
use pallas::ledger::primitives::{
    BoundedBytes, Constr, BigInt, MaybeIndefArray,Fragment
};
use pallas::ledger::primitives::conway::PlutusData;
use pallas::crypto::hash::Hash;                       
use pallas::txbuilder::{Input, Output, StagingTransaction ,BuildConway};
use serde::Serialize;
use pallas_wallet::{PrivateKey};
use pallas::crypto::key::ed25519::SecretKeyExtended;



#[derive(Debug, Serialize)]
pub struct Transactions {
    tx: Vec<u8>,
}

impl Transactions {


    pub fn new(
        hub_utxo_hash: Hash<32>,
        hub_utxo_index: u64,
        collateral_hash: Hash<32>,
        collateral_index: u64,
        script_address: Address,
        lovelace_amount: u64,
        p: i64,
        d: i64,
        voters_as_plutus_data: Vec<PlutusData>,
        hub_pkh: Hash<28>,
        signing_key_bytes: Vec<u8>,
    ) -> Result<Self, Box<dyn std::error::Error>> {

        let datum = PlutusData::Constr(Constr {
            tag: 233,
            any_constructor: None,
            fields: MaybeIndefArray::Def(vec![
                PlutusData::BigInt(BigInt::Int(p.into())),
                PlutusData::BigInt(BigInt::Int(d.into())),
                PlutusData::Array(MaybeIndefArray::Def(voters_as_plutus_data)),
                PlutusData::BoundedBytes(BoundedBytes::from(hub_pkh.to_vec())),
            ]),
        });
           let raw = datum.encode_fragment()?;
           let mut datum_bytes = Vec::new();
minicbor::encode(minicbor::bytes::ByteVec::from(raw), &mut datum_bytes)?;
   


        let built = StagingTransaction::new()
            .input(Input::new(hub_utxo_hash, hub_utxo_index))
            .collateral_input(Input::new(collateral_hash, collateral_index))
            .output(
                Output::new(script_address, lovelace_amount)
                    .set_inline_datum(datum_bytes),
            )
            .disclosed_signer(hub_pkh)
            .fee(300_000)
            .build_conway_raw()?;

        //  Sign
        let key_array: [u8; 64] = signing_key_bytes
            .try_into()
            .map_err(|_| "signing key must be exactly 64 bytes")?;

        let sk = SecretKeyExtended::from_bytes(key_array)
            .map_err(|e| format!("invalid signing key: {:?}", e))?;

        let private_key = PrivateKey::from(sk);

        let signed = built.sign(private_key)?;

        //  Extract bytes
        let bytes = signed.tx_bytes.0.to_vec();

        Ok(Self { tx: bytes })
    }

    pub fn serialize(&self) -> String {
        hex::encode(&self.tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pallas::crypto::hash::Hash;
    use pallas::ledger::addresses::Address;
    use pallas::ledger::primitives::conway::PlutusData;

    fn make_hash32(byte: u8) -> Hash<32> {
        Hash::from([byte; 32])
    }

    fn make_hash28(byte: u8) -> Hash<28> {
        Hash::from([byte; 28])
    }

    fn make_address() -> Address {
        Address::from_bech32(
            "addr_test1wz5qc7fk2pat0058w4zwvkw35ytptej3nuc3je2kgtan5dq3rt4sc"
        ).unwrap()
    }

    fn make_signing_key() -> Vec<u8> {
        let mut key = [0u8; 64];
        key[0]  &= 0b1111_1000;
        key[31] &= 0b0011_1111;
        key[31] |= 0b0100_0000;
        key.to_vec()
    }

    #[test]
    fn test_new_returns_ok_with_valid_inputs() {
        let result = Transactions::new(
            make_hash32(0xaa), 0,
            make_hash32(0xbb), 0,
            make_address(),
            2_000_000, 10, 5, vec![],
            make_hash28(0xcc),
            make_signing_key(),
        );
        assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
    }

    #[test]
    fn test_serialize_returns_hex_string() {
        let tx = Transactions::new(
            make_hash32(0xaa), 0,
            make_hash32(0xbb), 0,
            make_address(),
            2_000_000, 10, 5, vec![],
            make_hash28(0xcc),
            make_signing_key(),
        ).unwrap();

        let hex = tx.serialize();
        assert!(!hex.is_empty());
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_signing_key_wrong_length_returns_err() {
        let result = Transactions::new(
            make_hash32(0xaa), 0,
            make_hash32(0xbb), 0,
            make_address(),
            2_000_000, 10, 5, vec![],
            make_hash28(0xcc),
            vec![0u8; 32], 
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("64 bytes") || msg.contains("signing key"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn test_datum_with_voters() {
        use pallas::ledger::primitives::BigInt;

        let voters = vec![
            PlutusData::BigInt(BigInt::Int(1_i64.into())),
            PlutusData::BigInt(BigInt::Int(2_i64.into())),
        ];
        let result = Transactions::new(
            make_hash32(0x01), 1,
            make_hash32(0x02), 0,
            make_address(),
            5_000_000, 100, 50,
            voters,
            make_hash28(0x03),
            make_signing_key(),
        );
        assert!(result.is_ok(), "expected Ok with voters: {:?}", result.err());
    }

    #[test]
    fn test_different_utxo_indexes() {
        for index in [0u64, 1, 5, 100] {
            let result = Transactions::new(
                make_hash32(0xaa), index,
                make_hash32(0xbb), 0,
                make_address(),
                2_000_000, 0, 0, vec![],
                make_hash28(0xcc),
                make_signing_key(),
            );
            assert!(
                result.is_ok(),
                "failed for utxo_index={index}: {:?}", result.err()
            );}}}