use std::{io::Error, net::SocketAddr};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, lookup_host}};

// const IP: [u8; 4] = [127, 0, 0, 1];
const IP: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() {
    
    let socket = SocketAddr::from((IP, 3000));
    let listener = TcpListener::bind(socket).await.unwrap();

    println!("Server running on {}", listener.local_addr().unwrap());

    loop {

        let (stream, socket) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            handle_stream(stream, socket).await.unwrap();
        });
    }

}

async fn handle_stream(stream: TcpStream, socket_addr: SocketAddr) -> Result<(), Error> {

    println!("\n[*] New task !!");
    println!("client socket: {}", socket_addr);

    let mut req = [0; 4096];
    stream.peek(&mut req).await?;
    let ( method, host ) = get_method_host(&req);


    if method == "CONNECT".to_string() {
        https_connection(stream, &host).await;
    } else {
        connection(stream, &host).await;
    }
    
    Ok(())
}


async fn https_connection(mut stream: TcpStream, host: &str) {

    let mut trash = [0; 4096];
    // let mut trash = Vec::new();

    //empty the stream, needed because i only peeked to it just before
    stream.read(&mut trash).await.unwrap();


    let mut host = lookup_host(host).await.unwrap();
    let addr = host.next().unwrap();

    let tunnel = match TcpStream::connect(addr).await {
        Ok(tunnel) => {
            let res = b"HTTP/1.1 200 Connection established\r\n\r\n";
            stream.write_all(res).await.unwrap();

            tunnel
        },
        Err(_) => return (),
    };
    
    println!("CONNECTED");
    transfer(stream, tunnel).await.unwrap();
}

async fn connection(stream: TcpStream, host: &str) {
    let host = format!("{}:80", host);

    let mut host = lookup_host(host).await.unwrap();
    let addr = host.next().unwrap();

    let tunnel = TcpStream::connect(addr).await.unwrap();

    transfer(stream, tunnel).await.unwrap();
}


//function from tokio/example, cleanest possible
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

fn get_method_host(req: &[u8]) -> ( String, String ) {
    let req = String::from_utf8_lossy(req);
    let req = req.split_whitespace().collect::<Vec<&str>>();

    let method = req.get(0).unwrap().to_string();
    let host = req.get(4).unwrap().to_string();

    ( method, host )
} 