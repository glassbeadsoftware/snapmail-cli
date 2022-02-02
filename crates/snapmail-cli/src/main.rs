#![allow(non_upper_case_globals)]
#![allow(unused_doc_comments)]
#![allow(unused_attributes)]

use structopt::StructOpt;

// #[macro_use]
// extern crate lazy_static;

#[macro_use]
extern crate snapmail_common;

pub mod subcommands;
pub mod cli;

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
