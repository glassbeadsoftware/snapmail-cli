use tui::{
   widgets::TableState,
};
use std::collections::HashMap;
use snapmail::mail::entries::*;

///
pub struct AttachmentsTable {
   pub state: TableState,
   pub items: Vec<Vec<String>>,
   pub manifest_index_map: HashMap<usize, AttachmentInfo>,
}

impl AttachmentsTable {
   ///
   pub fn new(attachments: Vec<AttachmentInfo>) -> AttachmentsTable {
      let mut manifest_index_map = HashMap::new();
      let mut i = 0;
      let items: Vec<Vec<String>> = attachments.iter().map(|info| {
         manifest_index_map.insert(i, info.clone());
         i+= 1;
         let _status = String::new();
         let mut row: Vec<String> = Vec::new();
         let index_str = format!(" {}.", i);
         let filesize_str = format!("{} KiB", info.orig_filesize / 1024);
         //row.push(status);
         row.push(index_str);
         row.push(info.filename.to_string());
         row.push(filesize_str);
         row

      }).collect();

      let state = TableState::default();
      //state.select(Some(0));

      AttachmentsTable {
         state,
         items,
         manifest_index_map,
      }
   }

   ///
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

   ///
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

   ///
   pub fn toggle_state(&mut self, index: usize) {
      let current_state = self.items[index][0].as_str();
      let new_state = match current_state {
         "" => " * ",
         " * " => " OK ",
         " OK " => " * ",
         _ => unreachable!(),
      };
      self.items[index][0] = new_state.to_string();
   }

   ///
   pub fn toggle_selected(&mut self) {
      if let Some(index) = self.state.selected() {
         self.toggle_state(index);
      }

   }
}
