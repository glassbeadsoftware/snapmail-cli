//! Definitions of StructOpt options for use in the CLI
///

use crate::subcommands::setup::SetupSubcommand;
//use holochain_types::prelude::InstalledAppId;
//use std::path::Path;
//use std::path::PathBuf;
use structopt::StructOpt;

const DEFAULT_APP_ID: &str = "snapmail-app";

#[derive(StructOpt, Debug)]
pub enum SnapSubcommand {
   /// Set the agent's handle.
   Setup {
      #[structopt(long)]
      handle: String,
   },
   Info,
   Change {
      #[structopt(long)]
      handle: String,
   },
   Remove,
}

impl SnapSubcommand {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      msg!("running!");
      match self {
         Self::Setup {handle } => {msg!("Setup!"); setup(handle)},
         Self::Change {handle } => msg!("Change!"),
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

