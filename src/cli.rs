//! Definitions of StructOpt options for use in the CLI
///

use crate::subcommands::setup::SetupSubcommand;
//use holochain_types::prelude::InstalledAppId;
//use std::path::Path;
//use std::path::PathBuf;
use structopt::StructOpt;

const DEFAULT_APP_ID: &str = "snapmail-app";

///
#[derive(Debug, StructOpt)]
#[structopt(name = "snapmail-cli", about = "Command line interface for the Snapmail DNA")]
pub struct SnapCli {
   #[structopt(subcommand)]
   setup: SetupSubcommand,
}

impl SnapCli {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      // match self {
      //    Self::setup(cmd) => cmd.run().await?,
      // }
      Ok(())
   }
}