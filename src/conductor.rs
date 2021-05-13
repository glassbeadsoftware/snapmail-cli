use crate::globals::*;
use crate::holochain::*;
use snapmail::ZOME_NAME;
use holochain::conductor::ConductorHandle;
use holochain_types::dna::*;
use holochain_types::dna::zome::*;
use holochain_types::dna::wasm::DnaWasm;
use holochain_types::app::*;
use holochain_zome_types::*;
use holochain::conductor::{
   error::*,
   p2p_store,
};
use std::path::Path;
use holochain_keystore::keystore_actor::KeystoreSenderExt;


///
pub async fn start_conductor(sid: String) -> ConductorHandle {
   msg!("** start_conductor: {:?}", sid);
   /// Load conductor from config file
   let config_path = Path::new(&*CONFIG_PATH).join(sid.clone()).join(CONDUCTOR_CONFIG_FILENAME);
   let conductor = conductor_handle_from_config_path(Some(config_path)).await;
   /// Check state
   //let _ = conductor.print_setup();
   //let dnas = conductor.list_dnas().await.unwrap();
   //msg!("Installed DNAs: {:?}", dnas);
   //let apps = conductor.list_active_apps().await.unwrap();
   //msg!("Activate Apps: {:?}", apps);
   //let _interfaces = conductor.list_app_interfaces().await.unwrap();
   //msg!("App Interfaces: {:?}", interfaces);
   //let cell_ids = conductor.list_cell_ids().await.unwrap();
   //println!("Cell IDs: {:?}", cell_ids);
   /// Done
   return conductor;
}


/// Install Snapmail DNA from dna file
/// FIXME: hardcoded DNA file path
#[allow(deprecated)]
pub async fn install_app(sid: String, uid: String) -> ConductorResult<()> {
   /// Load conductor from config file
   let config_path = Path::new(&*CONFIG_PATH).join(sid.clone());
   let conductor_path = config_path.join(CONDUCTOR_CONFIG_FILENAME);
   let app_filepath = config_path.join(APP_CONFIG_FILENAME);
   std::fs::write(app_filepath, uid.as_bytes())?;
   let conductor = conductor_handle_from_config_path(Some(conductor_path)).await;
   /// Generate keys
   let agent_key = conductor
      .keystore()
      .generate_sign_keypair_from_pure_entropy()
      .await?;

   /// Load DnaFile
   // println!("Loading DNA wasm file: {}", WASM_PATH);
   // let wasm = &std::fs::read(WASM_PATH)?;
   // let wasm_str = format!("pub const DNA_WASM: [u8 ; {}] = {:?};\n", wasm.len(), wasm);
   // std::fs::write("./wasm.rs", wasm_str.as_bytes());
   // //let wasm2 = &std::fs::read("./dump.rs")?;
   // let dna_wasm = DnaWasm::from(wasm.to_owned());

   println!("Loading DNA wasm vec!");
   //let compressed = crate::wasm::DNA_WASM.to_vec();
   let compressed = base64::decode_config(crate::wasm::DNA_WASM_B64, base64::URL_SAFE_NO_PAD).unwrap();
   let (decompressed, _checksum) = yazi::decompress(&compressed, yazi::Format::Zlib).unwrap();

   let dna_wasm = DnaWasm::from(decompressed);

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
   /// Register DNA
   conductor.register_dna(dna_file.clone()).await?;
   /// Install DNA
   let cell_id = CellId::from((dna_file.dna_hash().clone(), agent_key.clone()));
   let cell_id_with_proof =  (InstalledCell::new(cell_id, "slot-1".to_string()), None);
   /// Call genesis
   conductor
       .clone()
       .install_app(SNAPMAIL_APP_ID.to_string(), vec![cell_id_with_proof])
       .await?;
   /// Activate app
   conductor.activate_app(SNAPMAIL_APP_ID.to_string()).await?;
   /// Done
   let dnas = conductor.list_dnas().await.unwrap();
   msg!("Installed DNAs: {:?}", dnas);
   Ok(())
}

///
pub fn dump_state(conductor: ConductorHandle) -> usize {
   let result = tokio_helper::block_on(async {
      //let p2p = conductor.holochain_p2p();
      //let broadcaster = conductor.signal_broadcaster();

      let cell_id = &conductor.list_cell_ids().await.unwrap()[0];

      // let cell = conductor.cell_by_id(cell_id).unwrap();
      // let arc = cell.env();
      // let source_chain = SourceChainBuf::new(arc.clone().into()).unwrap();
      // let source_chain_dump = source_chain.dump_state().await.unwrap();
      //let integration_dump = integrate_dht_ops_workflow::dump_state(arc.clone().into())?;

      let peer_dump = p2p_store::dump_state(
         conductor.get_p2p_env().await.clone().into(),
         Some(cell_id.clone()),
      ).unwrap();

      //let state = conductor.dump_cell_state(&cell_ids[0]).await.unwrap();
      //msg!(" {}", state);

      // msg!("Conductor state dump:");
      // msg!(" - peer dump: {}", peer_dump);
      // msg!(" - Peers: {}", peer_dump.peers.len());
      peer_dump.peers.len()
   }, std::time::Duration::from_secs(9));
   result.unwrap()
}