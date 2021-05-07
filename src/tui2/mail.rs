use tui::{
   widgets::TableState,
};
use chrono::{DateTime, TimeZone, Local};
use snapmail::mail::entries::*;
use std::collections::HashMap;
use holochain_types::dna::*;

pub struct MailTable<'a> {
   pub state: TableState,
   pub items: Vec<Vec<&'a str>>,
}
impl<'a> MailTable<'a> {
   pub fn new(mails: Vec<MailItem>, handle_map: &HashMap<AgentPubKey, String>) -> MailTable<'a> {
      let items: Vec<Vec<&str>> = mails.iter().map(|mail| {
         let mut row: Vec<&str> = Vec::new();
         row.push(&format!("{}", mail.address));
         let username = get_username(mail, handle_map).as_str();
         row.push(username);
         row.push(mail.mail.subject.as_str());
         let date: DateTime<Local> = Local.timestamp(mail.mail.date_sent as i64, 0);
         let date_str = format!("{}", date);
         row.push(date_str.as_str());
         //let status = format!("");
         row.push(get_status_string(mail).as_str());
         row

      }).collect();
      MailTable {
         state: TableState::default(),
         items,
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
}

///
fn get_status_string(mail: &MailItem) -> String {
   let char =
   match &mail.state {
      MailState::In(in_state) => {
         match in_state {
            InMailState::Incoming => ">>",
            InMailState::Arrived => "vv",
            InMailState::Acknowledged => "A-",
            InMailState::AckReceived => "A+",
            InMailState::Deleted => "XX",
         }
      },
      MailState::Out(out_state) => {
         match &out_state {
            OutMailState::Pending => "--",
            OutMailState::PartiallyArrived_NoAcknowledgement => "::",
            OutMailState::PartiallyArrived_PartiallyAcknowledged => ":A",
            OutMailState::Arrived_NoAcknowledgement => "vv",
            OutMailState::Arrived_PartiallyAcknowledged => "vA",
            OutMailState::Received => "OK",
            OutMailState::Deleted => "XX",
         }
      }
   };
   format!("[{}]", char)
}

///
fn get_username(mail: &MailItem, handle_map: HashMap<AgentPubKey, String>) -> String {
   let username: String = handle_map.get(&mail.author).unwrap_or(&"<unknown>".to_string()).to_owned();
   match mail.state {
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