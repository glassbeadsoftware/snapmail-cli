use holochain_types::dna::*;
use holochain_types::signal::*;
use snapmail::mail::*;
use snapmail::handle::*;
use snapmail::signal_protocol::*;
use holochain::conductor::ConductorHandle;
use crate::{
   utils::*,
};
use tokio_stream::{StreamExt};
use futures_util::{self, pin_mut};
use tokio::time::{Duration};
use std::sync::mpsc::Sender;

/// Listen to signals and display them in the feedback box
pub async fn listen_signal(/*app: &App, */conductor: ConductorHandle, signal_tx: Sender<String>) -> anyhow::Result<()> {

   /// Add app interface so we can get signals
   let mut interfaces = conductor.list_app_interfaces().await.unwrap();
   if interfaces.is_empty() {
      let _port = conductor.clone().add_app_interface(0).await.unwrap();
      interfaces = conductor.list_app_interfaces().await.unwrap();
      assert!(interfaces.len() > 0);
   }
   // msg!("App Interfaces: {:?}", interfaces);

   let mut handle_list = snapmail_get_all_handles(conductor.clone(), ())?;

   let signal_stream = conductor.signal_broadcaster().await.subscribe_merged();
   pin_mut!(signal_stream);

   /// Infinite loop
   loop {
      let res = tokio::time::timeout(
         Duration::from_millis(50),
         signal_stream.next(),
      ).await;
      match res {
         Err(_) |
         Ok(None) => {
            // let msg = format!("No signal...");
            let msg = String::new();
            let _res = signal_tx.send(msg);
         },
         Ok(Some(signal)) => {
            let msg = print_signal(conductor.clone(), &mut handle_list, signal);
            let _res = signal_tx.send(msg);
         },
      };
      tokio::time::sleep(Duration::from_millis(10)).await;
   }
}


///
fn print_signal(conductor: ConductorHandle, handle_list: &mut GetAllHandlesOutput, signal: Signal) -> String {
   match signal {
      Signal::App(_cell_id, app_signal) => {
         let snapmail_signal: SignalProtocol = app_signal.into_inner().decode().unwrap();
         return print_snapmail_signal(conductor, handle_list, snapmail_signal);
      },
      Signal::System(system_signal) => {
         return format!("{:?}", system_signal);
      },
   }
}

///
fn print_snapmail_signal(
   conductor: ConductorHandle,
   handle_list: &mut GetAllHandlesOutput,
   signal: SignalProtocol,
) -> String {
   match signal {
      SignalProtocol::ReceivedMail(item) => {
         let name = get_handle(conductor.clone(), handle_list, &item.author);
         return format!("Received Mail from {}: \"{}\" ({})", name, item.mail.subject, item.address);
      }
      SignalProtocol::ReceivedAck(ack) => {
         let name = get_handle(conductor.clone(), handle_list, &ack.from);
         let maybe_mail = snapmail_get_mail(conductor.clone(), ack.for_mail.clone()).unwrap();
         let subject = if let Some(mail) = maybe_mail.0 {
            match mail {
               Ok(inmail) => inmail.mail.subject,
               Err(outmail) => outmail.mail.subject,
            }
         } else { "<unknown>".to_string() };
         return format!("Received Acknowledgement from {} for mail \"{}\"", name, subject);
      }
      SignalProtocol::ReceivedFile(manifest) => {
         return format!("Received File {} ({} KiB)", manifest.filename, manifest.orig_filesize);
      }
   }
}

fn get_handle(conductor: ConductorHandle, handle_list: &mut GetAllHandlesOutput, pubkey: &AgentPubKey) -> String {
   let maybe_name = get_name(handle_list, pubkey);
   if maybe_name.is_none() {
      *handle_list = snapmail_get_all_handles(conductor.clone(), ()).unwrap();
      let maybe_name = get_name(handle_list, pubkey);
      return maybe_name.unwrap_or("<Unknown>".to_string());
   }
   maybe_name.unwrap()
}