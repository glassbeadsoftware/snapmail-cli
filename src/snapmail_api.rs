use crate::{
   globals::*,
   conductor::*,
   error::*,
};
use holochain_types::app::*;
use holochain_zome_types::*;
use holochain::conductor::error::*;
use holochain::conductor::ConductorHandle;
use holo_hash::*;

///
pub fn get_my_handle(conductor: ConductorHandle) -> SnapmailResult<String> {
   let payload = ExternIO::encode(()).unwrap();
   let result = tokio_helper::block_on(async {
      let result = call_zome(conductor, "get_my_handle", payload).await?;
      match result {
         ZomeCallResponse::Ok(io) => {
            let handle: String = io.decode()?;
            Ok(handle)
         },
         ZomeCallResponse::Unauthorized(_, _, _, _) => Err(SnapmailError::Unauthorized),
         ZomeCallResponse::NetworkError(err) => Err(SnapmailError::NetworkError(err)),
      }
   }, *DEFAULT_TIMEOUT).map_err(|_e| SnapmailError::Timeout)?;
   result
}

///
pub fn set_handle(conductor: ConductorHandle, handle: String) -> SnapmailResult<HeaderHash> {
   let payload = ExternIO::encode(handle).unwrap();
   let result = tokio_helper::block_on(async {
      let result = call_zome(conductor, "set_handle", payload).await?;
      match result {
         ZomeCallResponse::Ok(io) => {
            let hash: HeaderHash = io.decode()?;
            Ok(hash)
         },
         ZomeCallResponse::Unauthorized(_, _, _, _) => Err(SnapmailError::Unauthorized),
         ZomeCallResponse::NetworkError(err) => Err(SnapmailError::NetworkError(err)),
      }
   }, *DEFAULT_TIMEOUT).map_err(|_e| SnapmailError::Timeout)?;
   result
}