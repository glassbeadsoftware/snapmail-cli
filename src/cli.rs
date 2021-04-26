//! Definitions of StructOpt options for use in the CLI
///

use crate::{
   utils::*,
   subcommands::*,
   conductor::*,
};

use snapmail::handle::*;
use std::path::PathBuf;
use structopt::StructOpt;
use snapmail::mail::*;

//use holochain_zome_types::*;
//use holochain::conductor::ConductorHandle;


#[derive(StructOpt, Debug)]
pub enum SnapSubcommand {
   Setup(SetupCommand),
   Info,
   Change,
   #[structopt(name = "set-handle")]
   SetHandle {
      handle: String,
   },
   GetHandle,
   Clear,
   Ping {
      agent_id: String,
   },
   Pull,
   Directory,
   Send(SendCommand),
   List,
   Open {
      hash: String,
   },
}

impl SnapSubcommand {
   /// Run this command
   pub async fn run(self, uid: PathBuf) -> anyhow::Result<()> {
      msg!("running!");
      match self {
         Self::Setup(cmd)=> {
            msg!("Setup!");
            cmd.run(uid.clone());
            let _ = start_conductor(uid.to_string_lossy().to_string()).await;
         },
         Self::Change => msg!("Change! (TODO)"),
         Self::Send(cmd) => {
            msg!("Send!");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            cmd.run(conductor);
         },
         Self::SetHandle {handle } => {
            msg!("** Set handle: {}", handle);
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let hash = snapmail_set_handle(conductor, handle)?;
            msg!(" - {:?}", hash);
         },
         Self::GetHandle => {
            msg!("** Get handle: ");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let handle = snapmail_get_my_handle(conductor, ())?;
            msg!(" - {:?}", handle);
         },
         Self::Clear => { msg!("Clearing..."); clear(uid); },
         Self::Ping { agent_id } => {
            msg!("Ping...");
            let key = stoh(agent_id);
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let res = snapmail_ping_agent(conductor, key);
            match res {
               Ok(ponged) => {
                  msg!(" - {:?}", ponged);
               }
               Err(err) => {
                  msg ! (" - Failed: {:?}", err);
               }
            }
         },
         Self::Open { hash } => {
            msg!("Open...");
            let hh = stohh(hash);
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let maybe_mail = snapmail_get_mail(conductor.clone(), hh.clone())?;
            if let Some(mail) = maybe_mail.0 {
               let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
               msg!(" - mail: {:?}", mail);
               match mail {
                  Ok(inmail) => {
                     msg!("Subject: {}", inmail.mail.subject);
                     msg!("   From: {}", get_name(&handle_list, &inmail.from));
                     msg!("   Date: {}", inmail.mail.date_sent);
                     msg!("    Att: {}", inmail.mail.attachments.len());
                     msg!("\n\n{}\n", inmail.mail.payload);
                     let maybe_hash = snapmail_acknowledge_mail(conductor, hh);
                     if let Ok(hash) = maybe_hash {
                        msg!("Acknowledged: {}", hash);
                     }
                  },
                  Err(outmail) => {
                     msg!(" - outmail to : {:?}", outmail.mail.to);
                  },
               }
            } else {
               msg!(" !! No mail found at this hash");
            }
         },
         Self::Directory => {
            msg!("Directory...");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let handle_list = snapmail_get_all_handles(conductor, ())?.0;
            for pair in handle_list.iter() {
               msg!(" - {} - {}", pair.0, pair.1);
            }
         },
         Self::List => {
            msg!("List inbox...");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let all_mail_list = snapmail_get_all_mails(conductor.clone(), ())?.0;
            let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
            msg!(" {} mail(s) found:", all_mail_list.len());
            for item in all_mail_list.iter() {
               let username = get_name(&handle_list, &item.author);
               msg!("- {:?} | {} | {} | {}", item.state, username, item.mail.subject, item.address);
            }
         },
         Self::Pull => {
            msg!("Pull...");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let handle_list = snapmail_get_all_handles(conductor.clone(), ())?.0;
            let new_ack_list = snapmail_check_incoming_ack(conductor.clone(), ())?;
            msg!(" -  New Acks: {}", new_ack_list.len());
            let new_mail_list = snapmail_check_incoming_mail(conductor.clone(), ())?;
            msg!(" - New Mails: {}", new_mail_list.len());
            for mail_item in new_mail_list.iter() {
               msg!(" - {:?}", mail_item);
            }
            msg!(" -   Handles: {}", handle_list.len());
            let all_mail_list = snapmail_get_all_mails(conductor.clone(), ())?.0;
            msg!(" - All Mails: {}", all_mail_list.len());
         },
         _ => msg!("unimplemented!"),
      }
      Ok(())
   }
}


///
#[derive(Debug, StructOpt)]
#[structopt(name = "snapmail-cli", about = "Command line interface for Snapmail DNA")]
pub struct SnapCli {
   #[structopt(parse(from_os_str))]
   uid: PathBuf,
   #[structopt(subcommand)]
   cmd: SnapSubcommand,
}

impl SnapCli {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      self.cmd.run(self.uid).await
   }
}

