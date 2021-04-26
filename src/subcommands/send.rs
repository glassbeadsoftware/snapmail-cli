use crate::{
   //globals::*,
   utils::*,
};
use structopt::StructOpt;
use holochain::conductor::ConductorHandle;
use snapmail::mail::*;

#[derive(Debug, StructOpt, Clone)]
pub struct SendCommand {
   #[structopt(long)]
   to: String,
   #[structopt(short, long)]
   subject: String,
   #[structopt(short, long)]
   message: String,
}


impl SendCommand {
   ///
   pub fn run(self, conductor: ConductorHandle) {
      let first = stoh(self.to);
      let mail = SendMailInput {
         subject: self.subject,
         payload: self.message,
         to: vec![first],
         cc: vec![],
          bcc: vec![],
         manifest_address_list: vec![],
      };
      let send_count = mail.to.len() + mail.cc.len() + mail.bcc.len();
      let output = snapmail_send_mail(conductor, mail).unwrap();
      let pending_count = output.to_pendings.len() + output.cc_pendings.len() + output.bcc_pendings.len();
      msg!("Send done: {:?}", output.outmail);
      msg!("   - pendings: {} / {}", pending_count, send_count);
   }
}