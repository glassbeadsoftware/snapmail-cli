//! Definitions of StructOpt options for use in the CLI
///

use crate::{
   subcommands::*,
   conductor::*,
};

use snapmail::handle::*;
use snapmail_api::*;
use std::path::PathBuf;
use structopt::StructOpt;
use holochain_zome_types::*;
use holochain::conductor::ConductorHandle;


#[derive(StructOpt, Debug)]
pub enum SnapSubcommand {
   Setup(SetupCommand),
   Info,
   Change,
   #[structopt(name = "set-handle")]
   SetHandle {
      #[structopt(parse(from_os_str))]
      uid: PathBuf,
      handle: String,
   },
   GetHandle {
      #[structopt(parse(from_os_str))]
      uid: PathBuf,
   },
   Clear {
      #[structopt(parse(from_os_str))]
      uid: PathBuf,
   }
}

impl SnapSubcommand {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      msg!("running!");
      match self {
         Self::Setup(cmd)=> {msg!("Setup!"); cmd.run();},
         Self::Change => msg!("Change!"),
         Self::SetHandle {uid, handle } => {
            msg!("** Set handle: {}", handle);
            let conductor: ConductorHandle = start_conductor(uid.to_string_lossy().to_string()).await;
            let hash = snapmail_set_handle(conductor, handle)?;
            msg!(" - {:?}", hash);
         },
         Self::GetHandle {uid } => {
            msg!("** Get handle: ");
            let conductor = start_conductor(uid.to_string_lossy().to_string()).await;
            let handle: String = snapmail_get_my_handle(conductor, ())?;
            msg!(" - {}", handle);
         },
         Self::Clear {uid } => {msg!("Clearing..."); clear(uid)},
         _ => msg!("unimplemented!"),
      }
      Ok(())
   }
}


///
#[derive(Debug, StructOpt)]
#[structopt(name = "snapmail-cli", about = "Command line interface for Snapmail DNA")]
pub struct SnapCli {
   #[structopt(subcommand)]
   cmd: SnapSubcommand,
}

impl SnapCli {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      self.cmd.run().await
   }
}

