use structopt::StructOpt;
use std::path::PathBuf;
use holochain_p2p::kitsune_p2p::{KitsuneP2pConfig, TransportConfig, NetworkType};
use url2::Url2;
use snapmail_common::{
   globals::*,
   conductor::*,
   config::*,
};
use snapmail::handle::*;

/// This creates a new holochain sandbox
/// which is a
/// - conductor config
/// - databases
/// - keystore
#[derive(Debug, StructOpt, Clone)]
pub struct SetupCommand {

   #[structopt(subcommand, name = "network")]
   pub maybe_network: Option<NetworkCmd>,

   /// Network ID that this session will use
   uid: String,

   #[structopt(long, parse(from_os_str))]
   pub dna_path: Option<PathBuf>,

   // Set a root directory for the app's storage data to be placed into.
   // Defaults to the system's temp directory.
   // This directory must already exist.
   // #[structopt(name = "root", parse(from_os_str))]
   // pub maybe_root: Option<PathBuf>,
}

impl SetupCommand {
   ///
   pub async fn run(&self, sid: PathBuf) -> anyhow::Result<()> {
      let sid_str = sid.to_string_lossy().to_string();
      //let root = self.maybe_root.clone().unwrap_or(CONFIG_PATH.as_path().to_path_buf());
      let root = CONFIG_PATH.as_path().to_path_buf();
      let _ = generate(
         root,
         Some(sid.clone()),
         self.maybe_network.clone().map(|n| n.into_inner().into()),
      ).expect("Generate config failed. Maybe Invalid params.");


      let dna_hash = install_app(sid.to_string_lossy().to_string(), self.uid.clone(), self.dna_path.clone()).await?;
       msg!("    Using DNA: {}", dna_hash);
      let conductor = start_conductor(sid_str.clone()).await;
      let hash = snapmail_set_handle(conductor, sid_str.clone())?;
      msg!(" handle set: {} - {:?}", sid_str, hash);
      Ok(())
   }
}

#[derive(Debug, StructOpt, Clone)]
pub enum NetworkCmd {
   /// Add an optional network config
   Network(Network),
}

impl NetworkCmd {
   pub fn into_inner(self) -> Network {
      match self {
         NetworkCmd::Network(n) => n,
      }
   }
}

#[derive(Debug, StructOpt, Clone)]
pub struct Network {
   #[structopt(subcommand)]
   /// Set the type of network.
   pub transport_type: TransportType,
   #[structopt(short, long, parse(from_str = Url2::parse))]
   /// Optionally set a bootstrap service URL.
   /// A bootstrap service can used for peers to discover each other without
   /// prior knowledge of each other.
   pub bootstrap: Option<Url2>,
}

#[derive(Debug, StructOpt, Clone)]
pub enum TransportType {
   /// A transport that uses MDNS
   Mdns,
   /// A transport that uses the QUIC protocol.
   Quic(Quic),
}

#[derive(Debug, StructOpt, Clone)]
pub struct Quic {
   #[structopt(short, long, parse(from_str = Url2::parse))]
   /// To which network interface / port should we bind?
   /// Default: "kitsune-quic://0.0.0.0:0".
   pub bind_to: Option<Url2>,
   #[structopt(long)]
   /// If you have port-forwarding set up,
   /// or wish to apply a vanity domain name,
   /// you may need to override the local NIC ip.
   /// Default: None = use NIC ip.
   pub override_host: Option<String>,
   #[structopt(long)]
   /// If you have port-forwarding set up,
   /// you may need to override the local NIC port.
   /// Default: None = use NIC port.
   pub override_port: Option<u16>,
   #[structopt(short, parse(from_str = Url2::parse))]
   /// Run through an external proxy at this url.
   pub proxy: Option<Url2>,
}

impl From<Network> for KitsuneP2pConfig {
   fn from(n: Network) -> Self {
      let Network {
         transport_type: transport,
         bootstrap,
      } = n;
      let mut kit = KitsuneP2pConfig::default();
      kit.bootstrap_service = bootstrap;

      match transport {
         TransportType::Mdns => {
            kit.network_type = NetworkType::QuicMdns;
            kit.transport_pool = vec![TransportConfig::Quic {
               bind_to: None,
               override_host: None,
               override_port: None,
            }];
         },
         TransportType::Quic(Quic {
                              bind_to,
                              override_host,
                              override_port,
                              proxy: None,
                           }) => {
            kit.transport_pool = vec![TransportConfig::Quic {
               bind_to,
               override_host,
               override_port,
            }];
         }
         TransportType::Quic(Quic {
                              bind_to,
                              override_host,
                              override_port,
                              proxy: Some(proxy_url),
                           }) => {
            let transport = TransportConfig::Quic {
               bind_to,
               override_host,
               override_port,
            };
            kit.transport_pool = vec![TransportConfig::Proxy {
               sub_transport: Box::new(transport),
               proxy_config: holochain_p2p::kitsune_p2p::ProxyConfig::RemoteProxyClient {
                  proxy_url,
               },
            }]
         }
      }
      kit
   }
}
