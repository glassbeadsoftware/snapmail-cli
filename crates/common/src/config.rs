//! Helpers for creating, reading and writing [`ConductorConfig`]s.
use std::path::PathBuf;
use holochain_conductor_api::config::conductor::ConductorConfig;
use holochain_conductor_api::config::*;
use crate::globals::*;
use holochain_p2p::kitsune_p2p::KitsuneP2pConfig;

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
    path.push(CONDUCTOR_CONFIG_FILENAME);
    std::fs::write(path.clone(), serde_yaml::to_string(&config).unwrap())
       .expect("CONDUCTOR_CONFIG_FILENAME should be writable");
    path
}

/// Read the [`ConductorConfig`] from the file [`CONDUCTOR_CONFIG`] in the provided path.
pub fn read_config(mut path: PathBuf) -> anyhow::Result<Option<ConductorConfig>> {
    path.push(CONDUCTOR_CONFIG_FILENAME);

    match std::fs::read_to_string(path) {
        Ok(yaml) => Ok(Some(serde_yaml::from_str(&yaml)?)),
        Err(_) => Ok(None),
    }
}

/// Generate a new setup for the holochain conductor.
/// This creates a directory and a [`ConductorConfig`]
/// from an optional network config, root directory and sub directory
/// For example `my_root_dir/this_sub_dir/`
pub fn generate(
    root: PathBuf,
    maybe_directory: Option<PathBuf>,
    maybe_network: Option<KitsuneP2pConfig>,
) -> anyhow::Result<PathBuf> {
    let dir = generate_directory(root, maybe_directory)?;
    let mut config = create_config(dir.clone());
    config.network = maybe_network;
    config.admin_interfaces = Some(vec![AdminInterfaceConfig {
        driver: InterfaceDriver::Websocket { port: 0 },
    }]);

    let path = write_config(dir.clone(), &config);
    //msg!("Config {:?}", config);
    msg!(
        "Created directory at: {} {}",
        ansi_term::Style::new()
            .bold()
            .underline()
            .on(ansi_term::Color::Fixed(254))
            .fg(ansi_term::Color::Fixed(4))
            .paint(dir.display().to_string()),
        ansi_term::Style::new()
            .bold()
            .paint("Keep this path to rerun the same sandbox")
    );
    msg!("Created config at {}", path.display());
    Ok(dir)
}

/// Generate a new sandbox from a full config.
pub fn generate_with_config(
    maybe_config: Option<ConductorConfig>,
    root: PathBuf,
    maybe_directory: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    let dir = generate_directory(root, maybe_directory)?;
    let config = maybe_config.unwrap_or_else(|| create_config(dir.clone()));
    write_config(dir.clone(), &config);
    Ok(dir)
}

/// Generate a new directory structure for a holochain conductor.
pub fn generate_directory(
    root: PathBuf,
    maybe_directory: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    let mut dir = root;
    if let Some(directory) = maybe_directory {
        //let directory = maybe_directory.unwrap_or_else(|| nanoid::nanoid!().into());
        dir.push(directory);
    }
    msg!("Creating dir: {:?}", dir);
    std::fs::create_dir_all(&dir)?;
    let mut keystore_dir = dir.clone();
    keystore_dir.push("keystore");
    msg!("Creating keystore_dir: {:?}", keystore_dir);
    std::fs::create_dir(keystore_dir)?;
    Ok(dir)
}
