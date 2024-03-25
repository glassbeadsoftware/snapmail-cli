use snapmail_common::{
   utils::*,
   conductor::*,
};
use holochain_types::dna::*;
use snapmail::mail::entries::Mail;
use snapmail::handle::HandleItem;
use snapmail::mail::*;
use snapmail::handle::*;
use chrono::{DateTime, TimeZone, Local};
use tokio::time::{sleep, Duration};

fn print_mail(handle_list: &Vec<HandleItem>, mail: Mail, from: String, bcc: Vec<AgentPubKey>) {
   /// Get all CCs
   let mut cc_all = String::new();
   for cc in mail.cc.iter() {
      let name = get_name(&handle_list, &cc)
         .expect("Should have found cc handle in DHT");
      cc_all = format!("{}, {}", cc_all, name);
   }
   /// Get all BCCs
   let mut bcc_all = String::new();
   for bcc in bcc.iter() {
      let name = get_name(&handle_list, &bcc)
         .expect("Should have found bcc handle in DHT");
      bcc_all = format!("{}, {}",bcc_all, name);
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
pub async fn open(uid: String, ah: ActionHash) -> anyhow::Result<()> {
   let conductor = start_conductor(uid).await;
   let maybe_mail = snapmail_get_mail(conductor.clone(), ah.clone())?;
   if let Some(mail) = maybe_mail.0 {
      let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
      msg!(" - mail: {:?}", mail);
      match mail {
         Ok(inmail) => {
            let from = get_name(&handle_list, &inmail.from)
               .ok_or(anyhow::Error::msg("Handle not found"))?;
            print_mail(&handle_list, inmail.mail, from, vec![]);
            msg!("Acknowledging...");
            let maybe_hash = snapmail_acknowledge_mail(conductor.clone(), ah);
            match maybe_hash {
               Ok(hash) => msg!("Acknowledged: {}", hash),
               Err(e) => msg!("Done - {:?}", e),
            }
         },
         Err(outmail) => {
            print_mail(&handle_list, outmail.mail, "<myself>".to_string(), outmail.bcc);
         },
      }
   } else {
      msg!(" !! No mail found at this hash");
   }
   sleep(Duration::from_millis(20 * 1000)).await; // conductor.shutdown() is broken
   conductor.shutdown();
   Ok(())
}

///
pub async fn get_status(uid: String, ah: ActionHash) -> anyhow::Result<()> {
   let conductor = start_conductor(uid).await;
   let state = snapmail_get_outmail_state(conductor.clone(), ah.clone())?;
   msg!(" Outmail state: {:?}", state);
   let map = snapmail_get_outmail_delivery_state(conductor.clone(), ah.clone())?;
   for pair in map.iter() {
      msg!(" - {:?}", pair);
   }
   Ok(())
}