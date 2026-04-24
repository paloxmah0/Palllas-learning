
use std::net::{Ipv4Addr, SocketAddrV4};
use pallas::network::facades::PeerClient;
use pallas::network::miniprotocols::txsubmission::{
    EraTxBody, EraTxId, Request, TxIdAndSize,
};
use crate::transaction::Transactions;

pub async fn protocol_happy_path(
    hub_utxo_hash: pallas::crypto::hash::Hash<32>,
    hub_utxo_index: u64,
    collateral_hash: pallas::crypto::hash::Hash<32>,
    collateral_index: u64,
    script_address: pallas::ledger::addresses::Address,
    lovelace_amount: u64,
    p: i64,
    d: i64,
    voters: Vec<pallas::ledger::primitives::conway::PlutusData>,
    hub_pkh: pallas::crypto::hash::Hash<28>,
    signing_key_bytes: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {

    //  Build transaction 
    let new_tx = Transactions::new(
        hub_utxo_hash,
        hub_utxo_index,
        collateral_hash,
        collateral_index,
        script_address,
        lovelace_amount,
        p,
        d,
        voters,
        hub_pkh,
        signing_key_bytes,
    )?;

    let tx_bytes = hex::decode(new_tx.serialize())?;

    //  Connect via PeerClient (N2N) 
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3001);
    let mut peer = PeerClient::connect(addr, 764824073).await?;

    // Get txsubmission client 
    let client = peer.txsubmission();

    //  Start protocol 
    client.send_init().await?;

    // Build tx id and body 
    let tx_id = EraTxId(6, tx_bytes[..32].to_vec());
    let tx_body = EraTxBody(6, tx_bytes.clone());

    // Protocol loop 
    loop {
        match client.next_request().await? {
            Request::TxIds(_ack, _req) => {
                client.reply_tx_ids(vec![
                    TxIdAndSize(tx_id.clone(), tx_bytes.len() as u32),
                ]).await?;
            }
            Request::TxIdsNonBlocking(_ack, _req) => {
                client.reply_tx_ids(vec![]).await?;
                break;
            }
            Request::Txs(_ids) => {
                client.reply_txs(vec![tx_body.clone()]).await?;
            }
        }
    }

    client.send_done().await?;

    println!("Transaction submitted: {}", new_tx.serialize());

    Ok(())
}