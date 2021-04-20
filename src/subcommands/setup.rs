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

///
fn generateConductorConfig(
   config_path: PathBuf,
   maybe_boostrapUrl: Option<Url2>,
   storage_path: PathBuf,
   maybe_proxyUrl: Option<Url2>,
   admin_port: u32,
   canMdns: bool,
) -> Result<()> {
   println!("generateConductorConfig() with " + admin_port);

   // -- Prepare variables
   let proxy_url = match maybe_proxyUrl {
      Some(url) => url,
      None => DEFAULT_PROXY_URL,
   };
   let bootstrap_url = match maybe_boostrapUrl {
      Some(url) => url,
      None => DEFAULT_BOOTSTRAP_URL,
   };
   //proxyUrl = DEFAULT_PROXY_URL;
   let network_type = if (canMdns) { "quic_mdns" } else {"quic_bootstrap"};
   //let environment_path = wslPath(storage_path);
   let environment_path = storagePath;
   //println!("environment_path = {}", environment_path);

   // -- Form content
   let config =
   format!("environment_path: {environment_path}
   use_dangerous_test_keystore: false
   passphrase_service:
   type: cmd
   admin_interfaces:
   - driver:
   type: websocket
   port: ${admin_port}
   network:
   network_type: ${network_type}
   bootstrap_service: {boostrap_url}
   transport_pool:
   - type: proxy
   sub_transport:
   type: quic
   bind_to: kitsune-quic://0.0.0.0:0
   proxy_config:
   type: remote_proxy_client
   proxy_url: {proxy_url}",
           environment_path = environment_path, proxy_url = proxy_url, admin_port = admin_port,
           network_type = network_type, boostrap_url = boostrap_url,
   );

   // -- Write on disk
   let mut file = File::create(config_path)?;
   file.write_all(config.into_bytes().as_slice())?;
   Ok(())
}


#[derive(Debug, StructOpt, Clone)]
/// This creates a new holochain sandbox
/// which is a
/// - conductor config
/// - databases
/// - keystore
pub struct Setup {
   #[structopt(subcommand)]
   /// Add an optional network config
   pub network: Option<NetworkCmd>,
   /// Set a root directory for the app's storage data to be placed into.
   /// Defaults to the system's temp directory.
   /// This directory must already exist.
   #[structopt(long)]
   pub root: Option<PathBuf>,
   #[structopt(short, long)]
   /// Specify the directory name for each user that is created.
   /// By default, new user directories get a random name
   /// like "kAOXQlilEtJKlTM_W403b".
   /// Use this option to override those names with something explicit.
   pub directory: PathBuf,
}

///
pub fn setup(handle: String) {

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

#[derive(Debug, StructOpt, Clone)]
pub struct Existing {
   #[structopt(short, long, value_delimiter = ",")]
   /// Paths to existing sandbox directories.
   /// For example `hc run -e=/tmp/kAOXQlilEtJKlTM_W403b,/tmp/kddsajkaasiIII_sJ`.
   pub existing_paths: Vec<PathBuf>,
   #[structopt(short, long, conflicts_with_all = &["last", "indices"])]
   /// Run all the existing conductor sandboxes.
   pub all: bool,
   #[structopt(short, long, conflicts_with_all = &["all", "indices"])]
   /// Run the last created conductor sandbox.
   pub last: bool,
   /// Run a selection of existing conductor sandboxes.
   /// Existing sandboxes are visible via `hc list`.
   /// Use the index to choose which sandboxes to use.
   /// For example `hc run 1 3 5` or `hc run 1`
   #[structopt(conflicts_with_all = &["all", "last"])]
   pub indices: Vec<usize>,
}

impl Existing {
   pub fn load(mut self) -> anyhow::Result<Vec<PathBuf>> {
      let sandboxes = crate::save::load(std::env::current_dir()?)?;
      if self.all {
         // Get all the sandboxes
         self.existing_paths.extend(sandboxes.into_iter())
      } else if self.last && sandboxes.last().is_some() {
         // Get just the last sandbox
         self.existing_paths
             .push(sandboxes.last().cloned().expect("Safe due to check above"));
      } else if !self.indices.is_empty() {
         // Get the indices
         let e = self
            .indices
            .into_iter()
            .filter_map(|i| sandboxes.get(i).cloned());
         self.existing_paths.extend(e);
      } else if !self.existing_paths.is_empty() {
         // If there is existing paths then use those
      } else if sandboxes.len() == 1 {
         // If there is only one sandbox then use that
         self.existing_paths
             .push(sandboxes.last().cloned().expect("Safe due to check above"));
      } else if sandboxes.len() > 1 {
         // There is multiple sandboxes, the use must disambiguate
         msg!(
                "
There are multiple sandboxes and hc doesn't know which to run.
You can run:
    - `--all` `-a` run all sandboxes.
    - `--last` `-l` run the last created sandbox.
    - `--existing-paths` `-e` run a list of existing paths to sandboxes.
    - `1` run a index from the list below.
    - `0 2` run multiple indices from the list below.
Run `hc list` to see the sandboxes or `hc r --help` for more information."
            );
         crate::save::list(std::env::current_dir()?, 0)?;
      } else {
         // There is no sandboxes
         msg!(
                "
Before running or calling you need to generate a sandbox.
You can use `hc generate` or `hc g` to do this.
Run `hc g --help` for more options."
            );
      }
      Ok(self.existing_paths)
   }

   pub fn is_empty(&self) -> bool {
      self.existing_paths.is_empty() && self.indices.is_empty() && !self.all && !self.last
   }
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

impl Default for Setup {
   fn default() -> Self {
      Self {
         network: None,
         root: None,
         directory: PathBuf::default(),
      }
   }
}
