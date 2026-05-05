#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    binproto::debugger::run_debugger().await
}