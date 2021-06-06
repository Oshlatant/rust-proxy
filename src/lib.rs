mod connection;
pub mod init;

use config::Value;
use std::{collections::HashMap, net::SocketAddr, str::FromStr};
use tokio::{self, io::Error, net::TcpListener};

pub async fn process(config: HashMap<String, Value>) -> Result<(), Error> {
    let addr = init::get_caddr(&config);

    let socket = SocketAddr::from_str(&addr).unwrap();
    let listener = TcpListener::bind(socket).await.unwrap();

    println!("Server running on {}", listener.local_addr().unwrap());

    let ip_whitelist = init::get_ip_whitelist(&config);

    loop {
        let (stream, socket) = listener.accept().await.unwrap();

        //check if accepted socket ip is inside vector ipwhitelist, if yes -> spawn a task and handle the stream
        if let Ok(socket) = auth_ip(&ip_whitelist, socket) {
            let task =
                tokio::spawn(async move { connection::handle_stream(stream, &socket).await });
        }
    }
}

fn auth_ip(ip_whitelist: &Vec<String>, socket_add: SocketAddr) -> Result<SocketAddr, &str> {
    for ip in ip_whitelist.iter() {
        if ip.to_string() == socket_add.ip().to_string() {
            return Ok(socket_add);
        }
    }

    Err("Ip not allowed")
}
