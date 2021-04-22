use crate::globals::*;
use crate::holochain::*;
use holochain::conductor::*;
use holochain_types::dna::*;
use holochain_types::dna::zome::*;
use holochain_types::dna::wasm::DnaWasm;
use holochain_types::app::*;
use holochain_zome_types::*;
use holochain_conductor_api::*;
use holochain::conductor::error::*;
use std::path::Path;
//use holo_hash::*;
//use holochain_serialized_bytes::prelude::*;
use holochain_keystore::keystore_actor::KeystoreSenderExt;

///
pub async fn call_zome(conductor: ConductorHandle, fn_name: &str, payload: ExternIO) /*-> ConductorResult<()>*/ {
   let cell_ids = conductor.list_cell_ids().await.unwrap();
   println!("Cell IDs : {:?}", cell_ids);

   assert!(!cell_ids.is_empty());
   let cell_id = cell_ids[0].clone();
   let provenance = cell_ids[0].agent_pubkey().to_owned();

   let result = conductor.call_zome(ZomeCall {
                          cap: None,
                          cell_id,
                          zome_name: ZOME_NAME.into(),
                          fn_name: fn_name.into(),
                          provenance,
                          payload,
                       }).await.unwrap();
   println!("ZomeCall result: {:?}", result);
}


///
pub async fn start_conductor(uid: String) -> ConductorHandle {
   println!("** start_conductor: {:?}", uid);
   // Load conductor from config file
   let config_path = Path::new(&*CONFIG_PATH).join(uid.clone()).join(CONDUCTOR_CONFIG_FILENAME);
   // let config_path = CONDUCTOR_CONFIG_FILEPATH.to_path_buf();
   let conductor = conductor_handle_from_config_path(Some(config_path)).await;
   conductor.print_setup();

   // Startup
   conductor
      .clone()
      .startup_app_interfaces().await.unwrap();

   let dnas = conductor.list_dnas().await.unwrap();
   println!("Installed DNAs: {:?}", dnas);

   if (dnas.is_empty()) {
      install_app(conductor.clone(), uid.clone()).await.unwrap();
      let dnas = conductor.list_dnas().await.unwrap();
      println!("Installed DNAs: {:?}", dnas);
   }

   let apps = conductor.list_active_apps().await.unwrap();
   println!("Activate Apps: {:?}", apps);

   let interfaces = conductor.list_app_interfaces().await.unwrap();
   println!("App Interfaces: {:?}", interfaces);

   let cell_ids = conductor.list_cell_ids().await.unwrap();
   println!("Cell IDs: {:?}", cell_ids);
   assert!(!cell_ids.is_empty());
   //g_cell_id = cell_ids[0];


   // Done
   return conductor;
}


/// Install Snapmail DNA form dna file
/// FIXME: hardcoded DNA file path
pub async fn install_app(conductor: ConductorHandle, uid: String) -> ConductorResult<()> {
   // Generate keys
   let agent_key = conductor
      .keystore()
      .generate_sign_keypair_from_pure_entropy()
      .await?;

   // Load DnaFile
   println!("Loading DNA wasm file: {}", WASM_PATH);
   let wasm = &std::fs::read(WASM_PATH)?;
   let dna_wasm = DnaWasm::from(wasm.to_owned());
   let (_, wasm_hash) = holochain_types::dna::wasm::DnaWasmHashed::from_content(dna_wasm.clone())
      .await
      .into_inner();
   let zome_def: ZomeDef = WasmZome { wasm_hash }.into();
   let zome = (ZOME_NAME.into(), zome_def).into();

   let dna_file = DnaFile::new(DnaDef {
      name: SNAPMAIL_APP_ID.to_string(),
      uid: uid.to_string(),
      properties: SerializedBytes::try_from(()).unwrap(),
      zomes: vec![zome].into(),
   },
      vec![dna_wasm].into_iter(),
   ).await.expect("Dna file load failed");

   // Register DNA
   conductor.register_dna(dna_file.clone()).await?;

   // Install DNA
   let cell_id = CellId::from((dna_file.dna_hash().clone(), agent_key.clone()));
   let cell_id_with_proof =  (InstalledCell::new(cell_id, "slot-1".to_string()), None);

   // Call genesis
   conductor
       .clone()
       .install_app(SNAPMAIL_APP_ID.to_string(), vec![cell_id_with_proof])
       .await?;

   // Activate app
   conductor.activate_app(SNAPMAIL_APP_ID.to_string()).await?;

   // Done
   Ok(())
}
