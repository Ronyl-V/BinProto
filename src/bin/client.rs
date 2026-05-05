#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    use binproto::client::BinProtoClient;

    let mut client = BinProtoClient::connect("127.0.0.1:8989").await?;
    println!("Connecté à {}", client.peer_addr()?);

    let payload = b"hello server".to_vec();
    let (resp_type, resp) = client.send_raw(0x0001, payload).await?;
    println!("Réponse type=0x{:04X} : {:?}", resp_type, resp);

    Ok(())
}