//! Definitions of StructOpt options for use in the CLI
///

use crate::{
   utils::*,
   globals::*,
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
   /// Create agent and config
   Setup(SetupCommand),
   /// Display setup (conductor config, uid)
   Info,
   /// Modify the setup
   Change(ChangeCommand),
   /// Change agent's handle
   SetHandle {
      /// New handle name to use for this agent
      handle: String,
   },
   /// Get agent's current handle
   GetHandle,
   /// Erase a session from disk
   Clear,
   /// Check if a user is currently online
   Ping {
      #[structopt(name = "name", short, long)]
      /// Handle of the agent to Ping
      maybe_name: Option<String>,
      #[structopt(name = "id", long, required_unless = "name")]
      /// Agent ID of agent to ping
      maybe_agent_id: Option<String>,
   },
   /// Query the DHT for all relevant data (handles, mailbox, ackbox)
   Pull,
   /// Display all users part of the current network
   Directory,
   /// Send a mail to another agent
   Send(SendCommand),
   /// List all mails received by this agent
   List,
   /// List sessions that have been setup on this computer
   ListSessions,
   /// Read mail from mailbox (Will send an acknowledgement to mail author)
   Open {
      /// Hash of the mail to open
      hash: String,
   },
   /// Extract an attachment from a mail
   GetAttachment {
      /// Hash of the attachment to extract
      hash: String,
   },
   /// Launch an "always on" conductor that displays events & signals
   Listen,
}

impl SnapSubcommand {
   /// Run this command
   pub async fn run(self, sid: PathBuf) -> anyhow::Result<()> {
      let sid_str = sid.to_string_lossy().to_string();

      match self {
         Self::Setup(cmd)=> {
            msg!("Setup!");
            cmd.run(sid).await;
         },
         Self::Info => {
            msg!("{} Info:", sid_str);
            let path = CONFIG_PATH.as_path().join(sid);
            let config_filepath = path.join(CONDUCTOR_CONFIG_FILENAME);
            let app_filepath = path.join(APP_CONFIG_FILENAME);
            let uid = std::fs::read_to_string(app_filepath)
               .expect("Something went wrong reading APP CONFIG file");
            msg!("uid: {}", uid);
            // let f = std::fs::File::open(config_filepath)
            //    .expect("Something went wrong reading CONDUCTOR CONFIG file");
            let s = std::fs::read_to_string(config_filepath)
               .expect("Something went wrong reading CONDUCTOR CONFIG file");
            msg!("config:\n {}", s);

         },
         Self::Change(cmd) => {
            msg!("Change!");
            cmd.run(sid);
         },
         Self::ListSessions => {
            msg!("ListSessions: ");
            let root = CONFIG_PATH.as_path().to_path_buf();
            let paths = std::fs::read_dir(root).unwrap();
            for path in paths {
               msg!(" - {}", path.unwrap().path().display())
            }
         },
         Self::Clear => { msg!("Clearing..."); clear(sid); },
         Self::Listen => {
            msg!("Listening forever:");
            let conductor = start_conductor(sid_str).await;
            listen(conductor).await?;
         },
         Self::Send(cmd) => {
            msg!("Send!");
            let conductor = start_conductor(sid_str).await;
            cmd.run(conductor)?;
         },
         Self::SetHandle {handle } => {
            msg!("** Set handle: {}", handle);
            let conductor = start_conductor(sid_str).await;
            let hash = snapmail_set_handle(conductor, handle)?;
            msg!(" - {:?}", hash);
         },
         Self::GetHandle => {
            msg!("** Get handle: ");
            let conductor = start_conductor(sid_str.clone()).await;
            let handle = snapmail_get_my_handle(conductor, ())?;
            msg!("** Active handle for session {} : \"{}\"", sid_str, handle);
         },

         Self::Ping { maybe_name, maybe_agent_id } => {
            msg!("Ping...");
            let conductor = start_conductor(sid_str).await;
            let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
            let maybe_key = if let Some(name) = maybe_name {
               get_agent_id(&handle_list, &name)
            } else {
               let key = stoh(maybe_agent_id.unwrap());
               if let None = get_name(&handle_list, &key) { None }
                  else { Some(key) }

            };
            if let Some(key) = maybe_key {
               let res = snapmail_ping_agent(conductor, key);
               match res {
                  Ok(ponged) => msg!(" - {:?}", ponged),
                  Err(err) => err_msg!(" - Failed: {:?}", err),
               }
            } else {
               err_msg!(" - Unknown agent");
            }
         },
         Self::Open { hash } => {
            msg!("Open...");
            let hh: HeaderHash = stoh(hash);
            open(sid_str, hh).await?;
         },
         Self::GetAttachment { hash } => {
            msg!("GetAttachment...");
            let eh: EntryHash = stoh(hash);
            //let uid_str = uid.to_string_lossy().to_string();
            let conductor = start_conductor(sid_str).await;
            get_attachment(conductor, eh)?;
         },
         Self::Directory => {
            msg!("Directory...");
            let conductor = start_conductor(sid_str).await;
            let handle_list = snapmail_get_all_handles(conductor, ())?.0;
            for pair in handle_list.iter() {
               msg!(" - {} - {}", pair.0, pair.1);
            }
         },
         Self::List => {
            msg!("List inbox...");
            let conductor = start_conductor(sid_str).await;
            let all_mail_list = snapmail_get_all_mails(conductor.clone(), ())?.0;
            let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
            msg!(" {} mail(s) found:", all_mail_list.len());
            for item in all_mail_list.iter() {
               let username = get_name(&handle_list, &item.author).unwrap();
               msg!("- {:?} | {} | {} | {}", item.state, username, item.mail.subject, item.address);
            }
            dump_state(conductor);
         },
         Self::Pull => {
            msg!("Pull...");
            let conductor = start_conductor(sid_str).await;
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
      }
      Ok(())
   }
}


///
#[derive(Debug, StructOpt)]
#[structopt(name = "snapmail-cli", about = "Command line interface for Snapmail DNA")]
pub struct SnapCli {
   #[structopt(parse(from_os_str))]
   /// Session ID (user defined). Corresponds to an unique config, network id and agent
   sid: PathBuf,
   #[structopt(subcommand)]
   cmd: SnapSubcommand,
}

impl SnapCli {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      self.cmd.run(self.sid).await
   }
}

