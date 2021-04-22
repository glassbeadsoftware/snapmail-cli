//use crate::globals::*;
use holochain::conductor::{Conductor, ConductorBuilder, ConductorHandle};
use std::path::PathBuf;
use holochain::conductor::paths::ConfigFilePath;
use holochain::conductor::config::ConductorConfig;
use holochain_conductor_api::conductor::ConductorConfigError;

// -- Copied from holochain/main.rs -- //

const ERROR_CODE: i32 = 44;

pub async fn conductor_handle_from_config_path(config_path: Option<PathBuf>) -> ConductorHandle {
   let config_path_default = config_path.is_none();
   let config_path: ConfigFilePath = config_path.map(Into::into).unwrap_or_default();
   println!("config_path: {:?}", config_path);

   let config: ConductorConfig = load_config(&config_path, config_path_default);

   // Check if LMDB env dir is present
   // In interactive mode give the user a chance to create it, otherwise create it automatically
   let env_path = PathBuf::from(config.environment_path.clone());
   if !env_path.is_dir() {
      let result = std::fs::create_dir_all(&env_path);
      match result {
         Ok(()) => println!("Created LMDB environment at {}.", env_path.display()),
         Err(e) => {
            println!("Couldn't create LMDB environment: {}", e);
            std::process::exit(ERROR_CODE);
         }
      }
   }

   // Initialize the Conductor
   Conductor::builder()
      .config(config)
      .build()
      .await
      .expect("Could not initialize Conductor from configuration")
}

/// Load config, throw friendly error on failure
fn load_config(config_path: &ConfigFilePath, config_path_default: bool) -> ConductorConfig {
   match ConductorConfig::load_yaml(config_path.as_ref()) {
      Err(ConductorConfigError::ConfigMissing(_)) => {
         display_friendly_missing_config_message(config_path, config_path_default);
         std::process::exit(ERROR_CODE);
      }
      Err(ConductorConfigError::SerializationError(err)) => {
         display_friendly_malformed_config_message(config_path, err);
         std::process::exit(ERROR_CODE);
      }
      result => result.expect("Could not load conductor config"),
   }
}


fn display_friendly_missing_config_message(
   config_path: &ConfigFilePath,
   config_path_default: bool,
) {
   if config_path_default {
      println!(
         "
Error: The conductor is set up to load its configuration from the default path:

    {path}

but this file doesn't exist. If you meant to specify a path, run this command
again with the -c option. Otherwise, please either create a YAML config file at
this path yourself.
        ",
         path = config_path,
      );
   } else {
      println!(
         "
Error: You asked to load configuration from the path:

    {path}

but this file doesn't exist. Please either create a YAML config file at this
path yourself.
        ",
         path = config_path,
      );
   }
}

fn display_friendly_malformed_config_message(
   config_path: &ConfigFilePath,
   error: serde_yaml::Error,
) {
   println!(
      "
The specified config file ({})
could not be parsed, because it is not valid YAML. Please check and fix the
file. Details:

    {}

    ",
      config_path, error
   )
}
