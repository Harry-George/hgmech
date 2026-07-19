use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(default_value = "0.0.0.0:3536", env)]
    pub host: SocketAddr,
}
