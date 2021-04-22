use crate::conductor::*;
use crate::globals::*;
use holochain_types::app::*;
use holochain_zome_types::*;
use holochain::conductor::error::*;
use holochain::conductor::ConductorHandle;


pub fn get_my_handle(conductor: ConductorHandle) -> Result<String, ()> {
   let payload = ExternIO::encode(()).unwrap();
   let result = tokio_helper::block_on(async {
      let result = call_zome(conductor, "get_my_handle", payload).await.unwrap();
      if let ZomeCallResponse::Ok(io) = result {
         let handle: String = io.decode().unwrap();
         return Ok(handle);
      }
      Err(())
   }, *DEFAULT_TIMEOUT).unwrap();
   result
}
