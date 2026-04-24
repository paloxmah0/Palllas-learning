use pallas::network::facades::{PeerClient, PeerServer};
use pallas::network::miniprotocols::blockfetch::BlockRequest;
use pallas::network::miniprotocols::{blockfetch, Point};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]

pub async fn netting() {
    // ── Test data ────────────────────────────────────────────────
    let block_bodies = vec![
        hex::decode("deadbeefdeadbeef").unwrap(),
        hex::decode("c0ffeec0ffeec0ffee").unwrap(),
    ];

    let known_point = Point::Specific(
        1654413,
        hex::decode(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        ).unwrap(),
    );

    // ── Bind the listener before spawning server ─────────────────
    let listener = TcpListener::bind(
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3002)
    )
    .await
    .unwrap();

    // ── Server task ───────────────────────────────────────────────
    let server = tokio::spawn({
        let body = block_bodies.clone();
        let point = known_point.clone();

        async move {
            let mut peer_server = PeerServer::accept(&listener, 0)
                .await
                .expect("server: accept failed");

            let bf = peer_server.blockfetch();

            // Round 1: client requests a range, server streams blocks
            let BlockRequest(range) = bf
                .recv_while_idle()
                .await
                .expect("server: recv failed")
                .expect("server: client disconnected");

            assert_eq!(range, (point.clone(), point.clone()));
            assert_eq!(*bf.state(), blockfetch::State::Busy);

            bf.send_block_range(body)
                .await
                .expect("server: send_block_range failed");

            assert_eq!(*bf.state(), blockfetch::State::Idle);

            // Round 2: client requests again, server sends empty range
            let BlockRequest(_) = bf
                .recv_while_idle()
                .await
                .expect("server: recv failed")
                .expect("server: client disconnected");

            bf.send_block_range(vec![])
                .await
                .expect("server: empty send failed");

            assert_eq!(*bf.state(), blockfetch::State::Idle);

            // Round 3: client sends Done
            assert!(bf
                .recv_while_idle()
                .await
                .expect("server: recv failed")
                .is_none());

            assert_eq!(*bf.state(), blockfetch::State::Done);

            println!("[SERVER] BlockFetch exchange complete");
        }
    });

    // ── Client task ───────────────────────────────────────────────
    let client = tokio::spawn(async move {
        // Give server time to start
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut peer_client = PeerClient::connect("localhost:3002", 0)
            .await
            .expect("client: connect failed");

        let bf = peer_client.blockfetch();

        // Round 1: request range, receive blocks
        bf.send_request_range((known_point.clone(), known_point.clone()))
            .await
            .expect("client: send_request_range failed");

        assert!(bf
            .recv_while_busy()
            .await
            .expect("client: recv_while_busy failed")
            .is_some());

        let mut received_blocks = Vec::new();
        while let Some(block) = bf
            .recv_while_streaming()
            .await
            .expect("client: recv_while_streaming failed")
        {
            received_blocks.push(block);
        }

        assert_eq!(received_blocks, block_bodies);
        println!("[CLIENT] Received {} blocks", received_blocks.len());

        // Round 2: request range, receive empty
        bf.send_request_range((known_point.clone(), known_point.clone()))
            .await
            .expect("client: second request failed");

        assert!(bf
            .recv_while_busy()
            .await
            .expect("client: second recv failed")
            .is_none());

        // Round 3: send done
        bf.send_done()
            .await
            .expect("client: send_done failed");

        println!("[CLIENT] Done");
    });

    // ── Wait for both ─────────────────────────────────────────────
    tokio::try_join!(server, client).unwrap();
}
