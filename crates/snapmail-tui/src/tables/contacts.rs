use tui::{
   widgets::TableState,
};
use std::collections::HashMap;
use holochain_types::dna::*;

pub struct ContactsTable {
   pub state: TableState,
   pub items: Vec<Vec<String>>,
   pub agent_index_map: HashMap<usize, AgentPubKey>,
}

impl ContactsTable {
   pub fn new(handle_map: &HashMap<AgentPubKey, String>) -> ContactsTable {
      let mut agent_index_map = HashMap::new();
      let mut i = 0;
      let items: Vec<Vec<String>> = handle_map.iter().map(|(key, handle)| {
         agent_index_map.insert(i, key.clone());
         i+= 1;
         let status = String::new();
         let mut row: Vec<String> = Vec::new();
         //row.push(format!("{}", mail.address));
         row.push(status);
         row.push(handle.to_string());
         row

      }).collect();

      let state = TableState::default();
      //state.select(Some(0));

      ContactsTable {
         state,
         items,
         agent_index_map,
      }
   }

   pub fn next(&mut self) {
      if self.items.is_empty() {
         self.state.select(None);
         return;
      }
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
      let selection = match self.state.selected() {
         Some(i) => {
            Some(if i == 0 {
               self.items.len() - 1
            } else {
               i - 1
            })
         }
         None => if self.items.is_empty() {None} else {Some(0)},
      };
      self.state.select(selection);
   }

   pub fn toggle_state(&mut self, index: usize) {
      assert!(!self.items.is_empty());
      let current_state = self.items[index][0].as_str();
      let new_state = match current_state {
         "" => " to ",
         " to " => " cc ",
         " cc " => " bcc ",
         " bcc " => "",
         _ => unreachable!(),
      };
      self.items[index][0] = new_state.to_string();
   }

   pub fn toggle_selected(&mut self) {
      if let Some(index) = self.state.selected() {
         self.toggle_state(index);
      }

   }
}
