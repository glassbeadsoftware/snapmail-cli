use std::string::ToString;
use crate::{
   attachment::*,
   globals::*,
   tui2::{
      menu::*, MailTable, ContactsTable, SnapmailChain,
   }
};
use holochain_types::dna::*;
use snapmail::{
   mail::*,
   mail::entries::*,
   //file::*,
   //handle::*,
};
use holochain::conductor::ConductorHandle;
use std::path::PathBuf;

#[derive(AsStaticStr, ToString, Copy, Clone, Debug, PartialEq)]
pub enum InputMode {
   Normal,
   Editing,
}

#[derive(AsStaticStr, ToString, Copy, Clone, Debug, PartialEq)]
pub enum InputVariable {
   BoostrapUrl,
   ProxyUrl,
   Handle,
   Uid,
   Content,
   Attachment,
   Subject,
}

/// App holds the state of the application
pub struct App {
   /// Current value of the input box
   pub input: String,
   /// Current input mode
   pub input_mode: InputMode,
   /// Current settings to change
   pub input_variable: InputVariable,

   pub sid: String,
   pub uid: String,

   pub active_menu_item: TopMenuItem,
   pub active_folder_item: FolderItem,

   pub mail_table: MailTable,

   pub contacts_table: ContactsTable,
   pub write_subject: String,
   pub write_content: String,
   // pub write_attachments: Vec<PathBuf>,
   pub write_attachment: String,
   pub active_write_block: WriteBlock,

   /// - Debug
   pub frame_count: u32,
   /// History of recorded messages
   pub messages: Vec<String>,
}

impl App {
   ///
   pub fn new(sid: String, chain: &SnapmailChain) -> App {
      let mail_list = filter_chain(&chain, FolderItem::Inbox);
      let mail_table = MailTable::new(mail_list, &chain.handle_map);
      let contacts_table = ContactsTable::new(&chain.handle_map);

      /// - Get UID
      let path = CONFIG_PATH.as_path().join(sid.clone());
      let app_filepath = path.join(APP_CONFIG_FILENAME);
      let uid = std::fs::read_to_string(app_filepath)
         .unwrap_or("test-network".to_string());

      App {
         input: String::new(),
         input_mode: InputMode::Normal,
         input_variable: InputVariable::Content,
         messages: vec!["Welcome to Snapmail".to_string()],

         frame_count: 0,
         active_menu_item: TopMenuItem::View,
         active_folder_item: FolderItem::Inbox,
         sid,
         uid,
         mail_table,
         contacts_table,
         active_write_block: WriteBlock::Contacts,
         write_subject: String::new(),
         write_content: String::new(),
         write_attachment: String::new(),
         //write_attachments: Vec::new(),
      }
   }

   ///
   pub fn update_active_folder(&mut self, chain: &SnapmailChain, folder_item: FolderItem) {
      if self.active_menu_item == TopMenuItem::View {
         self.active_folder_item = folder_item;
         let mail_list = filter_chain(&chain, self.active_folder_item);
         self.mail_table = MailTable::new(mail_list, &chain.handle_map);
      }
   }

   ///
   pub fn set_write_block(&mut self, block: WriteBlock) {
      while self.active_write_block != block {
         self.toggle_write_block();
      }
   }

   ///
   pub fn toggle_write_block(&mut self) {
      self.active_write_block = match self.active_write_block {
         WriteBlock::Subject => {
            self.write_subject = self.input.clone();
            self.input = self.write_content.clone();
            self.input_variable = InputVariable::Content;
            WriteBlock::Content
         }
         WriteBlock::Content => {
            self.write_content = self.input.clone();
            self.input = self.write_attachment.clone();
            self.input_variable = InputVariable::Attachment;
            WriteBlock::Attachments
         },
         WriteBlock::Attachments => {
            self.write_attachment = self.input.clone();
            self.input = String::new();
            self.input_mode = InputMode::Normal;
            self.contacts_table.state.select(Some(1));
            WriteBlock::Contacts
         },
         WriteBlock::Contacts => {
            self.contacts_table.state.select(None);
            self.input = self.write_subject.clone();
            self.input_mode = InputMode::Editing;
            self.input_variable = InputVariable::Subject;
            WriteBlock::Subject
         },
      }
   }

