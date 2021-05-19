use std::string::ToString;
use snapmail_common::{
   attachment::*,
   globals::*,
};
use crate::{
      menu::*,
      tables::{MailTable, ContactsTable, AttachmentsTable},
      snapmail_chain::SnapmailChain,
};
use tui::style::Color;
use holochain_types::dna::*;
use snapmail::{
   api_error::*,
   mail::*,
   mail::entries::*,
   handle::*,
};
use holochain::conductor::ConductorHandle;
use std::path::PathBuf;

#[derive(AsStaticStr, ToString, Copy, Clone, Debug, PartialEq)]
pub enum InputMode {
   Normal,
   Editing,
   Scrolling,
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
   DownloadFolder,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppCommand {
   None,
   SendMail,
   AcknowledgeMail(HeaderHash),
   DeleteMail,
   UpdateHandle,
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

   pub content_width: usize,
   pub scroll_y: u16,

   pub download_folder: PathBuf,

   pub command: AppCommand,

   pub attachments_table: AttachmentsTable,

   pub active_menu_item: TopMenuItem,
   pub active_folder_item: FolderItem,

   pub mail_table: MailTable,

   pub contacts_table: ContactsTable,
   pub write_subject: String,
   pub write_content: String,
   // pub write_attachments: Vec<PathBuf>,
   pub write_attachment: String,
   pub active_write_block: WriteBlock,

   pub peer_count: usize,

   /// - Debug
   pub frame_count: u32,

   /// History of recorded messages
   pub feedback_index: u32,
   pub feedbacks: Vec<(String, Color, Color)>,
}

impl App {
   ///
   pub fn new(sid: String, chain: &SnapmailChain) -> App {
      let mail_list = filter_chain(&chain, FolderItem::Inbox);
      let mail_table = MailTable::new(mail_list, &chain.handle_map, 12);
      let contacts_table = ContactsTable::new(&chain.handle_map);

      /// - Get UID
      let path = CONFIG_PATH.as_path().join(sid.clone());
      let app_filepath = path.join(APP_CONFIG_FILENAME);
      let uid = std::fs::read_to_string(app_filepath)
         .unwrap_or("test-network".to_string());

      let dl_filepath = path.join(APP_DL_CONFIG_FILENAME);
      let mut download_folder = std::env::current_dir().unwrap();
       if let Ok(s) = std::fs::read_to_string(dl_filepath) {
          download_folder = PathBuf::from(s);
       }

      App {
         input: String::new(),
         input_mode: InputMode::Normal,
         input_variable: InputVariable::Content,
         feedback_index: 0,
         feedbacks: Vec::new(),
         command: AppCommand::None,
         attachments_table: AttachmentsTable::new(Vec::new()),
         frame_count: 0,
         active_menu_item: TopMenuItem::View,
         active_folder_item: FolderItem::Inbox,
         sid,
         uid,
         peer_count: 0,
         content_width: 12,
         download_folder,
         scroll_y: 0,
         mail_table,
         contacts_table,
         active_write_block: WriteBlock::None,
         write_subject: String::new(),
         write_content: String::new(),
         write_attachment: String::new(), //std::env::current_dir().unwrap().into_os_string().into_string().unwrap(),
         //write_attachments: Vec::new(),
      }
   }

   ///
   pub fn resize_width(&mut self, new_width: u16, chain: &SnapmailChain) {
      self.content_width = new_width as usize;
      let mail_list = filter_chain(&chain, self.active_folder_item);
      self.mail_table = MailTable::new(mail_list, &chain.handle_map, self.content_width);
   }

   ///
   pub fn try_download(&mut self, conductor: ConductorHandle, index: usize) {
      let maybe_info = self.attachments_table.manifest_index_map.get(&index);
      if let None = maybe_info {
         let msg = format!("No attachment at index {}", index);
         self.feedback_ext(&msg, Color::Yellow, Color::Black);
         return;
      }
      let info = maybe_info.unwrap();
      let maybe_path = get_attachment(
         conductor.clone(),
         info.manifest_eh.clone(),
         self.download_folder.clone(),
      );
      if let Ok(path) = maybe_path {
         let msg = format!("File written at: {:?}", path);
         self.feedback_ext(&msg, Color::Green, Color::Black);
      } else {
         let msg = format!("Failed written file ({})", info.manifest_eh.clone());
         self.feedback_ext(&msg, Color::Red, Color::Black);
      }
   }

   ///
   fn delete_mail(&mut self, conductor: ConductorHandle) {
      if let Some(index) = self.mail_table.state.selected() {
         let hh = self.mail_table.mail_index_map.get(&index).unwrap().clone();
         let res = snapmail_delete_mail(conductor, hh);
         if let Ok(output) = res {
            if let Some(hh2) = output.0 {
               let msg = &format!("Deleted mail {}", hh2);
               self.feedback_ext(&msg, Color::Green, Color::Black);
               return;
            }
         }
      }
      self.feedback_ext("Could not delete selected mail", Color::Yellow, Color::Black);
   }

