mod connection;
pub mod init;

use config::Value;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    str::FromStr,
};
use tokio::{self, io::Error, net::TcpListener};

pub async fn process(config: HashMap<String, Value>) -> Result<(), Error> {
    let addr = init::get_caddr(&config);

    let socket = SocketAddr::from_str(&addr).expect("Incorrect server address, edit Config.toml");
    let listener = TcpListener::bind(socket).await.expect("Address not free");
    let ip_whitelist = init::get_ip_whitelist(&config);

    println!("Server running on {}", addr);
    //from this point, panic is not allowed
    loop {
        let (stream, socket) = listener.accept().await?;

        // check if accepted socket ip is inside hashset ipwhitelist, if yes -> spawn a task and handle the stream
        if let Ok(socket) = auth_ip(&ip_whitelist, socket) {
            tokio::spawn(async move {
                connection::handle_stream(stream, &socket).await?;

                Ok::<(), Error>(())
            });
        }
    }
}

fn auth_ip(ip_whitelist: &HashSet<String>, socket_add: SocketAddr) -> Result<SocketAddr, ()> {
    let unknow_ip = socket_add.ip().to_string();

    match ip_whitelist.get(&unknow_ip) {
        Some(_) => Ok(socket_add),
        None => Err(println!("Ip not allowed")),
    }
}
