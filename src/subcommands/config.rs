//! Helpers for creating, reading and writing [`ConductorConfig`]s.
use std::path::PathBuf;
use url2::Url2;
use holochain_conductor_api::config::conductor::ConductorConfig;
use crate::globals::*;
use std::fs::File;
use std::io::prelude::*;
use std::fs;
use holochain_p2p::kitsune_p2p::KitsuneP2pConfig;
use holochain_p2p::kitsune_p2p::TransportConfig;

/// Name of the file that conductor config is written to.
pub const CONDUCTOR_CONFIG: &str = "conductor-config.yaml";


///
pub fn generateConductorConfig(
    config_file_path: PathBuf,
    maybe_boostrapUrl: Option<Url2>,
    storage_path: PathBuf,
    maybe_proxyUrl: Option<Url2>,
    admin_port: u32,
    canMdns: bool,
) -> Result<(), ()> {
    msg!("generateConductorConfig() with {}", admin_port);

    // -- Prepare variables
    let proxy_url = match maybe_proxyUrl {
        Some(url) => url.to_string(),
        None => (*DEFAULT_PROXY_URL).to_string(),
    };
    let bootstrap_url = match maybe_boostrapUrl {
        Some(url) => url.to_string(),
        None => (*DEFAULT_BOOTSTRAP_URL).to_string(),
    };
    let network_type = if canMdns { "quic_mdns" } else { "quic_bootstrap" };
    //let environment_path = wslPath(storage_path);
    let environment_path = storage_path.into_os_string().into_string().expect("Invalid os string");
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
   port: {admin_port}
   network:
   network_type: {network_type}
   bootstrap_service: {bootstrap_url}
   transport_pool:
   - type: proxy
   sub_transport:
   type: quic
   bind_to: kitsune-quic://0.0.0.0:0
   proxy_config:
   type: remote_proxy_client
   proxy_url: {proxy_url}",
               environment_path = environment_path, proxy_url = proxy_url, admin_port = admin_port,
               network_type = network_type, bootstrap_url = bootstrap_url,
       );

    // -- Write on disk
    msg!("Writing config file: {:?}", config_file_path.as_os_str());
    let mut file = File::create(config_file_path).expect("Config file created failed");
    file.write_all(config.into_bytes().as_slice()).expect("Config file write failed");
    Ok(())
}

/// Create a new default [`ConductorConfig`] with environment path
/// and keystore all in the same directory.
pub fn create_config(environment_path: PathBuf) -> ConductorConfig {
    let mut conductor_config = ConductorConfig::default();
    conductor_config.environment_path = environment_path.clone().into();
    let mut keystore_path = environment_path;
    keystore_path.push("keystore");
    conductor_config.keystore_path = Some(keystore_path);
    conductor_config
}

/// Write [`ConductorConfig`] to [`CONDUCTOR_CONFIG`]
pub fn write_config(mut path: PathBuf, config: &ConductorConfig) -> PathBuf {
    path.push(CONDUCTOR_CONFIG);
    std::fs::write(path.clone(), serde_yaml::to_string(&config).unwrap()).unwrap();
    path
}

/// Read the [`ConductorConfig`] from the file [`CONDUCTOR_CONFIG`] in the provided path.
pub fn read_config(mut path: PathBuf) -> anyhow::Result<Option<ConductorConfig>> {
    path.push(CONDUCTOR_CONFIG);

    match std::fs::read_to_string(path) {
        Ok(yaml) => Ok(Some(serde_yaml::from_str(&yaml)?)),
        Err(_) => Ok(None),
    }
}


// /// Generate a new sandbox.
// /// This creates a directory and a [`ConductorConfig`]
// /// from an optional network.
// /// The root directory and inner directory
// /// (where this sandbox will be created) can be overridden.
// /// For example `my_root_dir/this_sandbox_dir/`
// pub fn generate(
//     network: Option<KitsuneP2pConfig>,
//     root: PathBuf,
//     directory: Option<PathBuf>,
// ) -> anyhow::Result<PathBuf> {
//     let dir = generate_directory(root, directory)?;
//     let mut config = create_config(dir.clone());
//     config.network = network;
//     random_admin_port(&mut config);
//     let path = write_config(dir.clone(), &config);
//     msg!("Config {:?}", config);
//     msg!(
//         "Created directory at: {} {}",
//         ansi_term::Style::new()
//             .bold()
//             .underline()
//             .on(ansi_term::Color::Fixed(254))
//             .fg(ansi_term::Color::Fixed(4))
//             .paint(dir.display().to_string()),
//         ansi_term::Style::new()
//             .bold()
//             .paint("Keep this path to rerun the same sandbox")
//     );
//     msg!("Created config at {}", path.display());
//     Ok(dir)
// }

// /// Generate a new sandbox from a full config.
// pub fn generate_with_config(
//     config: Option<ConductorConfig>,
//     root: PathBuf,
//     directory: Option<PathBuf>,
// ) -> anyhow::Result<PathBuf> {
//     let dir = generate_directory(root, directory)?;
//     let config = config.unwrap_or_else(|| create_config(dir.clone()));
//     write_config(dir.clone(), &config);
//     Ok(dir)
// }

/// Generate a new directory structure for a sandbox.
pub fn generate_directory(
    root: PathBuf,
    directory: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    let mut dir = root;
    let directory = directory.unwrap_or_else(|| nanoid::nanoid!().into());
    dir.push(directory);
    std::fs::create_dir(&dir)?;
    let mut keystore_dir = dir.clone();
    keystore_dir.push("keystore");
    std::fs::create_dir(keystore_dir)?;
    Ok(dir)
}
