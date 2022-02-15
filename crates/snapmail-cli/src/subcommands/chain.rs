
use holochain::conductor::*;
use holochain_state::source_chain::*;
use colored::*;
use holochain_zome_types::*;
//use holo_hash::*;
use strum::AsStaticRef;

use snapmail::{
   //handle::*,
   EntryKind,
};

///
pub async fn print_chain(conductor: ConductorHandle) {
   let cell_ids = conductor.list_cell_ids(None);

   let vault = conductor.get_authored_env(cell_ids[0].dna_hash()).unwrap();

   let json_dump = dump_state(vault.clone().into(), cell_ids[0].agent_pubkey().clone()).await.unwrap();
   //let json = serde_json::to_string_pretty(&json_dump).unwrap();

   println!(" ====== SOURCE-CHAIN START ===== {}", json_dump.elements.len());
   //println!("source_chain_dump({}) of {:?}", json_dump.elements.len(), agent);

   let mut count = 0;
   for element in &json_dump.elements {
      let str = print_element(&element);
      println!(" {:2}. {}", count, str);
      count += 1;
   }

   println!(" ====== SOURCE-CHAIN END  ===== {}", json_dump.elements.len());
}



///
fn print_element(element: &SourceChainJsonElement) -> String {
   let mut str = format!("{:?} ", element.header.header_type());
   // let mut str = format!("({}) ", element.header_address);

   // if (element.header.header_type() == HeaderType::CreateLink) {
   //    str += &format!(" '{:?}'", element.header.tag());
   // }

   match &element.header {
      Header::CreateLink(create_link) => {
         // let s = std::str::from_utf8(&create_link.tag.0).unwrap();
         let s = String::from_utf8_lossy(&create_link.tag.0).to_string();
         str += &format!("'{:.20}'", s).yellow().to_string();
      },
      Header::Create(create_entry) => {
         let mut s = String::new();
         match &create_entry.entry_type {
            EntryType::App(app_entry_type) => {
               let entry_kind: &'static str = EntryKind::from_index(&app_entry_type.id()).as_static();
               s += "AppEntry ";
               s += &format!("'{}'", entry_kind);
               if app_entry_type.visibility() == &EntryVisibility::Public {
                  s = s.green().to_string();
               } else {
                  s = s.red().to_string();
               }
            },
            _ => {
               s += &format!("{:?}", create_entry.entry_type);
               s = s.green().to_string();
            }
         };
         str += &s;
      },
      Header::Update(update_entry) => {
         let mut s = String::new();
         match &update_entry.entry_type {
            EntryType::App(app_entry_type) => {
               let entry_kind: &'static str = EntryKind::from_index(&app_entry_type.id()).as_static();
               s += "AppEntry ";
               s += &format!("'{}'", entry_kind).green();
            },
            _ => {
               s += &format!("{:?}", update_entry.entry_type);
            }
         };
         str += &s.yellow().to_string();
      },
      Header::DeleteLink(delete_link) => {
         let s = format!("{}", delete_link.link_add_address);
         str += &format!("'{:.25}'", s).yellow().to_string();
      },
      Header::Delete(delete_entry) => {
         let s = format!("{}", delete_entry.deletes_address);
         str += &format!("'{:.25}'", s).green().to_string();
      }
      _ => {},
   }

   //       else {
   //    if (element.header.entry_type) {
   //       if (typeof element.header.entry_type === 'object') {
   //          str += ' - AppEntry ; id = ' + element.header.entry_type.App.id
   //       } else {
   //          str += ' - ' + element.header.entry_type
   //       }
   //    }
   // }

   let mut line = format!("{:<40} ({})", str, element.header_address);

   if element.header.is_genesis() {
      line = line.blue().to_string();
   }
   line
}
