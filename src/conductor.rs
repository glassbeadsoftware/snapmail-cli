use crate::globals::*;
use crate::holochain::*;
use holochain::conductor::{Conductor, ConductorBuilder, ConductorHandle};

//
pub async fn start_conductor(_handle: String) -> ConductorHandle {
   let config_path = CONDUCTOR_CONFIG_FILEPATH.to_path_buf();
   let conductor = conductor_handle_from_config_path(Some(config_path)).await;
   conductor.print_setup();
   return conductor;
}
