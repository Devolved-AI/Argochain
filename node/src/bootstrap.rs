use serde::{Deserialize, Serialize};
use libp2p::{
    autonat::{Behaviour as AutoNatBehaviour, Config as AutoNatConfig},
    relay::{Behaviour as RelayBehaviour, Config as RelayConfig},
    request_response::{RequestResponse, RequestResponseCodec, ProtocolSupport},
    swarm::{NetworkBehaviour, NetworkBehaviourEventProcess, SwarmBuilder},
    PeerId, Multiaddr,
};
use async_std::task;
use futures::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BootstrapRequest;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BootstrapResponse {
    peers: Vec<Multiaddr>,
}

#[derive(Clone)]
struct BootstrapCodec;

impl RequestResponseCodec for BootstrapCodec {
    type Protocol = String;
    type Request = BootstrapRequest;
    type Response = BootstrapResponse;

    fn encode_request(&mut self, _: &String, request: BootstrapRequest) -> Result<Vec<u8>, std::io::Error> {
        Ok(serde_json::to_vec(&request).unwrap())
    }

    fn decode_request(&mut self, _: &String, bytes: &[u8]) -> Result<BootstrapRequest, std::io::Error> {
        Ok(serde_json::from_slice(bytes).unwrap())
    }

    fn encode_response(&mut self, _: &String, response: BootstrapResponse) -> Result<Vec<u8>, std::io::Error> {
        Ok(serde_json::to_vec(&response).unwrap())
    }

    fn decode_response(&mut self, _: &String, bytes: &[u8]) -> Result<BootstrapResponse, std::io::Error> {
        Ok(serde_json::from_slice(bytes).unwrap())
    }
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    autonat: AutoNatBehaviour,
    relay: RelayBehaviour,
    request_response: RequestResponse<BootstrapCodec>,
}

impl NetworkBehaviourEventProcess<libp2p::request_response::RequestResponseEvent<BootstrapRequest, BootstrapResponse>> for MyBehaviour {
    fn inject_event(&mut self, event: libp2p::request_response::RequestResponseEvent<BootstrapRequest, BootstrapResponse>) {
        match event {
            libp2p::request_response::RequestResponseEvent::Message { peer, message } => match message {
                libp2p::request_response::RequestResponseMessage::Request { request, channel } => {
                    println!("Received bootstrap request from {:?}", peer);
                    let response = BootstrapResponse {
                        peers: vec!["/ip4/1.2.3.4/tcp/30333".parse().unwrap()],
                    };
                    self.request_response.send_response(channel, response).unwrap();
                }
                libp2p::request_response::RequestResponseMessage::Response { response } => {
                    println!("Received bootstrap response: {:?}", response);
                }
            },
            libp2p::request_response::RequestResponseEvent::OutboundFailure { peer, error, request_id } => {
                println!("Outbound failure to peer {:?}: {:?}", peer, error);
            }
            libp2p::request_response::RequestResponseEvent::InboundFailure { peer, error, request_id } => {
                println!("Inbound failure from peer {:?}: {:?}", peer, error);
            }
            libp2p::request_response::RequestResponseEvent::ResponseSent { peer, request_id } => {
                println!("Response sent to peer {:?}", peer);
            }
        }
    }
}

async fn build_swarm() -> Swarm<MyBehaviour> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let transport = TcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(libp2p::noise::NoiseConfig::xx(&local_key).unwrap())
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .or_transport(TcpConfig::new().multiplex(libp2p::mplex::MplexConfig::new()));

    let request_response_config = libp2p::request_response::RequestResponseConfig::default();
    let request_response = RequestResponse::new(
        BootstrapCodec,
        std::iter::once((String::from("/bootstrap/1.0.0"), ProtocolSupport::Full)),
        request_response_config,
    );

    let autonat_config = AutoNatConfig::default();
    let autonat = AutoNatBehaviour::new(local_peer_id.clone(), autonat_config);

    let relay_config = RelayConfig::default();
    let relay = RelayBehaviour::new(relay_config);

    let behaviour = MyBehaviour {
        autonat,
        relay,
        request_response,
    };

    SwarmBuilder::new(transport, behaviour, local_peer_id.clone())
        .executor(Box::new(|fut| { task::spawn(fut); }))
        .build()
}

async fn bootstrap(swarm: &mut Swarm<MyBehaviour>, bootstrap_peers: Vec<Multiaddr>) {
    for addr in bootstrap_peers {
        if let Ok(peer_id) = addr.iter().last().and_then(|comp| match comp {
            libp2p::multiaddr::Protocol::P2p(peer_id) => PeerId::from_multihash(peer_id.clone()).ok(),
            _ => None,
        }) {
            swarm.behaviour_mut().request_response.send_request(&peer_id, BootstrapRequest);
        }
    }

    loop {
        match swarm.next().await {
            _ => {}
        }
    }
}

