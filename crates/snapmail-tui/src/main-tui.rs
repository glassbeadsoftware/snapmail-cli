#![allow(non_upper_case_globals)]
#![allow(unused_doc_comments)]
#![allow(unused_attributes)]

#[macro_use]
extern crate strum_macros;

pub mod render;
pub mod tables;
pub mod app;
pub mod listen_signal;
pub mod menu;
pub mod run;
pub mod snapmail_chain;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use snapmail_common::{
   globals::*,
};
use tui::{
   Terminal,
   backend::CrosstermBackend,
};


static USAGE_TEXT: &str = "USAGE:
    snapmail-tui <sid>

FLAGS:
    -l, List available Session IDs
    -h, Prints help information
    -V, Prints version information

ARGS:
    <sid>    Session ID. Corresponds to an unique config, network id and agent
    ";

///
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

   /// Parse args
   let args: Vec<String> = std::env::args().collect();
   //println!("{:?}", args);
   if args.len() != 2 {
      println!("Wrong number of arguments. Expected:\n");
      println!("{}", USAGE_TEXT);
      return Ok(());
   }
   if args[1] == "-h" {
      println!("\n{}", USAGE_TEXT);
      return Ok(());
   }
   if args[1] == "-v" {
      println!("\n{}", SNAPMAIL_VERSION);
      return Ok(());
   }
   if args[1] == "-l" {
      println!("Available Session IDs: ");
      let root = CONFIG_PATH.as_path().to_path_buf();
      let paths = std::fs::read_dir(root).expect("Should have config dir set up during setup");
      for path in paths {
         println!(" - {}", path.unwrap().path().display());
      }
      return Ok(());
   }
   let sid = args[1].to_string();

   /// Set raw mode ('Enter' not required)
   enable_raw_mode().expect("can run in raw mode");

   /// Setup terminal
   let stdout = io::stdout();
   let backend = CrosstermBackend::new(stdout);
   let mut terminal = Terminal::new(backend)?;
   terminal.clear()?;

   /// Run TUI app
   std::env::set_var("WASM_LOG", "NONE");
   std::env::set_var("RUST_LOG", "NONE");
   let _res = crate::run::run(&mut terminal, sid).await;

   /// Clean up & Shutdown
   terminal.clear()?;
   disable_raw_mode()?;
   terminal.show_cursor()?;
   Ok(())
}
