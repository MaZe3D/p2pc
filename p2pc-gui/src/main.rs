use clap::Parser as _;

/// Start or connect to an existing p2pc network
#[derive(clap::Parser, Debug)]
struct CliArguments {
    /// Initial peers to connect to
    #[arg(short, long, num_args(1..), value_name="MULTIADDRESS")]
    peer_addresses: Vec<libp2p::Multiaddr>,

    /// Interfaces to listen on
    #[arg(short, long, num_args(1..), value_name="MULTIADDRESS", default_value = "/ip6/::/tcp/0")]
    listen_addresses: Vec<libp2p::Multiaddr>,
}

#[tokio::main]
async fn main() {
    let args = CliArguments::parse();

    println!("args: {:?}", args);
}
