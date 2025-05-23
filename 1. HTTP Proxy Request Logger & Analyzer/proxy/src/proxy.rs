use std::{convert::Infallible, io, net::{SocketAddr, TcpStream}};
use chrono::Utc;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream as TokioTcpStream}};
use hyper::{body::Incoming, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use hyper::service::service_fn;

use crate::logger::ProxyLogger;

pub async fn run_proxy(addr: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    // Use logger for proxy start message
    ProxyLogger::log_proxy_start(addr);

    loop {
        let (mut client_stream, _) = listener.accept().await?;
        let peer_addr = client_stream.peer_addr().unwrap_or_else(|_| addr);
        
        // Use logger for connection acceptance
        ProxyLogger::log_connection_accepted(peer_addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut client_stream, peer_addr).await {
                ProxyLogger::log_error("client handling", &e);
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
    
    // Use logger for detailed request logging
    ProxyLogger::log_detailed_request(&request, conn_info);

    let mut lines = request.lines();
    if let Some(connect_line) = lines.next() {
        let parts: Vec<&str> = connect_line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "CONNECT" {
            let target = parts[1];
            if let Some((host, port)) = target.split_once(':') {
                // Use logger for CONNECT attempt
                ProxyLogger::log_connect_request(host, port);

                match TokioTcpStream::connect(target).await {
                    Ok(mut server_stream) => {
                        // Use logger for successful connection
                        ProxyLogger::log_connection_established(target);
                        
                        client_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

                        let (mut client_read, mut client_write) = client_stream.split();
                        let (mut server_read, mut server_write) = server_stream.split();

                        let client_to_server = tokio::io::copy(&mut client_read, &mut server_write);
                        let server_to_client = tokio::io::copy(&mut server_read, &mut client_write);

                        tokio::try_join!(client_to_server, server_to_client)?;
                        return Ok(());
                    }
                    Err(e) => {
                        // Use logger for failed connection
                        ProxyLogger::log_connection_failed(target, &e.to_string());
                        
                        client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}