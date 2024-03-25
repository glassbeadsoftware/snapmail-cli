use holochain::conductor::ConductorHandle;
use holochain_types::dna::*;

use std::collections::HashMap;

use snapmail::mail::entries::*;
use snapmail::mail::*;
use snapmail::handle::*;

///
pub struct SnapmailChain {
   pub my_handle: String,
   pub handle_map: HashMap<AgentPubKey, String>,
   pub mail_map: HashMap<ActionHash, MailItem>,
}

impl SnapmailChain {
   /// Pull latest data from the DHT and local source chain
   pub async fn from_latest(conductor: ConductorHandle) -> SnapmailChain {
      /// Get my handle
      /// Cell ID and agent pubkey
      // let cell_ids = conductor.list_cell_ids().await.expect("list_cell_ids() should work");
      // assert!(!cell_ids.is_empty());
      // let agent_pubkey = cell_ids[0].agent_pubkey().to_owned();
      //let my_handle = snapmail_get_handle(conductor.clone(), agent_pubkey).unwrap();
      let my_handle = snapmail_get_my_handle(conductor.clone(), ()).unwrap();
      /// Query DHT
      let handle_list = snapmail_get_all_handles(conductor.clone(), ()).unwrap_or(Vec::new());
      let _new_ack_list = snapmail_check_ack_inbox(conductor.clone(), ());
      let _new_mail_list = snapmail_check_mail_inbox(conductor.clone(), ());

      let all_mail_list = snapmail_get_all_mails(conductor.clone(), ()).unwrap_or(Vec::new());
      /// Change list to HashMap
      let mut handle_map = HashMap::new();
      for item in handle_list {
         handle_map.insert(item.agentId, item.name);
      }
      // /// Get my handle
      // let my_handle = handle_map.get(&agent_pubkey)
      //                           .expect("My handle should be published on the DHT")
      //                           .to_string();
      /// Change list to HashMap
      let mut mail_map = HashMap::new();
      for item in all_mail_list {
         mail_map.insert(item.address.clone(), item.clone());
      }
      /// Done
      SnapmailChain {
         my_handle,
         handle_map,
         mail_map,
      }
   }
}