   ///
   pub fn send_mail(&mut self, conductor: ConductorHandle, chain: &SnapmailChain) {
      /// Form recepient lists from ContactsTable
      let mut to_list: Vec<AgentPubKey> = Vec::new();
      let mut cc_list: Vec<AgentPubKey> = Vec::new();
      let mut bcc_list: Vec<AgentPubKey> = Vec::new();
      let mut i: i32 = -1;
      for contact_item in &self.contacts_table.items {
         i += 1;
         match contact_item[0].as_str() {
            " to " => {
               to_list.push(self.contacts_table.agent_index_map.get(&(i as usize)).unwrap().clone());
            },
            " cc " =>  {
               cc_list.push(self.contacts_table.agent_index_map.get(&(i as usize)).unwrap().clone());
            },
            " bcc " =>  {
               bcc_list.push(self.contacts_table.agent_index_map.get(&(i as usize)).unwrap().clone());
            },
            _ => { } ,
         }
      }
      /// Form attachment list
      let mut manifest_address_list: Vec<HeaderHash> = Vec::new();
      // for attachment in &self.write_attachments {
      //    let maybe_hh = write_attachment(conductor.clone(), attachment.clone());
      //    if let Ok(hh) = maybe_hh {
      //       manifest_address_list.push(hh);
      //    }
      // }

      //if let Ok(path) = PathBuf::from(self.write_attachment.clone()) {
      let path = PathBuf::from(self.write_attachment.clone());
         let maybe_hh = write_attachment(conductor.clone(), path);
         if let Ok(hh) = maybe_hh {
            manifest_address_list.push(hh);
         }
      //}
      /// Form MailInput
      let mail = SendMailInput {
         subject: self.write_subject.clone(),
         payload: self.write_content.clone(),
         to: to_list,
         cc: cc_list,
         bcc: bcc_list,
         manifest_address_list,
      };
      let send_count = mail.to.len() + mail.cc.len() + mail.bcc.len();
      /// Send
      let output = snapmail_send_mail(conductor, mail).unwrap();
      /// Show results
      let pending_count = output.to_pendings.len() + output.cc_pendings.len() + output.bcc_pendings.len();
      let message = format!("Send done ({} / {}): {:?}", pending_count, send_count, output.outmail);
      self.messages.insert(0, message);

      // Erase State
      self.input = String::new();
      self.write_content = String::new();
      self.write_attachment = String::new();
      //self.write_attachments = Vec::new();
      self.write_subject = String::new();
      self.contacts_table = ContactsTable::new(&chain.handle_map);
   }

}


///
pub fn filter_chain(chain: &SnapmailChain, folder: FolderItem) -> Vec<MailItem> {
   let mut res = Vec::new();
   match folder {
      FolderItem::Inbox => {
         for item in chain.mail_map.values() {
            if let MailState::In(_) = item.state {
               res.push(item.clone());
            }
         }
      }
      FolderItem::Sent => {
         for item in chain.mail_map.values() {
            if let MailState::Out(_) = item.state {
               res.push(item.clone());
            }
         }
      }
      FolderItem::All => {
         for item in chain.mail_map.values() {
            res.push(item.clone());
         }
      }
      FolderItem::Trash => {
         for item in chain.mail_map.values() {
            match &item.state {
               MailState::Out(state) => {
                  if let OutMailState::Deleted = state {
                     res.push(item.clone());
                  }
               }
               MailState::In(state) => {
                  if let InMailState::Deleted = state {
                     res.push(item.clone());
                  }
               }
            }
         }
      }
   }
   res
}