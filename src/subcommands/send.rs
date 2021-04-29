use crate::{
   utils::*,
};
use structopt::StructOpt;
use holochain::conductor::ConductorHandle;
use snapmail::{
   CHUNK_MAX_SIZE, FILE_MAX_SIZE,
   mail::*,
   file::*,
   handle::*,
};
use std::path::PathBuf;
use holochain_types::dna::*;
use std::io::prelude::*;

#[derive(Debug, StructOpt, Clone)]
pub struct SendCommand {
   #[structopt(long)]
   to: Vec<String>,
   // #[structopt(long)]
   // cc: Option<Vec<String>>,
   #[structopt(short, long)]
   subject: String,
   #[structopt(short, long)]
   message: String,
   #[structopt(name = "attachment", short, long, parse(from_os_str))]
   pub maybe_attachment: Option<PathBuf>,
}


impl SendCommand {
   ///
   pub fn run(self, conductor: ConductorHandle) -> anyhow::Result<()> {
      // Form "to" list
      let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
      let mut to_list: Vec<AgentPubKey> = Vec::new();
      for name in self.to.iter() {
         let agent_id = get_agent_id(&handle_list, name)
            .ok_or(anyhow::Error::msg("username not found"))?;
         to_list.push(agent_id);
      }
      // Form attachment list
      let mut manifest_address_list: Vec<HeaderHash> = Vec::new();
      if let Some(attachment) = self.maybe_attachment {
         let hh = write_attachment(conductor.clone(), attachment).unwrap();
         manifest_address_list.push(hh);
      }
      // Form MailInput
      let mail = SendMailInput {
         subject: self.subject,
         payload: self.message,
         to: to_list,
         cc: vec![],
         bcc: vec![],
         manifest_address_list,
      };
      let send_count = mail.to.len() + mail.cc.len() + mail.bcc.len();
      // Send
      let output = snapmail_send_mail(conductor, mail).unwrap();
      // Show results
      let pending_count = output.to_pendings.len() + output.cc_pendings.len() + output.bcc_pendings.len();
      msg!("Send done: {:?}", output.outmail);
      msg!("   - pendings: {} / {}", pending_count, send_count);
      Ok(())
   }
}

///
fn write_attachment(conductor: ConductorHandle, filepath: PathBuf) -> std::io::Result<HeaderHash> {
   /// Load file
   msg!("Reading attachment file: {:?}", filepath);
   let maybe_filename = filepath.file_name();
   let filename = if let Some(filename) = maybe_filename {
      filename.to_string_lossy().to_string()
   } else { "__no_filename_found_".to_string() };
   let mut file = std::fs::File::open(filepath.clone())?;
   /// Get metadata and check size
   let file_meta = file.metadata().unwrap();
   if file_meta.len() > FILE_MAX_SIZE as u64 {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "Attachment too big!"));
   }
   if !file_meta.is_file() {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "Attachment is not a file!"));
   }
   msg!("filesize: {} KiB", file_meta.len() / 1024);

   /// Read file content & hash it
   let mut file_content = Vec::new();
   file.read_to_end(&mut file_content)?;

   let data_hash = holo_hash::encode::blake2b_256(file_content.as_slice());
   //msg!(" data hash: {}", String::from_utf8_lossy(&data_hash));
   /// Split into chunks
   let chunk_count = (file_meta.len() as f64 / CHUNK_MAX_SIZE as f64).ceil() as usize;
   msg!("chunk_count: {}", chunk_count);
   let mut chunk_list= Vec::with_capacity(chunk_count);
   let mut o: usize = 0;
   for i in 0..chunk_count {
      msg!("chunking: {}", i);
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
      msg!(" chunk size: {} KiB ({} KiB)", chunk.len() / 1024, chunk_b64.len() / 1024);
      let chunk_input = FileChunk {
         data_hash: String::from_utf8_lossy(&chunk_hash).to_string(),
         chunk_index: i,
         chunk: chunk_b64,
      };
      let hh = snapmail_write_chunk(conductor.clone(), WriteChunkInput(chunk_input)).unwrap();
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
   let res = snapmail_write_manifest(conductor, input).unwrap();
   Ok(res)
}

///
pub fn get_attachment(conductor: ConductorHandle, eh: EntryHash) -> std::io::Result<()> {
   let manifest = snapmail_get_manifest(conductor.clone(), AnyDhtHash::from(eh)).unwrap();
   // Print
   msg!("  Filename: {}", manifest.filename);
   msg!("      type: {}", manifest.filetype);
   msg!("      size: {} KiB", manifest.orig_filesize / 1024);

   let mut data: Vec<u8> = Vec::new();

   for chunk_eh in manifest.chunks {
      let chunk_b64 = snapmail_get_chunk(conductor.clone(), chunk_eh).unwrap();
      let chunk = base64::decode_config(chunk_b64.clone(), base64::URL_SAFE_NO_PAD).unwrap();
      msg!(" chunk size: {} KiB ({} KiB)", chunk.len() / 1024, chunk_b64.len() / 1024);
      data.extend(&chunk);
   }

   /// Write file to local dir
   let cd = std::env::current_dir().unwrap();
   let filepath = cd.join(manifest.filename);
   let mut file = std::fs::File::create(filepath.clone())?;
   file.write_all(&data)?;
   msg!("File writen at: {:?}", filepath);

   Ok(())
}
