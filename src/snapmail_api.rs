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

/// Macro for calling call_zome()
macro_rules! snapmail {
    ($handle:tt, $name:tt, $ret:ty, $payload:tt) => ({
      let payload = ExternIO::encode($payload).unwrap();
      let result = tokio_helper::block_on(async {
         let result = call_zome($handle, $name, payload).await?;
         match result {
            ZomeCallResponse::Ok(io) => {
               let hash: $ret = io.decode()?;
               Ok(hash)
            },
            ZomeCallResponse::Unauthorized(_, _, _, _) => Err(SnapmailError::Unauthorized),
            ZomeCallResponse::NetworkError(err) => Err(SnapmailError::NetworkError(err)),
         }
      }, *DEFAULT_TIMEOUT).map_err(|_e| SnapmailError::Timeout)?;
      result
    })
}

///
pub fn get_my_handle(conductor: ConductorHandle) -> SnapmailResult<String> {
   snapmail!(conductor, "get_my_handle", String, ())
}

///
pub fn set_handle(conductor: ConductorHandle, handle: String) -> SnapmailResult<HeaderHash> {
   snapmail!(conductor, "set_handle", HeaderHash, handle)
}
