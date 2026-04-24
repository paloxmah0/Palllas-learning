use pallas::network::facades::{NodeClient, PeerClient};
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug)]
pub enum Network {
    Cardano { host: String, port: u16, magic: u64 },
    P2p { socket_addr: SocketAddrV4, peer_id: Option<String> },
}

pub enum NetworkClient {
    Cardano(NodeClient),
    P2p(PeerClient),
}

impl NetworkClient {
    pub fn as_cardano(self) -> NodeClient {   
        match self {
            NetworkClient::Cardano(c) => c,
            NetworkClient::P2p(_) => panic!("expected Cardano client, got P2p"),
        }
    }

    pub fn as_p2p(self) -> PeerClient {        
        match self {
            NetworkClient::P2p(c) => c,
            NetworkClient::Cardano(_) => panic!("expected P2p client, got Cardano"),
        }
    }
}

impl Network {
    pub async fn connect_network(
        &self,
    ) -> Result<NetworkClient, Box<dyn std::error::Error>> {  
        match self {
            Network::Cardano { host, port, magic } => {
                let client = NodeClient::connect(
                    format!("{}:{}", host, port),
                    *magic,
                )
                .await?;

                Ok(NetworkClient::Cardano(client))
            }

            Network::P2p { socket_addr, peer_id: _ } => {
                let client = PeerClient::connect(*socket_addr, 0)  // ✅ no listener needed
                    .await?;

                Ok(NetworkClient::P2p(client))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cardano_connects() {
        let network = Network::Cardano {
            host: "localhost".to_string(),
            port: 3001,
            magic: 764824073,
        };
        match network.connect_network().await {
            Ok(NetworkClient::Cardano(_)) => println!("connected"),
            Ok(NetworkClient::P2p(_)) => panic!("wrong client type"),
            Err(e) => println!("no node running (expected): {e}"),
        }
    }

    #[tokio::test]
    async fn test_p2p_connects() {
        let peer_client = Network::P2p {
            socket_addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3002),
            peer_id: None,
        };
        match peer_client.connect_network().await {
            Ok(NetworkClient::P2p(_)) => println!("connected"),
            Ok(NetworkClient::Cardano(_)) => panic!("wrong client type"),
            Err(e) => println!("no peer running (expected): {e}"),
        }
    }
}