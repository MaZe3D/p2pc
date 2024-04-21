use libp2p::futures::StreamExt as _;

pub enum Action {
    ListenOn(libp2p::Multiaddr),
    Dial(libp2p::Multiaddr),
}

pub enum ActionResult {
    ListenOn(
        libp2p::Multiaddr,
        Option<libp2p::TransportError<std::io::Error>>,
    ),
    Dial(libp2p::Multiaddr, Option<libp2p::swarm::DialError>),
}

pub enum Event {
    ActionResult(ActionResult),
}

pub struct P2pc {
    sender: tokio::sync::mpsc::UnboundedSender<Action>,
}

impl P2pc {
    pub fn new<F>(keypair: libp2p::identity::Keypair, mut callback: F) -> anyhow::Result<Self>
    where
        F: FnMut(Event) + Send + 'static,
    {
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|_| libp2p::ping::Behaviour::default())?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(30))
            })
            .build();

        let (sender_outside_tokio, receiver_inside_tokio) = tokio::sync::mpsc::unbounded_channel();

        // start event loop
        tokio::spawn(async move {
            let mut receiver = receiver_inside_tokio;
            loop {
                let mut action_received = None;
                tokio::select! {
                    swarm_event = swarm.select_next_some() =>
                        match swarm_event {
                            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                                log::info!("listening on {address:?}")
                            }
                            libp2p::swarm::SwarmEvent::Behaviour(event) => println!("{event:?}"),
                            event => {
                                log::info!("{event:?}");
                            }
                        },
                    action = receiver.recv() => action_received = action
                }

                if let Some(action) = action_received {
                    callback(Event::ActionResult(match action {
                        Action::ListenOn(address) => {
                            ActionResult::ListenOn(address.clone(), swarm.listen_on(address).err())
                        }
                        Action::Dial(address) => {
                            ActionResult::Dial(address.clone(), swarm.dial(address).err())
                        }
                    }));
                }
            }
        });

        Ok(Self {
            sender: sender_outside_tokio,
        })
    }

    pub fn execute(
        &mut self,
        action: Action,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.sender.send(action)
    }
}
