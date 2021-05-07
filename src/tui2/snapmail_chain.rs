//use holo_hash::EntryHash;
//use holo_hash::AgentPubKey;
use holochain::conductor::ConductorHandle;
use holochain_types::dna::*;

use std::collections::HashMap;

use snapmail::mail::entries::*;
use snapmail::mail::*;
use snapmail::handle::*;
use snapmail::file::*;

pub struct SnapmailChain {
   pub my_handle: String,
   pub handle_map: HashMap<AgentPubKey, String>,
   pub mail_map: HashMap<HeaderHash, MailItem>,
}


///
pub async fn pull_source_chain(conductor: ConductorHandle) -> SnapmailChain {
   /// Query DHT
   let my_handle = snapmail_get_my_handle(conductor.clone(), ()).unwrap();
   let handle_list = snapmail_get_all_handles(conductor.clone(), ()).unwrap().0;
   let _new_ack_list = snapmail_check_incoming_ack(conductor.clone(), ()).unwrap();
   let _new_mail_list = snapmail_check_incoming_mail(conductor.clone(), ()).unwrap();
   let all_mail_list = snapmail_get_all_mails(conductor.clone(), ()).unwrap().0;
   /// Change list to HashMap
   let mut handle_map = HashMap::new();
   for item in handle_list {
      handle_map.insert(item.1, item.0);
   }
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
