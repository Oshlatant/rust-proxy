use proxy::init;

#[tokio::main]
async fn main() {
    let config = init::server_config();
    proxy::process(config).await;
}
