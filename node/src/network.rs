use libp2p::{
    kad::{record::store::MemoryStore, Kademlia, KademliaConfig, Quorum, Record},
    relay::v2::client::Client as RelayClient,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    PeerId, Multiaddr, identity, noise, tcp, yamux, Transport, core::upgrade, dns,
    mdns::Mdns,
    websocket::WsConfig,
};
use tokio::io::{self, AsyncBufReadExt};

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    kademlia: Kademlia<MemoryStore>,
    relay_client: RelayClient,
    mdns: Mdns,
}

impl MyBehaviour {
    async fn new(local_key: &identity::Keypair) -> Self {
        let local_peer_id = PeerId::from(local_key.public());
        let store = MemoryStore::new(local_peer_id);
        
        let mut cfg = KademliaConfig::default();
        cfg.set_query_timeout(std::time::Duration::from_secs(60));
        let kademlia = Kademlia::with_config(local_peer_id, store, cfg);

        let relay_client = RelayClient::new(local_peer_id);
        let mdns = Mdns::new(Default::default()).await.unwrap();

        MyBehaviour {
            kademlia,
            relay_client,
            mdns,
        }
    }
}

#[tokio::main]
async fn run_network() -> Result<(), Box<dyn std::error::Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("Local peer id: {:?}", local_peer_id);

    let transport = dns::TokioDnsConfig::system(tcp::TcpConfig::new().nodelay(true))?
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(local_key.clone()).into_authenticated())
        .multiplex(yamux::YamuxConfig::default())
        .or_transport(WsConfig::new(dns::TokioDnsConfig::system(tcp::TcpConfig::new().nodelay(true))?))
        .boxed();

    let behaviour = MyBehaviour::new(&local_key).await;
    let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.unwrap();
                let mut args = line.split(" ");
                match args.next() {
                    Some("PUT") => {
                        let key = args.next().unwrap().to_string();
                        let value = args.next().unwrap().as_bytes().to_vec();
                        swarm.behaviour_mut().kademlia.put_record(Record::new(key.into_bytes(), value), Quorum::One).unwrap();
                    },
                    Some("GET") => {
                        let key = args.next().unwrap().to_string();
                        swarm.behaviour_mut().kademlia.get_record(&key.into_bytes(), Quorum::One);
                    },
                    Some("DIAL") => {
                        let peer_id = args.next().unwrap().parse::<PeerId>().unwrap();
                        let address = args.next().unwrap().parse::<Multiaddr>().unwrap();
                        swarm.dial(address.clone()).unwrap();
                        println!("Dialed {} at {}", peer_id, address);
                    },
                    _ => {
                        println!("Unknown command");
                    },
                }
            }
            event = swarm.next() => {
                match event {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(kademlia_event)) => {
                        println!("Kademlia event: {:?}", kademlia_event);
                    },
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns_event)) => {
                        println!("Mdns event: {:?}", mdns_event);
                    },
                    SwarmEvent::Behaviour(MyBehaviourEvent::RelayClient(relay_event)) => {
                        println!("Relay event: {:?}", relay_event);
                    },
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                        println!("Connection established to {:?} via {:?}", peer_id, endpoint);
                    }
                    SwarmEvent::IncomingConnection { .. } => {
                        println!("Incoming connection");
                    }
                    _ => {}
                }
            }
        }
    }
}
