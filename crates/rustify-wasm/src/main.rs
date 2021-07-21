#![allow(unused_doc_comments)]

use yazi::*;
use holochain_types::dna::wasm::DnaWasm;
//use std::hash::Hash;

pub const DEFAULT_WASM_PATH: &str = "/home/ddd/github/snapmail-rsm/target/wasm32-unknown-unknown/release/snapmail.wasm";
//pub const DNA_PATH: &str        = "/home/ddd/github/snapmail-cli/dna/snapmail.dna";
//pub const DNA_PATH: &str        = "~/github/snapmail-cli/dna/snapmail.dna";

pub const RUST_OUTPUT: &str = "./crates/common/src/wasm.rs";

#[tokio::main]
async fn main() -> anyhow::Result<()> {

   // Prints each argument on a separate line
   // for argument in std::env::args() {
   //    println!("{}", argument);
   // }
   let mut args = std::env::args();
   args.next(); // skip exe
   let wasm_path = if let Some(arg) = args.next() {
      arg
   } else {
      DEFAULT_WASM_PATH.to_string()
   };

   /// Load DnaFile
   println!("Loading DNA wasm file: {}", wasm_path);
   let wasm = &std::fs::read(wasm_path)?;
   let dna_wasm = DnaWasm::from(wasm.to_vec());
   let (_, wasm_hash) = holochain_types::dna::wasm::DnaWasmHashed::from_content(dna_wasm.clone())
      .await
      .into_inner();
   let compressed = compress(wasm, Format::Zlib, CompressionLevel::BestSize).unwrap();
   let compressed_len = compressed.len();
   let chunk_b64 = base64::encode_config(compressed, base64::URL_SAFE_NO_PAD);
   //let wasm_str = format!("pub const DNA_WASM: [u8 ; {}] = {:?};\n", compressed.len(), compressed);
   let wasm_str = format!("pub const SNAPMAIL_WASM_HASH: &str = \"{}\";\npub const DNA_WASM_B64: &str = {:?};\n", wasm_hash, chunk_b64);

   let path = std::env::current_dir()?;
   println!("writing Rust file at: {:?}", path.join(std::path::Path::new(RUST_OUTPUT)));
   std::fs::write(RUST_OUTPUT, wasm_str.as_bytes())?;
   println!("Wrote file wasm.rs ({} KiB)", wasm_str.len() / 1024);
   println!("Size: {} KiB => {} KiB => {} KiB", wasm.len() / 1024, compressed_len / 1024, chunk_b64.len() / 1024);
   println!("Wasm Hash = {}", wasm_hash);
   Ok(())
}