   ///
   pub fn next_mail(&mut self, chain: &SnapmailChain) {
      self.mail_table.next();
      if let Some(index) = self.mail_table.state.selected() {
         let hh = self.mail_table.mail_index_map.get(&index).unwrap().clone();
         self.command = AppCommand::AcknowledgeMail(hh.clone());
         self.feedback(&format!("Reading mail: {}", hh));
         /// Attachment
         let item = chain.mail_map.get(&hh).unwrap();
         self.attachments_table = AttachmentsTable::new(item.mail.attachments.clone());
         if self.attachments_table.manifest_index_map.len() > 9 {
            let msg = format!("Max attachments exceeded({})", self.attachments_table.manifest_index_map.len());
            self.feedback_ext(&msg, Color::Red, Color::Black);
         }
      }
   }

   ///
   pub fn previous_mail(&mut self, chain: &SnapmailChain) {
      self.mail_table.previous();
      if let Some(index) = self.mail_table.state.selected() {
         let hh = self.mail_table.mail_index_map.get(&index).unwrap().clone();
         self.command = AppCommand::AcknowledgeMail(hh.clone());
         self.feedback(&format!("Reading mail: {}", hh));
         /// Attachment
         let item = chain.mail_map.get(&hh).unwrap();
         self.attachments_table = AttachmentsTable::new(item.mail.attachments.clone());
         if self.attachments_table.manifest_index_map.len() > 9 {
            let msg = format!("Max attachments exceeded({})", self.attachments_table.manifest_index_map.len());
            self.feedback_ext(&msg, Color::Red, Color::Black);
         }
      }
   }

   ///
   pub fn update_data(&mut self, chain: &SnapmailChain) {
      /// Update mail table && keep current selection
      let mail_list = filter_chain(&chain, self.active_folder_item);
      let maybe_hh = if let Some(i) = self.mail_table.state.selected() {
         Some(self.mail_table.mail_index_map.get(&i).unwrap().clone())
      } else { None };
      self.mail_table = MailTable::new(mail_list, &chain.handle_map, self.content_width);
      if let Some(hh) = maybe_hh {
         for (index, current) in &self.mail_table.mail_index_map {
            if *current == hh {
               self.mail_table.state.select(Some(*index));
            }
         }
      }
      // Update contacts table
      self.contacts_table = ContactsTable::new(&chain.handle_map);
   }

   pub fn feedback(&mut self, msg: &str) {
      self.feedback_ext(msg, Color::White, Color::Black);
   }

   pub fn feedback_ext(&mut self, msg: &str, fg: Color, bg: Color) {
      if msg.is_empty() {
         return;
      }
      self.feedbacks.push((msg.to_string(), fg, bg));
      self.feedback_index = self.feedbacks.len() as u32 - 1;
   }

   /// Returns true if chain should be updated
   pub fn process_command(&mut self, conductor: ConductorHandle, chain: &SnapmailChain) -> bool {
      let mut can_update_chain = false;
      match &self.command {
         AppCommand::SendMail => {
            let res = self.send_mail(conductor.clone(), chain);
            match res {
               Err(e) => self.feedback_ext(&format!("Send mail failed: {}", e), Color::Black, Color::Red),
               Ok(_) => can_update_chain = true,
            }
         },
         AppCommand::AcknowledgeMail(hh) => {
            if let Some(mail_item) = chain.mail_map.get(hh) {
               match mail_item.state {
                  MailState::In(InMailState::Incoming) |
                  MailState::In(InMailState::Arrived) => {
                     let res = snapmail_acknowledge_mail(conductor, hh.clone());
                     if let Ok(_entry_hash) = res {
                        let msg = format!("Mail acknowledged: {}", hh);
                        self.feedback_ext(&msg, Color::Green, Color::Black);
                        can_update_chain = true;
                     }
                  }
                  _ => {},
               }
            }
         },
         AppCommand::DeleteMail => {
            self.delete_mail(conductor);
            can_update_chain = true;
         },
         AppCommand::UpdateHandle => {
            let res = snapmail_set_handle(conductor.clone(), self.input.clone());
            match res {
               Err(e) => self.feedback_ext(&format!("Set handle failed: {}", e), Color::Black, Color::Red),
               Ok(hash) => {
                  can_update_chain = true;
                  self.feedback_ext(&format!("New Handle entry hash: {}", hash.to_string()), Color::Green, Color::Black);
               }
            }
         },
         _ => {},
      }
      self.command = AppCommand::None;
      can_update_chain
   }

