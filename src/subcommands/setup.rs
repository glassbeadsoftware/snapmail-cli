use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use derive_more::FromStr;
use std::path::Path;
use std::path::PathBuf;
use holochain_p2p::kitsune_p2p::KitsuneP2pConfig;
use holochain_p2p::kitsune_p2p::TransportConfig;
use url2::Url2;
use std::fs::File;
use std::io::prelude::*;
use std::fs;
use directories::ProjectDirs;
use crate::globals::*;
use holochain_conductor_api::config::conductor::ConductorConfig;
use crate::subcommands::config::*;


#[derive(Debug, StructOpt, Clone)]
/// This creates a new holochain sandbox
/// which is a
/// - conductor config
/// - databases
/// - keystore
pub struct SetupCommand {
   #[structopt(long)]
   handle: String,
   #[structopt(parse(from_os_str))]
   pub uid: PathBuf,
   // #[structopt(name = "bootstrap", parse(from_str = Url2::parse))]
   // maybe_bootstrap: Option<Url2>,
   // #[structopt(name = "proxy", parse(from_str = Url2::parse))]
   // maybe_proxy: Option<Url2>,
   // #[structopt(name = "mdns")]
   // maybe_can_mdns: Option<bool>,
   #[structopt(subcommand, name = "network")]
   /// Add an optional network config
   pub maybe_network: Option<NetworkCmd>,
   /// Set a root directory for the app's storage data to be placed into.
   /// Defaults to the system's temp directory.
   /// This directory must already exist.
   #[structopt(name = "root", parse(from_os_str))]
   pub maybe_root: Option<PathBuf>,
}

impl SetupCommand {
   ///
   pub fn run(self) {
      let root = self.maybe_root.unwrap_or(CONFIG_PATH.as_path().to_path_buf());
      let _ = generate(
         root,
         Some(self.uid),
         self.maybe_network.map(|n| n.into_inner().into()),
      ).expect("Generate config failed. Maybe Invalid params.");
   }
}

#[derive(Debug, StructOpt, Clone)]
pub enum NetworkCmd {
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
   pub transport: NetworkType,
   #[structopt(short, long, parse(from_str = Url2::parse))]
   /// Optionally set a bootstrap service URL.
   /// A bootstrap service can used for peers to discover each other without
   /// prior knowledge of each other.
   pub bootstrap: Option<Url2>,
}

#[derive(Debug, StructOpt, Clone)]
pub enum NetworkType {
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
   #[structopt(short, long)]
   /// If you have port-forwarding set up,
   /// or wish to apply a vanity domain name,
   /// you may need to override the local NIC ip.
   /// Default: None = use NIC ip.
   pub override_host: Option<String>,
   #[structopt(short, long)]
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
         transport,
         bootstrap,
      } = n;
      let mut kit = KitsuneP2pConfig::default();
      kit.bootstrap_service = bootstrap;

      match transport {
         NetworkType::Mdns => (),
         NetworkType::Quic(Quic {
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
         NetworkType::Quic(Quic {
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
