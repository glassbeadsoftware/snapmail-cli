use structopt::StructOpt;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
   if std::env::var_os("RUST_LOG").is_some() {
      observability::init_fmt(observability::Output::Log).ok();
   }
   let opt = cli::Opt::from_args();
   opt.run().await
   //Ok(())
}