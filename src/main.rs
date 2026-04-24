mod transaction;
mod apnetwork;
mod utility;

use pallas::ledger::traverse::MultiEraBlock;
use std::fs;



#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    println!("Hello, world!");
   // network::neting().await;
    // netcodec::netting();S
   let hex = fs::read_to_string("test/allegra1.block").expect("Unable to read file");
   let bytes= hex::decode(hex.trim()).expect("Decoding failed");
let block = MultiEraBlock::decode(&bytes).expect("Failed to parse block");

print!("Block transactions: {:?}", block.txs());

}