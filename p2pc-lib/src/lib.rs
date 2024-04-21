use std::hash::Hasher as _;

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

fn build_swarm(
    keypair: libp2p::identity::Keypair,
) -> anyhow::Result<libp2p::Swarm<libp2p::gossipsub::Behaviour>> {
    Ok(libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let gossipsub_message_id_function = |message: &libp2p::gossipsub::Message| {
                let mut s = std::hash::DefaultHasher::new();
                std::hash::Hash::hash(&message.data, &mut s);
                libp2p::gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = libp2p::gossipsub::ConfigBuilder::default()
                .heartbeat_interval(std::time::Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .validation_mode(libp2p::gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                .message_id_fn(gossipsub_message_id_function) // content-address messages. No two messages of the same content will be propagated.
                .build()?;

            let gossipsub: libp2p::gossipsub::Behaviour<
                libp2p::gossipsub::IdentityTransform,
                libp2p::gossipsub::AllowAllSubscriptionFilter,
            > = libp2p::gossipsub::Behaviour::new(
                libp2p::gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            Ok(gossipsub)
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(30))
        })
        .build())
}

async fn run_event_loop<F>(
    mut swarm: libp2p::Swarm<libp2p::gossipsub::Behaviour>,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<Action>,
    mut callback: F,
) where
    F: FnMut(Event) + Send + 'static,
{
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
}

impl P2pc {
    pub fn new<F>(keypair: libp2p::identity::Keypair, callback: F) -> anyhow::Result<Self>
    where
        F: FnMut(Event) + Send + 'static,
    {
        let swarm = build_swarm(keypair)?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move { run_event_loop(swarm, receiver, callback) });
        Ok(Self { sender })
    }

    pub fn execute(
        &mut self,
        action: Action,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.sender.send(action)
    }
}
