/// Snapmail CLI
///

use structopt::StructOpt;


///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::InferSubcommands)]
pub enum Opt {
   /// Work with hApp bundles
   App(hc_bundle::HcAppBundle),
   /// Work with DNA bundles
   Dna(hc_bundle::HcDnaBundle),
   /// Work with sandboxed environments for testing and development
   Sandbox(hc_sandbox::HcSandbox),
}

impl Opt {
   /// Run this command
   pub async fn run(self) -> anyhow::Result<()> {
      match self {
         Self::App(cmd) => cmd.run().await?,
         Self::Dna(cmd) => cmd.run().await?,
         Self::Sandbox(cmd) => cmd.run().await?,
      }
      Ok(())
   }
}