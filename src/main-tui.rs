#![allow(unused_doc_comments)]
#![allow(non_upper_case_globals)]

pub mod error;
pub mod globals;
#[macro_use]
pub mod utils;
pub mod holochain;
pub mod conductor;
pub mod tui2;
pub mod wasm;
pub mod attachment;

#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate url2;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use crate::{
   //tui2::*,
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
      println!("\n v0.0.4");
      return Ok(());
   }
   if args[1] == "-l" {
      println!("Available Session IDs: ");
      let root = CONFIG_PATH.as_path().to_path_buf();
      let paths = std::fs::read_dir(root).unwrap();
      for path in paths {
         println!(" - {}", path.unwrap().path().display());
      }
      return Ok(());
   }
   //let sid = args[1].to_string();

   /// Set raw mode ('Enter' not required)
   enable_raw_mode().expect("can run in raw mode");

   /// Setup terminal
   let stdout = io::stdout();
   let backend = CrosstermBackend::new(stdout);
   let mut terminal = Terminal::new(backend)?;
   terminal.clear()?;

   /// Run TUI app
   let _res = tui2::run(&mut terminal, args[1].clone()).await;

   terminal.clear()?;

   /// Shutdown terminal
   disable_raw_mode()?;
   terminal.show_cursor()?;
   Ok(())
}
