#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    use binproto::server::BinProtoServer;
    use std::sync::Arc;

    let server = BinProtoServer::new();

    server.register_handler(0x0001, Arc::new(|payload| {
        println!("[Handler] Reçu {} octets", payload.len());
        payload // echo
    })).await;

    server.listen("0.0.0.0:8989").await
}