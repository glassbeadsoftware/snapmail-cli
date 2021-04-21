use crate::globals::*;
use crate::holochain::*;
use holochain::conductor::*;
use holochain_types::dna::*;
use holochain::conductor::error::*;
//use holo_hash::*;
pub use holochain_serialized_bytes::prelude::*;

//
pub async fn start_conductor(_handle: String) -> ConductorHandle {
   let config_path = CONDUCTOR_CONFIG_FILEPATH.to_path_buf();
   let mut conductor = conductor_handle_from_config_path(Some(config_path)).await;
   conductor.print_setup();

   conductor.clone().startup_app_interfaces().await.unwrap();


   return conductor;
}

pub async fn install_app(conductor: ConductorHandle, uid: String) -> ConductorResult<()> {
   // let agent_pub_key = conductor
   //    .keystore()
   //    .clone()
   //    .generate_sign_keypair_from_pure_entropy()
   //    .await?;

   let dna_file = DnaFile::new(
      DnaDef {
         name: SNAPMAIL_APP_ID.to_string(),
         uid,
         properties: SerializedBytes::try_from(()).unwrap(),
         zomes: vec![/*TestWasm::Create.into()*/].into(),
      },
      vec![/*TestWasm::Create.into()*/],
   )
      .await
      .unwrap();


   // {
   //    uid,
   //    properties: undefined,
   //    path: './dna/snapmail.dna',
   // }

   let hash = conductor.register_dna(dna_file).await?;

   // let cell_id = CellId::from((hash.clone(), agent_key.clone()));
   // ConductorApiResult::Ok((InstalledCell::new(cell_id, nick), membrane_proof))
   // // Call genesis
   // conductor_handle
   //     .clone()
   //     .install_app(installed_app_id.clone(), cell_ids_with_proofs.clone())
   //     .await?;

   Ok(())
}
