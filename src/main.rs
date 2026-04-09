mod network;

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    println!("Hello, world!");
    network::neting().await;
}