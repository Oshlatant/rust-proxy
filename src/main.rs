use std::{io::Error};

use proxy::init;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = init::server_config();
    proxy::process(config).await?;

    Ok(())
}
