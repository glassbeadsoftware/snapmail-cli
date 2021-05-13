#![allow(unused_doc_comments)]

use yazi::*;
use holochain_types::dna::wasm::DnaWasm;
//use std::hash::Hash;

pub const WASM_PATH: &str                   = "/home/ddd/github/snapmail-rsm/target/wasm32-unknown-unknown/release/snapmail.wasm";
//pub const DNA_PATH: &str                   = "/home/ddd/github/snapmail-cli/dna/snapmail.dna";
//pub const DNA_PATH: &str                   = "~/github/snapmail-cli/dna/snapmail.dna";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
   /// Load DnaFile
   println!("Loading DNA wasm file: {}", WASM_PATH);
   let wasm = &std::fs::read(WASM_PATH)?;
   let dna_wasm = DnaWasm::from(wasm.to_vec());
   let (_, wasm_hash) = holochain_types::dna::wasm::DnaWasmHashed::from_content(dna_wasm.clone())
      .await
      .into_inner();
   let compressed = compress(wasm, Format::Zlib, CompressionLevel::BestSize).unwrap();
   let compressed_len = compressed.len();
   let chunk_b64 = base64::encode_config(compressed, base64::URL_SAFE_NO_PAD);
   //let wasm_str = format!("pub const DNA_WASM: [u8 ; {}] = {:?};\n", compressed.len(), compressed);
   let wasm_str = format!("pub const SNAPMAIL_WASM_HASH: &str = \"{}\";\npub const DNA_WASM_B64: &str = {:?};\n", wasm_hash, chunk_b64);
   std::fs::write("./wasm.rs", wasm_str.as_bytes())?;
   println!("Wrote file wasm.rs ({} KiB)", wasm_str.len() / 1024);
   println!("Size: {} KiB => {} KiB => {} KiB", wasm.len() / 1024, compressed_len / 1024, chunk_b64.len() / 1024);
   println!("Wasm Hash = {}", wasm_hash);
   Ok(())
}
