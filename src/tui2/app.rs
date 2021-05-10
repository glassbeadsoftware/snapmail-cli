use std::string::ToString;
use crate::{
   globals::*,
   tui2::{
      menu::*, MailTable, ContactsTable, SnapmailChain,
   }
};
use snapmail::mail::entries::*;


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
   Mail,
   Attachment,
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
         .expect("Something went wrong reading APP CONFIG file");

      App {
         input: String::new(),
         input_mode: InputMode::Normal,
         input_variable: InputVariable::Mail,
         messages: vec!["Welcome to Snapmail".to_string()],

         frame_count: 0,
         active_menu_item: TopMenuItem::View,
         active_folder_item: FolderItem::Inbox,
         sid,
         uid,
         mail_table,
         contacts_table,

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