use structopt::StructOpt;

#[macro_use]
pub mod utils;
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
   opts.run().await?;
   Ok(())
}