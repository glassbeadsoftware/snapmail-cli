#![allow(unused_doc_comments)]

pub const WASM_PATH: &str                   = "/home/ddd/github/snapmail-rsm/target/wasm32-unknown-unknown/release/snapmail.wasm";
//pub const DNA_PATH: &str                   = "/home/ddd/github/snapmail-cli/dna/snapmail.dna";
//pub const DNA_PATH: &str                   = "~/github/snapmail-cli/dna/snapmail.dna";

fn main() -> anyhow::Result<()> {
   /// Load DnaFile
   println!("Loading DNA wasm file: {}", WASM_PATH);
   let wasm = &std::fs::read(WASM_PATH)?;
   let wasm_str = format!("pub const DNA_WASM: [u8 ; {}] = {:?};\n", wasm.len(), wasm);
   std::fs::write("./wasm.rs", wasm_str.as_bytes())?;
   println!("Wrote file wasm.rs ({})", wasm_str.len());
   Ok(())
}