   ///
   pub fn update_active_folder(&mut self, chain: &SnapmailChain, folder_item: FolderItem) {
      if self.active_menu_item == TopMenuItem::View {
         self.active_folder_item = folder_item;
         let mail_list = filter_chain(&chain, self.active_folder_item);
         self.mail_table = MailTable::new(mail_list, &chain.handle_map, self.content_width);
      }
   }

   ///
   pub fn set_write_block(&mut self, block: WriteBlock) {
      if block == WriteBlock::None {
         self.save_input();
         self.input_mode = InputMode::Normal;
         self.active_write_block = WriteBlock::None;
      }
      while self.active_write_block != block {
         self.toggle_write_block();
      }
   }

   ///
   pub fn save_input(&mut self) {
      match self.active_write_block {
         WriteBlock::Subject => {
            self.write_subject = self.input.clone();
         }
         WriteBlock::Content => {
            self.write_content = self.input.clone();
         },
         WriteBlock::Attachments => {
            self.write_attachment = self.input.clone();
         },
         WriteBlock::Contacts => {
            self.contacts_table.state.select(None);
         }
         _ => {},
      }
   }


   ///
   pub fn toggle_write_block(&mut self) {
      self.save_input();
      self.input_mode = InputMode::Editing;
      self.active_write_block = match self.active_write_block {
         WriteBlock::Subject => {
            self.input = self.write_content.clone();
            self.input_variable = InputVariable::Content;
            WriteBlock::Content
         }
         WriteBlock::Content => {
            self.input = self.write_attachment.clone();
            self.input_variable = InputVariable::Attachment;
            WriteBlock::Attachments
         },
         WriteBlock::Attachments => {
            self.input = String::new();
            if let None = self.contacts_table.state.selected() {
               self.contacts_table.state.select(Some(0));
            }
            WriteBlock::Contacts
         },
         WriteBlock::None | WriteBlock::Contacts => {
            self.input = self.write_subject.clone();
            self.input_variable = InputVariable::Subject;
            WriteBlock::Subject
         },
      }
   }

   ///
   pub fn send_mail(&mut self, conductor: ConductorHandle, chain: &SnapmailChain) -> SnapmailApiResult<()> {
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
      if 0 == to_list.len() + cc_list.len() + bcc_list.len() {
         self.feedback_ext("Send aborted: No recepient selected", Color::Yellow, Color::Black);
         return Err(SnapmailApiError::Unique("No recepient selected".to_string()));
      }

      /// Form attachment list
      let mut manifest_address_list: Vec<HeaderHash> = Vec::new();
      // for attachment in &self.write_attachments {
      //    let maybe_hh = write_attachment(conductor.clone(), attachment.clone());
      //    if let Ok(hh) = maybe_hh {
      //       manifest_address_list.push(hh);
      //    }
      // }

      if !self.write_attachment.is_empty() {
         let path = PathBuf::from(self.write_attachment.clone());
         let maybe_hh = write_attachment(conductor.clone(), path);
         match maybe_hh {
            Ok(hh) => manifest_address_list.push(hh),
            Err(e) => {
               self.feedback_ext("Send Aborted. Failed loading attachment file", Color::Black, Color::Red);
               return Err(SnapmailApiError::IoError(e));
            }
         }
      }

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
      let output = snapmail_send_mail(conductor, mail)?;
      /// Show results
      let pending_count = output.to_pendings.len() + output.cc_pendings.len() + output.bcc_pendings.len();
      let message = format!("Mail sent. Pendings:  {} / {} ({})", pending_count, send_count, output.outmail);
      let fg_color =  if pending_count == 0 {
         Color::Green
      } else if pending_count == send_count {
         Color::LightMagenta
      }  else {
         Color::Yellow
      };

      self.feedback_ext(&message, fg_color, Color::Black);

      // Erase State
      self.input = String::new();
      self.write_content = String::new();
      self.write_attachment = String::new(); //std::env::current_dir().unwrap().into_os_string().into_string().unwrap();
      //self.write_attachments = Vec::new();
      self.write_subject = String::new();
      self.contacts_table = ContactsTable::new(&chain.handle_map);
      Ok(())
   }
}

///
pub fn filter_chain(chain: &SnapmailChain, folder: FolderItem) -> Vec<MailItem> {
   let mut res = Vec::new();
   match folder {
      FolderItem::Inbox => {
         for item in chain.mail_map.values() {
            if let MailState::In(state) = &item.state {
               if state != &InMailState::Deleted {
                  res.push(item.clone());
               }
            }
         }
      }
      FolderItem::Sent => {
         for item in chain.mail_map.values() {
            if let MailState::Out(state) = &item.state {
               if state != &OutMailState::Deleted {
                  res.push(item.clone());
               }
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