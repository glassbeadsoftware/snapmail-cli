#![allow(unused_doc_comments)]

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
pub mod error;
pub mod attachment;
pub mod config;
pub mod wasm;

///
#[tokio::main]
async fn main() -> anyhow::Result<()> {
   if std::env::var_os("RUST_LOG").is_some() {
      observability::init_fmt(observability::Output::Log).ok();
   }
   let opts = cli::SnapCli::from_args();

   println!("{:?}", opts);
   let res = opts.run().await;
   if let Err(e) = res {
      err_msg!("{}", e);
   }
   Ok(())
}
