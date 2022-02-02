#![allow(non_upper_case_globals)]
#![allow(unused_doc_comments)]
#![allow(unused_attributes)]

//use structopt::StructOpt;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate url2;

#[macro_use]
pub mod utils;
#[macro_use]
pub mod attachment;
pub mod conductor;
pub mod config;
pub mod error;
pub mod globals;
pub mod holochain;
pub mod wasm;
