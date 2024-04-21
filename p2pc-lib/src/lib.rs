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
    tokio_runtime: tokio::runtime::Runtime,
    sender: tokio::sync::mpsc::Sender<Action>,
    receiver: std::sync::mpsc::Receiver<Event>,
}

impl P2pc {
    pub fn new(keypair: libp2p::identity::Keypair) -> anyhow::Result<Self> {
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

        let (sender_outside_tokio, receiver_inside_tokio) = tokio::sync::mpsc::channel(100);
        let (sender_inside_tokio, receiver_outside_tokio) = std::sync::mpsc::channel();

        // start tokio runtime
        let tokio_runtime = tokio::runtime::Runtime::new()?;
        tokio_runtime.spawn(async move {
            let sender = sender_inside_tokio;
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

                match action_received {
                    None => {}
                    Some(action) => {
                        sender
                            .send(Event::ActionResult(match action {
                                Action::ListenOn(address) => ActionResult::ListenOn(
                                    address.clone(),
                                    swarm.listen_on(address).err(),
                                ),
                                Action::Dial(address) => {
                                    ActionResult::Dial(address.clone(), swarm.dial(address).err())
                                }
                            }))
                            .unwrap();
                    }
                }
            }
        });

        Ok(Self {
            tokio_runtime,
            sender: sender_outside_tokio,
            receiver: receiver_outside_tokio,
        })
    }

    pub fn execute(&mut self, action: Action) {
        let sender = self.sender.clone();
        self.tokio_runtime
            .spawn(async move { sender.send(action).await });
    }

    pub fn poll_event(&self) -> anyhow::Result<Event> {
        Ok(self.receiver.try_recv()?)
    }
}
