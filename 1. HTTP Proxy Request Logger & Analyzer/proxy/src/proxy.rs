use std::{convert::Infallible, io, net::{SocketAddr, TcpStream}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream as TokioTcpStream}};
use hyper::{body::Incoming, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use hyper::service::service_fn;

pub async fn run_proxy(addr: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    println!("Proxy running at http://{}", addr);

    loop {
        let (mut client_stream, _) = listener.accept().await?;
        let peer_addr = client_stream.peer_addr().unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));

        // Handle incoming requests, including the CONNECT method
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut client_stream, peer_addr).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}

pub async fn handle_client(client_stream: &mut TokioTcpStream, conn_info: SocketAddr) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    let n = client_stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(()); // Ignore empty requests
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    println!("Raw buffer: {:?}", &buf[..n]);
    println!("Request: {}", request);
    println!("Client IP: {}", get_client_ip(conn_info));

    let mut lines = request.lines();
    if let Some(connect_line) = lines.next() {
        let parts: Vec<&str> = connect_line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "CONNECT" {
            let target = parts[1];
            if let Some((host, port)) = target.split_once(':') {
                println!("CONNECT to {}:{}", host, port);
                if let Ok(std_server_stream) = TcpStream::connect(target) {
                    let mut server_stream = TokioTcpStream::from_std(std_server_stream)?;
                    client_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
                    let (mut client_read, mut client_write) = client_stream.split();
                    let (mut server_read, mut server_write) = server_stream.split();
                    let client_to_server = tokio::io::copy(&mut client_read, &mut server_write);
                    let server_to_client = tokio::io::copy(&mut server_read, &mut client_write);
                    tokio::try_join!(client_to_server, server_to_client)?;
                    return Ok(());
                } else {
                    client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
                    return Ok(());
                }
            }
        }
    }

    // Handle non-CONNECT requests without cloning
    let service = service_fn(move |req| handle_request(req, conn_info));
    let io = TokioIo::new(client_stream); // Use the stream directly
    let conn = hyper::server::conn::http1::Builder::new().serve_connection(io, service);
    if let Err(err) = conn.await {
        eprintln!("Connection error: {:?}", err);
    }

    Ok(())
}

pub async fn handle_request(
    req: Request<Incoming>,
    conn_info: SocketAddr,
) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {

    let location = req.uri().to_string();
    println!("Request URI: {}", location);

    let headers = req.headers();
    println!("Headers: {:?}", headers);

    let client_ip = get_client_ip(conn_info);
    println!("Logging client info: {}", client_ip);

    let response = Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", location)
        .body(Full::from(hyper::body::Bytes::new()))
        .unwrap();

    Ok(response)
}

pub fn get_client_ip(conn_info: SocketAddr) -> String {
    conn_info.ip().to_string()
}