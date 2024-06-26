use std::hash::{Hash as _, Hasher as _};

use libp2p::futures::StreamExt as _;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub participants: Vec<String>,
    pub content: String,
    pub id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub answer_to: Option<uuid::Uuid>,
}

pub enum Action {
    ListenOn(libp2p::Multiaddr),
    Dial(libp2p::Multiaddr),
    SendMessage(ChatMessage),
}

pub enum ActionResult {
    SendMessage {
        message_id: uuid::Uuid,
        chat_id: uuid::Uuid,
        optional_errors: Vec<Option<libp2p::gossipsub::PublishError>>,
    },
}

pub enum Event {
    ActionResult(ActionResult),
    MessageReceived(ChatMessage),
    NewListenAddress(libp2p::Multiaddr),
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
                let mut hasher = std::hash::DefaultHasher::new();
                message.data.hash(&mut hasher);
                if let Some(sequence_number) = message.sequence_number {
                    sequence_number.hash(&mut hasher);
                }
                libp2p::gossipsub::MessageId::from(hasher.finish().to_string())
            };

            let gossipsub_config = libp2p::gossipsub::ConfigBuilder::default()
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
    let this_node_topic = libp2p::gossipsub::IdentTopic::new("chat");
    log::info!("subscribing to this node's topic: {}", this_node_topic);
    swarm.behaviour_mut().subscribe(&this_node_topic).ok();

    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => handle_swarm_event(&mut swarm, &swarm_event, &mut callback),
            Some(action) = receiver.recv() => handle_action(&mut swarm, action, &mut callback),
        }
    }
}

fn handle_swarm_event<F>(
    swarm: &mut libp2p::Swarm<libp2p::gossipsub::Behaviour>,
    swarm_event: &libp2p::swarm::SwarmEvent<libp2p::gossipsub::Event>,
    callback: &mut F,
) where
    F: FnMut(Event) + Send + 'static,
{
    match swarm_event {
        libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
            callback(Event::NewListenAddress(address.clone()));
        }
        libp2p::swarm::SwarmEvent::Behaviour(libp2p::gossipsub::Event::Message {
            message, ..
        }) => {
            if let Ok(serialized_chat_message) = String::from_utf8(message.data.clone()) {
                if let Ok(mut chat_message) =
                    serde_json::from_str::<ChatMessage>(&serialized_chat_message)
                {
                    // remove own id
                    let local_id = &swarm.local_peer_id().to_string();
                    if chat_message.participants.contains(local_id) {
                        chat_message.participants = chat_message
                            .participants
                            .into_iter()
                            .filter(|participant| participant != local_id)
                            .collect();

                        callback(Event::MessageReceived(chat_message));
                    }
                }
            }
        }
        event => log::info!("{event:?}"),
    }
}

fn handle_action<F>(
    swarm: &mut libp2p::Swarm<libp2p::gossipsub::Behaviour>,
    action: Action,
    callback: &mut F,
) where
    F: FnMut(Event) + Send + 'static,
{
    match action {
        Action::ListenOn(address) => {
            swarm.listen_on(address).ok();
        }
        Action::Dial(address) => {
            swarm.dial(address).ok();
        }
        Action::SendMessage(mut chat_message) => {
            let destinations = chat_message.participants.clone();
            chat_message
                .participants
                .push(swarm.local_peer_id().to_string());
            let optional_errors = match serde_json::to_string(&chat_message) {
                Ok(serialized_message) => destinations
                    .iter()
                    .map(|participant| {
                        log::debug!("participant: {}", participant);
                        let topic = libp2p::gossipsub::IdentTopic::new("chat");
                        swarm
                            .behaviour_mut()
                            .publish(topic, serialized_message.as_bytes())
                            .err()
                    })
                    .collect(),
                Err(_) => vec![],
            };
            callback(Event::ActionResult(ActionResult::SendMessage {
                message_id: chat_message.id,
                chat_id: chat_message.chat_id,
                optional_errors,
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
        tokio::spawn(run_event_loop(swarm, receiver, callback));
        Ok(Self { sender })
    }

    pub fn execute(
        &mut self,
        action: Action,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        self.sender.send(action)
    }
}
