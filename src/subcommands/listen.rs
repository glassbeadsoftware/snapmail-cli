use holochain_types::dna::*;
use snapmail::mail::*;
use snapmail::handle::*;
use std::{thread, time};
use holochain::conductor::{ConductorHandle};

/// Always on conductor that displays events & signals
pub fn listen(conductor: ConductorHandle) -> anyhow::Result<()> {
   /// Infinite loop
   loop {
      let now = time::Instant::now();
      let new_mail_list = snapmail_check_incoming_mail(conductor.clone(), ())?;
      if !new_mail_list.is_empty() {
         msg!("[{:?}] New mail(s) received: {}", now, new_mail_list.len());
         for hh in new_mail_list.iter() {
            let maybe_mail = snapmail_get_mail(conductor.clone(), hh.clone())?.0;
            let inmail = maybe_mail.unwrap().ok().unwrap();
            let username = try_get_name(conductor.clone(), &inmail.from).unwrap();
            msg!("   - {} | {} | {} file(s)", username, inmail.mail.subject, inmail.mail.attachments.len());
         }
      }
      let new_ack_list = snapmail_check_incoming_ack(conductor.clone(), ())?;
      if !new_ack_list.is_empty() {
         msg!("[{:?}] New ack(s) received: {}", now, new_ack_list.len());
         for eh in new_ack_list.iter() {
            msg!("   - {:?}", eh);
         }
      }
      thread::sleep(time::Duration::from_millis(100));
   }
   //Ok(())
}



/// Get username from AgentPubKey
/// Update Handle list if necessary
pub fn try_get_name(conductor: ConductorHandle, candidate: &AgentPubKey) -> Result<String, ()> {
   let handle_list = snapmail_get_all_handles(conductor.clone(), ()).unwrap().0;
   for pair in handle_list.iter() {
      if &pair.1 == candidate {
         return Ok(pair.0.clone());
      }
   }
   Err(())
}