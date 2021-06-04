use holochain::conductor::ConductorHandle;
use snapmail::{
   CHUNK_MAX_SIZE, FILE_MAX_SIZE,
   file::*,
};
use std::path::PathBuf;
use holochain_types::dna::*;
use std::io::prelude::*;
use std::io::Result;

///
pub fn write_attachment(conductor: ConductorHandle, filepath: PathBuf) -> Result<HeaderHash> {
   /// Load file
   let maybe_filename = filepath.file_name();
   let filename = if let Some(filename) = maybe_filename {
      filename.to_string_lossy().to_string()
   } else { "__no_filename_found_".to_string() };
   let mut file = std::fs::File::open(filepath.clone())?;
   /// Get metadata and check size
   let file_meta = file.metadata().expect("File should always have metadata");
   if file_meta.len() > FILE_MAX_SIZE as u64 {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "Attachment too big!"));
   }
   if !file_meta.is_file() {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "Attachment is not a file!"));
   }
   //msg!("filesize: {} KiB", file_meta.len() / 1024);

   /// Read file content & hash it
   let mut file_content = Vec::new();
   file.read_to_end(&mut file_content)?;

   let data_hash = holo_hash::encode::blake2b_256(file_content.as_slice());
   //msg!(" data hash: {}", String::from_utf8_lossy(&data_hash));
   /// Split into chunks
   let chunk_count = (file_meta.len() as f64 / CHUNK_MAX_SIZE as f64).ceil() as usize;
   //msg!("chunk_count: {}", chunk_count);
   let mut chunk_list= Vec::with_capacity(chunk_count);
   let mut o: usize = 0;
   for _i in 0..chunk_count {
      //msg!("chunking: {}", i);
      let end = std::cmp::min(o + CHUNK_MAX_SIZE, file_meta.len() as usize);
      let chunk_content = &file_content[o..end];
      chunk_list.push(chunk_content);
      o += CHUNK_MAX_SIZE;
   }
   /// Write each chunk
   let mut i = 0;
   let mut chunk_hh_list = Vec::new();
   for chunk in chunk_list {
      let chunk_hash = holo_hash::encode::blake2b_256(chunk);
      let chunk_b64 = base64::encode_config(chunk.clone(), base64::URL_SAFE_NO_PAD);
      //msg!(" chunk_hash: {}", String::from_utf8_lossy(&chunk_hash));
      //msg!(" chunk size: {} KiB ({} KiB)", chunk.len() / 1024, chunk_b64.len() / 1024);
      let chunk_input = FileChunk {
         data_hash: String::from_utf8_lossy(&chunk_hash).to_string(),
         chunk_index: i,
         chunk: chunk_b64,
      };
      let hh = snapmail_write_chunk(conductor.clone(), chunk_input)
         .map_err(|_err| std::io::Error::from(std::io::ErrorKind::Other))?;
      chunk_hh_list.push(hh);
      i += 1;
   }

   /// Write Manifest
   let input = WriteManifestInput {
      data_hash: String::from_utf8_lossy(&data_hash).to_string(),
      filename,
      filetype: filepath.extension().unwrap().to_string_lossy().to_string(),
      orig_filesize: file_meta.len() as usize,
      chunks: chunk_hh_list,
   };
   let res = snapmail_write_manifest(conductor, input)
      .map_err(|_err| std::io::Error::from(std::io::ErrorKind::Other))?;
   Ok(res)
}

///
pub fn get_attachment(conductor: ConductorHandle, eh: EntryHash, path: PathBuf) -> std::io::Result<PathBuf> {
   let manifest = snapmail_get_manifest(conductor.clone(), AnyDhtHash::from(eh))
      .map_err(|_err| std::io::Error::from(std::io::ErrorKind::Other))?;

   // /// Print
   // msg!("  Filename: {}", manifest.filename);
   // msg!("      type: {}", manifest.filetype);
   // msg!("      size: {} KiB", manifest.orig_filesize / 1024);

   let mut data: Vec<u8> = Vec::new();

   for chunk_eh in manifest.chunks {
      let chunk_b64 = snapmail_get_chunk(conductor.clone(), chunk_eh).unwrap();
      let chunk = base64::decode_config(chunk_b64.clone(), base64::URL_SAFE_NO_PAD).unwrap();
      // msg!(" chunk size: {} KiB ({} KiB)", chunk.len() / 1024, chunk_b64.len() / 1024);
      data.extend(&chunk);
   }

   /// Write file to local dir
   let filepath = path.join(manifest.filename);
   let mut file = std::fs::File::create(filepath.clone())?;
   file.write_all(&data)?;
   Ok(filepath)
}
