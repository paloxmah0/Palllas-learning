

use pallas::network::facades::PeerClient;
use pallas::network::miniprotocols::chainsync::{NextResponse};
use pallas::network::miniprotocols::{Point};
use hex::decode;
use pallas::ledger::traverse::{block};


pub async fn neting() {
let mut peer = PeerClient::connect
("https://cardano-preprod-v6.ogmios-m1.dmtr.host", 764824073).await.unwrap();
let client =peer.chainsync();
let known_point = Point::Specific(1654413,
     hex::decode("b0a1c8e5d9f1e8b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6").unwrap());
     let (point,_)=client.find_intersect(vec![known_point.clone()]).await.unwrap();
println!("Found intersection point: {:?}", point);
match point{
    Some(point)=> assert!(point == known_point),
    None=> panic!("No intersection found"),
}

let next_response = client.request_next().await.unwrap();
match next_response{
    NextResponse::RollForward(block,point )=>
    println!("Received new block: {:?} at point: {:?}", block, point),
    NextResponse::RollBackward(_,point )=>
    println!("Received rollback: {:?}", point),
    NextResponse::Await=> println!("No new blocks, awaiting..."),
    
}
}