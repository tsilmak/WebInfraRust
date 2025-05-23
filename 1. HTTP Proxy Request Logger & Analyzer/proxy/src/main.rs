use std::{io, net::SocketAddr};
mod proxy;
mod logger;

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 1, 0], 3000));
    proxy::run_proxy(addr).await
}