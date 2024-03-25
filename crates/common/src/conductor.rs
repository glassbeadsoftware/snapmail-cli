use crate::{
   globals::*,
   //config::*,
};
use crate::holochain::*;
use snapmail::ZOME_NAME;
use holochain::conductor::ConductorHandle;
use holochain_types::dna::*;
use holochain_types::dna::wasm::DnaWasm;
use holochain_types::app::*;
use holochain_zome_types::*;
use holochain_p2p::*;
use holochain::conductor::{
   error::*,
   p2p_agent_store,
};
use std::path::Path;
use std::path::PathBuf;
//use holochain_keystore::keystore_actor::KeystoreSenderExt;
use holochain::conductor::config::ConductorConfig;

///
pub async fn start_conductor_or_abort(sid: String) -> (ConductorHandle, DnaHash) {
   /// Make sure config exists
   let config_path = Path::new(&*CONFIG_PATH).join(sid.clone()).join(CONDUCTOR_CONFIG_FILENAME);
   if let Err(_e) = ConductorConfig::load_yaml(config_path.as_ref()) {
      err_msg!("Failed to load config for session \"{}\"", sid);
      err_msg!("Make sure it has been setup with snapmail-cli");
      std::process::abort();
   }
   /// Load conductor from config file
   let conductor = conductor_handle_from_config_path(Some(config_path)).await;
   /// Make sure it has the correct DNA
   /// - Get UID
   let path = CONFIG_PATH.as_path().join(sid.clone());
   let app_filepath = path.join(APP_CONFIG_FILENAME);
   let uid = std::fs::read_to_string(app_filepath).expect("Should have config folder");
   let expected_dna = load_dna_from_rs(uid).await;
   let expected_wasm = expected_dna.get_wasm_for_zome(&ZomeName::from("snapmail")).unwrap();
   let expected_wasm_hash = holo_hash::WasmHash::with_data(expected_wasm).await;

   /// - Get Installed DNAs
   let dnas = conductor.list_dnas();
   /// - Check
   if dnas.len() != 1 {
      err_msg!("No installed DNA found ({})", dnas.len());
      err_msg!("Make sure it has been setup with snapmail-cli");
      std::process::abort();
   }
   let dna = conductor.get_dna(&dnas[0]).unwrap();
   let expected_hash = dna.dna_hash().clone();
   let maybe_wasm = dna.get_wasm_for_zome(&ZomeName::from("snapmail"));
   if maybe_wasm.is_err() {
      err_msg!("Installed DNA Mismatch:");
      err_msg!(" - \"snapmail\" zome not found");
      err_msg!("Make sure it has been setup with snapmail-cli");
      std::process::abort();
   }
   let wasm_hash = holo_hash::WasmHash::with_data(maybe_wasm.unwrap()).await;
   if wasm_hash != expected_wasm_hash {
      err_msg!("Installed DNA Mismatch:");
      err_msg!(" - Installed Wasm: {}\n  - Expected Wasm: {}", wasm_hash, expected_wasm_hash);
      err_msg!("Make sure it has been setup with snapmail-cli");
      std::process::abort();
   }
   /// Done
   return (conductor, expected_hash);
}



///
pub async fn start_conductor(sid: String) -> ConductorHandle {
   msg!("** start_conductor: {:?}", sid);
   /// Load conductor from config file
   let config_path = Path::new(&*CONFIG_PATH).join(sid.clone()).join(CONDUCTOR_CONFIG_FILENAME);
   let conductor = conductor_handle_from_config_path(Some(config_path)).await;
   /// Check state
   //let _ = conductor.print_setup();
   let dnas = conductor.list_dnas();
   msg!("Installed DNAs: {:?}", dnas);
   //let apps = conductor.list_active_apps();
   //msg!("Activate Apps: {:?}", apps);
   //let _interfaces = conductor.list_app_interfaces().await.unwrap();
   //msg!("App Interfaces: {:?}", interfaces);
   let cell_ids = conductor.list_cell_ids(None);
   println!("Cell IDs: {:?}", cell_ids);
   /// Done
   return conductor;
}

/// Create a DnaFile from a path to a *.dna bundle
async fn load_dna_from_path(ns: NetworkSeed, path: &Path) -> holochain_types::dna::error::DnaResult<DnaFile> {
   let mut dna = DnaBundle::read_from_file(path)
      .await?
      .into_dna_file(None, None)
      .await?
      .0;
   dna = dna.with_network_seed(ns).await?;
   Ok(dna)
}


