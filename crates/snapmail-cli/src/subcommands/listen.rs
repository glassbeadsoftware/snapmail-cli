use holochain_types::dna::*;
use holochain_types::signal::*;
use snapmail::mail::*;
use snapmail::handle::*;
use snapmail::signal_protocol::*;
use holochain::conductor::{ConductorHandle};
use snapmail_common::{
   conductor::*,
   utils::*,
};
use tokio_stream::{StreamExt};
use futures_util::{self, pin_mut};
use tokio::time::{Duration};


/// Launch an "always on" conductor that displays events & signals
pub async fn listen(conductor: ConductorHandle, loop_interval_sec: u64) -> anyhow::Result<()> {

   /// Add app interface so we can get signals
   let mut interfaces = conductor.list_app_interfaces().await.unwrap();
   if interfaces.is_empty() {
      let _port = conductor.clone().add_app_interface(0).await.unwrap();
      interfaces = conductor.list_app_interfaces().await.unwrap();
   }
   msg!("App Interfaces: {:?}", interfaces);

   let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;

   let signal_stream = conductor.signal_broadcaster().await.subscribe_merged();
   pin_mut!(signal_stream);

   // while let Some(signal) = signal_stream.next().await {
   //    print_signal(conductor.clone(), &handle_list, signal)
   // }

   /// Infinite loop
   loop {
      let res = tokio::time::timeout(
         Duration::from_secs(loop_interval_sec),
         signal_stream.next(),
      ).await;
      match res {
         Err(_e) => {
               let peer_count = dump_state(conductor.clone());
               let all_mail_list = snapmail_get_all_mails(conductor.clone(), ())?.0;
               msg!("Peers: {} | Mails: {}", peer_count, all_mail_list.len());
         },
         Ok(None) => msg!("No signal found"),
         Ok(Some(signal)) => print_signal(conductor.clone(), &handle_list, signal),
      }
      tokio::time::sleep(Duration::from_millis(100)).await;
   }
   //Ok(())
}

///
fn print_signal(conductor: ConductorHandle, handle_list: &GetAllHandlesOutput, signal: Signal) {
   match signal {
      Signal::App(_cell_id, app_signal) => {
         let snapmail_signal: SignalProtocol = app_signal.into_inner().decode().unwrap();
         print_snapmail_signal(conductor, &handle_list, snapmail_signal);
      },
      Signal::System(system_signal) => {
         msg!("{:?}", system_signal);
      },
   }
}

///
fn print_snapmail_signal(conductor: ConductorHandle, handle_list: &GetAllHandlesOutput, signal: SignalProtocol) {
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