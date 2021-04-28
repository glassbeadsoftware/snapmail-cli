use holochain_types::dna::*;
use holochain_types::signal::*;
use snapmail::mail::*;
use snapmail::handle::*;
use snapmail::signal_protocol::*;
//use std::{thread, time};
use holochain::conductor::{ConductorHandle};
use crate::{
   conductor::*,
   utils::*,
};
//use chrono::Utc;
use tokio_stream::{StreamExt};
use futures_util::{self, pin_mut};
//use tokio::StreamExt;
//use futures::Stream;
//use futures::StreamExt;
use tokio::time::{Duration};


/// Always on conductor that displays events & signals
pub async fn listen(conductor: ConductorHandle) -> anyhow::Result<()> {

   //let start = Utc::now();
   let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;

   let signal_stream = conductor.signal_broadcaster().await.subscribe_merged();
   pin_mut!(signal_stream);

   /// Infinite loop
   loop {
      let res = tokio::time::timeout(
         Duration::from_millis(5000),
         signal_stream.next(),
      ).await;

      match res {
         Err(_e) => {
               let peer_count = dump_state(conductor.clone());
               let all_mail_list = snapmail_get_all_mails(conductor.clone(), ())?.0;
               msg!("Peers: {} | Mails: {}", peer_count, all_mail_list.len());
         },
         Ok(None) => msg!("No signal found"),
         Ok(Some(signal)) => {
            match signal {
               Signal::App(_cell_id, app_signal) => {
                  let snapmail_signal: SignalProtocol = app_signal.into_inner().decode().unwrap();
                  print_signal(conductor.clone(), &handle_list, snapmail_signal);
               },
               Signal::System(system_signal) => {
                  msg!("{:?}", system_signal);
               },
            }
         }
      }

      tokio::time::sleep(Duration::from_millis(10)).await;

      // let process = signal_stream.for_each(|item| {
      //    let now = Utc::now().format("%H:%M:%S");
      //    msg!("[{}] signals: {:?}", now, item);
      // });

     // let res = process.timeout(std::time::Duration::from_millis(100));


      // let new_mail_list = snapmail_check_incoming_mail(conductor.clone(), ())?;
      // if !new_mail_list.is_empty() {
      //    msg!("[{}] New mail(s) received: {}", now, new_mail_list.len());
      //    for hh in new_mail_list.iter() {
      //       let maybe_mail = snapmail_get_mail(conductor.clone(), hh.clone())?.0;
      //       let inmail = maybe_mail.unwrap().ok().unwrap();
      //       let username = try_get_name(conductor.clone(), &inmail.from).unwrap();
      //       msg!("   - {} | {} | {} file(s)", username, inmail.mail.subject, inmail.mail.attachments.len());
      //    }
      // }
      // let new_ack_list = snapmail_check_incoming_ack(conductor.clone(), ())?;
      // if !new_ack_list.is_empty() {
      //    msg!("[{}] New ack(s) received: {}", now, new_ack_list.len());
      //    for eh in new_ack_list.iter() {
      //       msg!("   - {:?}", eh);
      //    }
      // }
   }
}


fn print_signal(conductor: ConductorHandle, handle_list: &GetAllHandlesOutput, signal: SignalProtocol) {
   match signal {
      SignalProtocol::ReceivedMail(item) => {
         let name = get_name(handle_list, &item.author).unwrap_or("<unknown>".to_string());
         msg!("Received Mail from {}: \"{}\" ({})", name, item.mail.subject, item.address);
      }
      SignalProtocol::ReceivedAck(ack) => {
         let name = get_name(handle_list, &ack.from).unwrap_or("<unknown>".to_string());
         let maybe_mail = snapmail_get_mail(conductor.clone(), ack.for_mail.clone()).unwrap();
         let subject = if let Some(mail) = maybe_mail.0 {
            match mail {
               Ok(inmail) => inmail.mail.subject,
               Err(outmail) => outmail.mail.subject,
            }
         } else { "<unknown>".to_string() };
         msg!("Received Acknowledgement from {} for mail \"{}\"", name, subject);
      }
      SignalProtocol::ReceivedFile(manifest) => {
         msg!("Received File {} ({} KiB)", manifest.filename, manifest.orig_filesize);
      }
   }
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