#![allow(unused_doc_comments)]

use yazi::*;
pub const WASM_PATH: &str                   = "/home/ddd/github/snapmail-rsm/target/wasm32-unknown-unknown/release/snapmail.wasm";
//pub const DNA_PATH: &str                   = "/home/ddd/github/snapmail-cli/dna/snapmail.dna";
//pub const DNA_PATH: &str                   = "~/github/snapmail-cli/dna/snapmail.dna";

fn main() -> anyhow::Result<()> {
   /// Load DnaFile
   println!("Loading DNA wasm file: {}", WASM_PATH);
   let wasm = &std::fs::read(WASM_PATH)?;
   let compressed = compress(wasm, Format::Zlib, CompressionLevel::BestSize).unwrap();
   let compressed_len = compressed.len();
   let chunk_b64 = base64::encode_config(compressed, base64::URL_SAFE_NO_PAD);
   //let wasm_str = format!("pub const DNA_WASM: [u8 ; {}] = {:?};\n", compressed.len(), compressed);
   let wasm_str = format!("pub const DNA_WASM_B64: &str = {:?};\n", chunk_b64);
   std::fs::write("./wasm.rs", wasm_str.as_bytes())?;
   println!("Wrote file wasm.rs ({} KiB)", wasm_str.len() / 1024);
   println!("Size: {} KiB => {} KiB => {} KiB", wasm.len() / 1024, compressed_len / 1024, chunk_b64.len() / 1024);
   Ok(())
}
