use snapmail_common::{
   utils::*,
   attachment::*,
};
use structopt::StructOpt;
use holochain::conductor::ConductorHandle;
use snapmail::{
   mail::*,
   handle::*,
};
use std::path::PathBuf;
use holochain_types::dna::*;
use tokio::time::{sleep, Duration};

#[derive(Debug, StructOpt, Clone)]
pub struct SendCommand {
   #[structopt(long)]
   /// Recepients
   to: Vec<String>,
   // #[structopt(long)]
   // cc: Option<Vec<String>>,
   #[structopt(short, long)]
   /// Subject of the mail
   subject: String,
   #[structopt(short, long)]
   /// Content to send
   message: String,
   #[structopt(name = "attachment", short, long, parse(from_os_str))]
   /// Add a file atachment
   pub maybe_attachment: Option<PathBuf>,
}


impl SendCommand {
   ///
   pub async fn run(self, conductor: ConductorHandle) -> anyhow::Result<()> {
      // Form "to" list
      let handle_list = snapmail_get_all_handles(conductor.clone(), ())?;
      let mut to_list: Vec<AgentPubKey> = Vec::new();
      for name in self.to.iter() {
         let agent_id = get_agent_id(&handle_list, name)
            .ok_or(anyhow::Error::msg("username not found"))?;
         to_list.push(agent_id);
      }
      // Form attachment list
      let mut manifest_address_list: Vec<ActionHash> = Vec::new();
      if let Some(attachment) = self.maybe_attachment {
         msg!("Reading attachment file: {:?}", attachment);
         let hh = write_attachment(conductor.clone(), attachment)?;
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
      //let send_count = mail.to.len() + mail.cc.len() + mail.bcc.len();
      // Send
      let sent_hh = snapmail_send_mail(conductor.clone(), mail)?;

      sleep(Duration::from_millis(10 * 1000)).await; // conductor.shutdown() is broken

      // Get State
      let mail_state = snapmail_get_outmail_state(conductor.clone(), sent_hh.clone())?;
      // Show results
      //let pending_count = output.to_pendings.len() + output.cc_pendings.len() + output.bcc_pendings.len();
      msg!("Send done: {:?}", sent_hh);
      msg!("   - mail_state: {:?}", mail_state);

      // Wait for post-commit to finish
      sleep(Duration::from_millis(20 * 1000)).await; // conductor.shutdown() is broken
      conductor.shutdown();
      Ok(())
   }
}
