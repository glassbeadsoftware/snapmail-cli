//! Definitions of StructOpt options for use in the CLI
///

use crate::{
   utils::*,
   subcommands::*,
   conductor::*,
   subcommands::open,
};

use snapmail::handle::*;
use std::path::PathBuf;
use structopt::StructOpt;
use snapmail::mail::*;
use holochain_types::dna::*;

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
   GetAttachment {
      hash: String,
   },
   Listen,
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
         Self::Listen => {
            msg!("Listening forever:");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            listen(conductor)?;
         },
         Self::Send(cmd) => {
            msg!("Send!");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            cmd.run(conductor)?;
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
            let hh: HeaderHash = stoh(hash);
            let uid_str = uid.to_string_lossy().to_string();
            open(uid_str, hh).await?;
         },
         Self::GetAttachment { hash } => {
            msg!("GetAttachment...");
            let eh: EntryHash = stoh(hash);
            //let uid_str = uid.to_string_lossy().to_string();
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            get_attachment(conductor, eh)?;
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
               let username = get_name(&handle_list, &item.author).unwrap();
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

