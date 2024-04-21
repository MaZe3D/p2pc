use clap::Parser as _;

/// Start or connect to an existing p2pc network
#[derive(clap::Parser, Debug)]
struct CliArguments {
    /// Initial peers to connect to
    #[arg(short, long, num_args(0..), value_name="MULTIADDRESS")]
    peer_addresses: Vec<libp2p::Multiaddr>,

    /// Interfaces to listen on
    #[arg(short, long, num_args(0..), value_name="MULTIADDRESS", default_value = "/ip6/::/tcp/0")]
    listen_addresses: Vec<libp2p::Multiaddr>,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = CliArguments::parse();
    log::info!("{:?}", args);

    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let mut p2pc = p2pc_lib::P2pc::new(keypair).unwrap();

    for address in args.listen_addresses {
        p2pc.execute(p2pc_lib::Action::ListenOn(address));
    }
    for address in args.peer_addresses {
        p2pc.execute(p2pc_lib::Action::Dial(address));
    }

    loop {
        while let Ok(event) = p2pc.poll_event() {
            match event {
                p2pc_lib::Event::ActionResult(action_result) => match action_result {
                    p2pc_lib::ActionResult::ListenOn(address, None) => {
                        log::info!("successfully listening on {}", address);
                    }
                    p2pc_lib::ActionResult::ListenOn(address, Some(err)) => {
                        log::info!("failed to listen on {}: {}", address, err);
                    }
                    p2pc_lib::ActionResult::Dial(address, None) => {
                        log::info!("successfully dialed {}", address);
                    }
                    p2pc_lib::ActionResult::Dial(address, Some(err)) => {
                        log::info!("failed to dial {}: {}", address, err);
                    }
                },
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