// ///
// async fn load_dna_from_rs(ns: NetworkSeed) -> DnaFile {
//    let compressed = base64::decode_config(crate::wasm::DNA_WASM_B64, base64::URL_SAFE_NO_PAD).unwrap();
//    let (decompressed, _checksum) = yazi::decompress(&compressed, yazi::Format::Zlib).unwrap();
//
//    let dna_wasm = DnaWasm::from(decompressed);
//    let (_, wasm_hash) = holochain_types::dna::wasm::DnaWasmHashed::from_content(dna_wasm.clone())
//    .await
//    .into_inner();
//    let zome_def: ZomeDef = ZomeDef::from_hash(wasm_hash.clone());
//    let zome = (ZOME_NAME.into(), zome_def).into();
//    //let name = SNAPMAIL_APP_ID.to_string();
//    let name =format!("{}-{}", SNAPMAIL_APP_ID, ns);
//    println!(" - name: {}", name);
//    let dna_file = DnaFile::new(DnaDef {
//    name,
//    modifiers: {
//       network_seed: ns,
//       properties: SerializedBytes::try_from(()).unwrap(),
//       origin_time: Timestamp::HOLOCHAIN_EPOCH,
//       quantum_time: Duration::from_secs(5 * 60),
//    }
//    integrity_zomes: vec![zome].into(),
//    coordinator_zomes: vec![zome].into(),
//    },
//    vec![dna_wasm].into_iter(),
//    ).await.expect("Dna file load failed");
//    dna_file
// }

/// Install Snapmail DNA from dna file
/// FIXME: hardcoded DNA file path
#[allow(deprecated)]
pub async fn install_app(sid: String, ns: NetworkSeed, maybe_path: Option<PathBuf>) -> ConductorResult<DnaHash> {
   /// Load conductor from config file
   let config_path = Path::new(&*CONFIG_PATH).join(sid.clone());
   let conductor_path = config_path.join(CONDUCTOR_CONFIG_FILENAME);
   let app_filepath = config_path.join(APP_CONFIG_FILENAME);
   std::fs::write(app_filepath, ns.as_bytes())?;
   let conductor = conductor_handle_from_config_path(Some(conductor_path)).await;
   /// Generate keys
   let agent_key = conductor
      .keystore()
      .new_sign_keypair_random()
      .await?;

   /// Load DnaFile
   let dna_file = if let Some(path) = maybe_path {
      println!("Loading DNA from path: {}", path.to_string_lossy());
      load_dna_from_path(ns, &path).await.unwrap()
   } else {
      println!("Building DNA from wasm stored in Rust code.");
      load_dna_from_rs(ns).await
   };

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
   conductor
      .clone()
      .enable_app(SNAPMAIL_APP_ID.into())
      .await?;
   /// Done
   let dnas = conductor.list_dnas();
   msg!("Installed DNAs: {:?}", dnas);
   Ok(dna_file.dna_hash().clone())
}

///
pub fn dump_state(conductor: ConductorHandle) -> usize {
   let result = holochain_util::tokio_helper::block_on(async {
      //let p2p = conductor.holochain_p2p();
      //let broadcaster = conductor.signal_broadcaster();

      let cell_id = &conductor.list_cell_ids(None)[0];

      // let cell = conductor.cell_by_id(cell_id).unwrap();
      // let arc = cell.env();
      // let source_chain = SourceChainBuf::new(arc.clone().into()).unwrap();
      // let source_chain_dump = source_chain.dump_state().await.unwrap();
      //let integration_dump = integrate_dht_ops_workflow::dump_state(arc.clone().into())?;

      let space = cell_id.dna_hash().to_kitsune();
      let p2p_env = conductor.get_p2p_env(space);

      let peer_dump = p2p_agent_store::dump_state(
         p2p_env.into(),
         Some(cell_id.clone()),
      ).await.expect("p2p_agent_store::dump_state() should not fail");

      //let state = conductor.dump_cell_state(&cell_ids[0]).await.unwrap();
      //msg!(" {}", state);

      // msg!("Conductor state dump:");
      // msg!(" - peer dump: {}", peer_dump);
      // msg!(" - Peers: {}", peer_dump.peers.len());
      peer_dump.peers.len()
   }, std::time::Duration::from_secs(9));
   result.expect("dump_state() should not fail")
}