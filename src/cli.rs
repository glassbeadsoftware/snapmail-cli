//! Definitions of StructOpt options for use in the CLI
///

use crate::subcommands::*;
//use holochain_types::prelude::InstalledAppId;
//use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use url2::Url2;

const DEFAULT_APP_ID: &str = "snapmail-app";


#[derive(StructOpt, Debug)]
pub enum SnapSubcommand {
   Setup(SetupCommand),
   Info,
   Change,
   #[structopt(name = "set-handle")]
   SetHandle {
      #[structopt(long)]
      handle: String,
   },
   Clear,
}

impl SnapSubcommand {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      msg!("running!");
      match self {
         Self::Setup(cmd)=> {msg!("Setup!"); cmd.run();},
         Self::Change => msg!("Change!"),
         Self::SetHandle {handle } => msg!("** Set handle: {}", handle),
         Self::Clear => {msg!("Clearing..."); clear()},
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

