use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use derive_more::FromStr;

// /// The list of subcommands for `hc sandbox`
// #[derive(Debug, StructOpt)]
// #[structopt(setting = structopt::clap::AppSettings::InferSubcommands)]
// pub enum HcSandboxSubcommand {
//    Handle,

#[derive(Debug, StructOpt, Clone, FromStr)]
pub struct SetupSubcommand {
   /// Set the agent's handle.
   pub handle: String,
}

impl SetupSubcommand {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      msg!("setup!");
      // match self {
      //    Self::handle(cmd) => cmd.run().await?,
      // }
      Ok(())
   }
}