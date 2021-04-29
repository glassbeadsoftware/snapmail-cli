
use crate::{
   utils::*,
   conductor::*,
};
use holochain_types::dna::*;
use snapmail::mail::entries::Mail;
use snapmail::handle::GetAllHandlesOutput;
use snapmail::mail::*;
use snapmail::handle::*;
use chrono::{DateTime, TimeZone, Local};

fn print_mail(handle_list: &GetAllHandlesOutput, mail: Mail, from: String, bcc: Vec<AgentPubKey>) {
   /// Get all CCs
   let mut cc_all = String::new();
   for cc in mail.cc.iter() {
      cc_all = format!("{}, {}", cc_all, get_name(&handle_list, &cc).unwrap());
   }
   /// Get all BCCs
   let mut bcc_all = String::new();
   for bcc in bcc.iter() {
      bcc_all = format!("{}, {}",bcc_all, get_name(&handle_list, &bcc).unwrap());
   }
   ///
   let date: DateTime<Local> = Local.timestamp(mail.date_sent as i64, 0);
   /// Print
   msg!("     Subject: {}", mail.subject);
   msg!("        From: {}", from);
   msg!("          CC: {}", cc_all);
   msg!("         BCC: {}", bcc_all);
   msg!("        Date: {}", date);
   msg!(" Attachments: {}", mail.attachments.len());
   for attachment in mail.attachments.iter() {
      msg!("            - ({} KiB) {} | {}", attachment.orig_filesize, attachment.filename, attachment.manifest_eh);
   }
   msg!("\n\n{}\n", mail.payload);
}

///
pub async fn open(uid: String, hh: HeaderHash) -> anyhow::Result<()> {
   let conductor = start_conductor(uid).await;
   let maybe_mail = snapmail_get_mail(conductor.clone(), hh.clone())?;
   if let Some(mail) = maybe_mail.0 {
      let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
      msg!(" - mail: {:?}", mail);
      match mail {
         Ok(inmail) => {
            let from = get_name(&handle_list, &inmail.from).unwrap();
            print_mail(&handle_list, inmail.mail, from, vec![]);
            msg!("Acknowledging...");
            let maybe_hash = snapmail_acknowledge_mail(conductor, hh);
            if let Ok(hash) = maybe_hash {
               msg!("Acknowledged: {}", hash);
            } else { msg!("Done");}
         },
         Err(outmail) => {
            print_mail(&handle_list, outmail.mail, "<myself>".to_string(), outmail.bcc);
         },
      }
   } else {
      msg!(" !! No mail found at this hash");
   }
   Ok(())
}