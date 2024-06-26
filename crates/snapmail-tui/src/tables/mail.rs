use tui::{
   widgets::TableState,
};
use chrono::{DateTime, TimeZone, Local};
use snapmail::mail::entries::*;
use std::collections::HashMap;
use holochain_types::dna::*;
use crate::snapmail_chain::*;

pub struct MailTable {
   pub state: TableState,
   pub items: Vec<Vec<String>>,
   pub mail_index_map: HashMap<usize, ActionHash>,
}

impl MailTable {
   pub fn new(mails: Vec<MailItem>, handle_map: &HashMap<AgentPubKey, String>, width: usize) -> MailTable {
      /// Sort mails
      let mut sorted_mails = mails.clone();
      sorted_mails.sort_by(|a, b| {a.date.cmp(&b.date)});
      /// Convert each mail item to table item
      let mut i = 0;
      let mut mail_index_map = HashMap::new();
      let items: Vec<Vec<String>> = sorted_mails
         .iter()
         .map(|mail| {
         mail_index_map.insert(i, mail.address.clone());
         i+= 1;
         let status = get_status_string(mail);
         /// From
         let mut username = get_username(mail, handle_map.clone());
         username = if username.len() > 20 {
            let base = username[0..17].to_string();
            base + "..."
         } else { username.clone() };
         /// Subject
         let subject: String = if mail.mail.subject.len() > 28 {
            let base = mail.mail.subject[0..25].to_string();
            base + "..."
         } else { mail.mail.subject.clone() };
         /// Content
         let first_line = mail.mail.payload.lines().next().unwrap_or("");
         let mut concat = first_line.to_string();
         for line in mail.mail.payload.lines().next().iter() {
            concat = format!("{} {}", concat, line);
         }
         let width = std::cmp::max(4, width);
         let message: String = if concat.len() > width {
            let base = concat[0..width - 3].to_string();
            base + "..."
         } else { concat.to_string() };
         /// Date
         let date: DateTime<Local> = Local.timestamp(mail.mail.date_sent as i64, 0);
         let date_str = format!("{}", date.format("%H:%M %Y-%m-%d"));

         let mut row: Vec<String> = Vec::new();
         //row.push(format!("{}", mail.address));
         row.push(status);
         row.push(username);
         row.push(subject);
         row.push(message);
         row.push(date_str);
         row

      }).collect();
      MailTable {
         state: TableState::default(),
         items,
         mail_index_map,
      }
   }

   pub fn next(&mut self) {
      let i = match self.state.selected() {
         Some(i) => {
            if i >= self.items.len() - 1 {
               0
            } else {
               i + 1
            }
         }
         None => 0,
      };
      self.state.select(Some(i));
   }

   pub fn previous(&mut self) {
      let i = match self.state.selected() {
         Some(i) => {
            if i == 0 {
               self.items.len() - 1
            } else {
               i - 1
            }
         }
         None => 0,
      };
      self.state.select(Some(i));
   }

   pub fn get_mail_text(&self, index: usize, chain: &SnapmailChain) -> String {
      let hh = self.mail_index_map.get(&index).unwrap();
      let item = chain.mail_map.get(hh).unwrap();
      let maybe_author = chain.handle_map.get(&item.author);
      let author = match maybe_author {
         Some(a) => a,
         None => "<unknown>",
      };
      let date: DateTime<Local> = Local.timestamp(item.mail.date_sent as i64, 0);
      let date_str = format!("{}", date.format("%H:%M %Y-%m-%d"));
      /// TO line
      let mut to_line = "     To:".to_string();
      if let Some(first) = item.mail.to.first() {
         to_line += &format!(" {}", chain.handle_map.get(&first).unwrap());
      }
      for to in item.mail.to.iter().skip(1) {
         to_line +=  &format!(", {}", chain.handle_map.get(&to).unwrap());
      }
      /// CC line
      let mut cc_line = "     Cc:".to_string();
      if let Some(first) = item.mail.cc.first() {
         cc_line += &format!(" {}", chain.handle_map.get(&first).unwrap());
      }
      for to in item.mail.cc.iter().skip(1) {
         cc_line +=  &format!(", {}", chain.handle_map.get(&to).unwrap());
      }
      /// BCC line
      let mut bcc_line = "    Bcc:".to_string();
      if let Some(first) = item.bcc.first() {
         bcc_line += &format!(" {}", chain.handle_map.get(&first).unwrap());
      }
      for to in item.bcc.iter().skip(1) {
         bcc_line +=  &format!(", {}", chain.handle_map.get(&to).unwrap());
      }
      /// Subject & From
      let mut text = format!("Subject: {}\n", item.mail.subject);
      text += &format!("   From: {} - {}\n", author, date_str);

      /// Add recepîents if there are some
      if to_line.len() > 9 {
         text += &format!("{}\n", to_line);
      }
      if cc_line.len() > 9 {
         text += &format!("{}\n", cc_line);
      }
      if bcc_line.len() > 9 {
         text += &format!("{}\n", bcc_line);
      }

      // Payload
      text += &format!("\n{}", &item.mail.payload);
      text
   }

}

///
fn get_status_string(mail: &MailItem) -> String {
   let char =
   match &mail.state {
      MailState::In(in_state) => {
         match in_state {
            InMailState::Unacknowledged => "··",
            InMailState::AckUnsent => "AU",
            InMailState::AckPending => "AP",
            InMailState::AckDelivered => "OK",
            InMailState::Deleted => "XX",
         }
      },
      MailState::Out(out_state) => {
         match &out_state {
            OutMailState::Unsent => ">>",
            OutMailState::AllSent => "::",
            OutMailState::AllReceived => "vv",
            OutMailState::AllAcknowledged => "OK",
            OutMailState::Deleted => "XX",
         }
      }
   };
   format!("[{}]", char)
}

///
fn get_username(mail: &MailItem, handle_map: HashMap<AgentPubKey, String>) -> String {
   let username: String = handle_map.get(&mail.author).unwrap_or(&"<unknown>".to_string()).to_owned();
   match &mail.state {
      MailState::In(_in_state) => {
         username
      },
      MailState::Out(_out_state) => {
         let recepient_key =
         if !mail.mail.to.is_empty() {
            mail.mail.to[0].clone()
         } else {
            if !mail.mail.cc.is_empty() {
               mail.mail.cc[0].clone()
            } else {
               assert!(!mail.bcc.is_empty());
               mail.bcc[0].clone()
            }
         };
         let recepient = handle_map.get(&recepient_key).unwrap_or(&"<unknown>".to_string()).to_owned();
         format!("To: {}", recepient)
      },
   }
}