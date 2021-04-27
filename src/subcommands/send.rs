use crate::{
   //globals::*,
   utils::*,
};
use structopt::StructOpt;
use holochain::conductor::ConductorHandle;
use snapmail::{
   CHUNK_MAX_SIZE, FILE_MAX_SIZE,
   mail::*,
   file::*,
};
use std::path::PathBuf;
use holochain_types::dna::*;
use std::fs::File;
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
   pub fn run(self, conductor: ConductorHandle) {
      // Form "to" list
      let mut to_list: Vec<AgentPubKey> = Vec::new();
      for id_str in self.to.iter() {
         to_list.push(stoh(id_str.to_owned()));
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
   }
}


// export function splitFile(full_data_string) {
//    const hash = sha256(full_data_string);
//    console.log('file hash: ' + hash)
//    const chunks = chunkSubstr(full_data_string, CHUNK_MAX_SIZE);
//    return {
//    dataHash: hash,
//    numChunks: chunks.length,
//    chunks: chunks,
//    }
// }


fn write_attachment(conductor: ConductorHandle, filepath: PathBuf) -> std::io::Result<HeaderHash> {
   /// Load file
   println!("Reading attachmentfile: {:?}", filepath);
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

   /// Read file content & hash it
   let mut file_content = Vec::new();
   file.read_to_end(&mut file_content)?;
   //let file_slice: &[u8] = file_content.as_slice();

   let data_hash = holo_hash::encode::blake2b_256(file_content.as_slice());
   /// Split into chunks
   let chunk_count = (file_meta.len() as f64 / CHUNK_MAX_SIZE as f64).ceil() as usize;
   let mut chunk_list= Vec::with_capacity(chunk_count);
   let mut o: usize = 0;
   for _i in 0..chunk_count {
      let chunk_content = &file_content[o..CHUNK_MAX_SIZE];
      chunk_list.push(chunk_content);
      o += CHUNK_MAX_SIZE;
   }
   /// Write each chunk
   let mut i = 0;
   let mut chunk_hh_list = Vec::new();
   for chunk in chunk_list {
      let chunk_hash = holo_hash::encode::blake2b_256(chunk);
      let chunk_input = FileChunk {
         data_hash: String::from_utf8(chunk_hash).unwrap(),
         chunk_index: i,
         chunk: String::from_utf8(chunk.to_owned()).unwrap(),
      };
      let hh = snapmail_write_chunk(conductor.clone(), WriteChunkInput(chunk_input)).unwrap();
      chunk_hh_list.push(hh);
      i += 1;
   }

   /// Write Manifest
   let input = WriteManifestInput {
      data_hash: String::from_utf8(data_hash).unwrap(),
      filename,
      filetype: filepath.extension().unwrap().to_string_lossy().to_string(),
      orig_filesize: file_meta.len() as usize,
      chunks: chunk_hh_list,
   };
   let res = snapmail_write_manifest(conductor, input).unwrap();
   Ok(res)
}