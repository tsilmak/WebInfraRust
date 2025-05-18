use std::{io, net::SocketAddr};
mod proxy;

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    proxy::run_proxy(addr).await
}