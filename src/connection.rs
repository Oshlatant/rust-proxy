use std::{io::ErrorKind, net::SocketAddr};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, Error},
    net::TcpStream,
};

//side effect
pub async fn handle_stream(mut stream: TcpStream, socket_addr: &SocketAddr) -> Result<(), Error> {
    println!("\n[*] New connection -");
    println!("[*] Socket: {} -", socket_addr);

    let mut req = [0; 4096];
    stream.peek(&mut req).await?;

    let (method, host) = match no_seffect::get_method_host(&req) {
        Some(headers) => headers,
        None => {
            //bad request
            let res = b"HTTP/1.1 400 Bad Request \r\n\r\n";
            stream.write_all(res).await?;

            return Err(Error::new(
                ErrorKind::InvalidInput,
                "[X] Bad request, closing socket -",
            ));
        }
    };

    if method == "CONNECT".to_string() {
        // empty the stream, needed because i only peeked to it just before
        stream.read(&mut req).await?;

        let tunnel = match no_seffect::https_tunnel(&host).await {
            Ok(tunnel) => {
                let res = b"HTTP/1.1 200 Connection established\r\n\r\n";
                stream.write_all(res).await?;

                tunnel
            }
            Err(e) => {
                let res: &[u8] = match e.kind() {
                    ErrorKind::InvalidInput => "HTTP/1.1 400 Bad Request \r\n\r\n".as_bytes(),
                    _ => "HTTP/1.1 500 Internal Server Error \r\n\r\n".as_bytes(),
                };
                stream.write_all(res).await?;

                return Err(e);
            }
        };

        transfer(stream, tunnel).await?;
    } else {
        let tunnel = no_seffect::tunnel(&host).await?;

        transfer(stream, tunnel).await?;
    }

    Ok(())
}

//side effect
async fn transfer(mut inbound: TcpStream, mut outbound: TcpStream) -> Result<(), Error> {
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

pub(super) mod no_seffect {

    use std::io::ErrorKind;
    use tokio::{
        io::Error,
        net::{lookup_host, TcpStream},
    };

    //function from tokio/example, cleanest possible
    //99% no side effect
    pub async fn https_tunnel(host: &str) -> Result<TcpStream, Error> {
        let addr = match lookup_host(host).await?.next() {
            Some(addr) => addr,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "[X] Invalid host, closing socket -",
                ))
            }
        };

        let tunnel = TcpStream::connect(addr).await?;
        println!("[*] Tunnel open to: {} ( {} ) -", addr, host);

        Ok(tunnel)
    }

    //99% no side effect
    pub async fn tunnel(host: &str) -> Result<TcpStream, Error> {
        let host = format!("{}:80", host);

        let addr = match lookup_host(&host).await?.next() {
            Some(addr) => addr,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "[X] Invalid host, closing socket -",
                ))
            }
        };

        let tunnel = TcpStream::connect(addr).await?;
        println!("[*] Tunnel open to: {} ( {} ) -", addr, host);

        Ok(tunnel)
    }

    //no side effect
    pub fn get_method_host(req: &[u8]) -> Option<(String, String)> {
        let req = String::from_utf8_lossy(req);
        let req = req.split_whitespace().collect::<Vec<&str>>();

        let method = req.get(0)?.to_string();
        let host = req.get(4)?.to_string();

        Some((method, host))
    }
}
