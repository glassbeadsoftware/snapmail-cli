use structopt::StructOpt;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate url2;
#[macro_use]
pub mod utils;
pub mod subcommands;
pub mod cli;
pub mod globals;
pub mod holochain;
pub mod conductor;
pub mod snapmail_api;
pub mod error;

///
#[tokio::main]
async fn main() -> anyhow::Result<()> {
   if std::env::var_os("RUST_LOG").is_some() {
      observability::init_fmt(observability::Output::Log).ok();
   }
   let opts = cli::SnapCli::from_args();

   println!("{:?}", opts);
   opts.run().await?;
   Ok(())
}