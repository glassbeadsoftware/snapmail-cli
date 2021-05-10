use crossterm::{
   event::{self, Event as CEvent, KeyCode},
};
use std::path::Path;
use std::sync::mpsc;
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use snapmail::mail::*;
use snapmail::mail::entries::*;
use snapmail::handle::*;

use tui::{
   backend::CrosstermBackend,
   widgets::TableState,
   Terminal,
};

use crate::{
   error::SnapmailError,
   tui2::*,
   tui2::menu::*,
   app::*,
   globals::*,
   holochain::*,
   conductor::*,
};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Event<I> {
   Input(I),
   Tick,
}


pub struct UiState {
   pub frame_count: u32,
   pub sid: String,
   pub uid: String,
   pub active_menu_item: TopMenuItem,
   pub folder_item: FolderItem,
   pub mail_table: MailTable,
   pub contacts_table: ContactsTable,
}


///
pub async fn run(
   terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
   sid: String,
) -> Result<(), Box<dyn std::error::Error>> {
   /// - Startup holochain
   let conductor = start_conductor(sid.clone()).await;
   let chain = pull_source_chain(conductor.clone()).await;
   terminal.clear()?;
   /// - Get UID
   let path = CONFIG_PATH.as_path().join(sid.clone());
   let app_filepath = path.join(APP_CONFIG_FILENAME);
   let uid = std::fs::read_to_string(app_filepath)
      .expect("Something went wrong reading APP CONFIG file");
   /// - Setup UI
   let mail_list = filter_chain(&chain, FolderItem::Inbox);
   let mail_table = MailTable::new(mail_list, &chain.handle_map);
   let contacts_table = ContactsTable::new(&chain.handle_map);
   let mut ui = UiState {
      frame_count: 0,
      sid,
      uid,
      active_menu_item: TopMenuItem::View,
      folder_item: FolderItem::Inbox,
      mail_table,
      contacts_table,
   };
   let mut app = App::default();

   /// Setup input loop
   let (tx, rx) = mpsc::channel();
   let tick_rate = Duration::from_millis(200);
   thread::spawn(move || {
      let mut last_tick = Instant::now();
      loop {
         let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
         if event::poll(timeout).expect("poll works") {
            if let CEvent::Key(key) = event::read().expect("can read events") {
               tx.send(Event::Input(key)).expect("can send events");
            }
         }
         if last_tick.elapsed() >= tick_rate {
            if let Ok(_) = tx.send(Event::Tick) {
               last_tick = Instant::now();
            }
         }
      }
   });


   /// Render loop
   loop {
      ui.frame_count += 1;
      /// Render
      terminal.draw(|main_rect| {
         draw(main_rect, &chain, &mut ui, &app);
      })?;

      /// Check if input received
      let event = rx.recv()?;
      let key_code =
         if let Event::Input(key_event) = event {
            app.messages.insert(0, format!("Key pressed: {:?}", key_event.code));
            key_event.code
         } else { KeyCode::Null };
      let input_mode = app.input_mode.clone();

      match input_mode {
         InputMode::Normal => {
            match key_code  {
               KeyCode::Esc => return Ok(()),
               /// Top Menu
               KeyCode::Char('q') => return Ok(()),
               KeyCode::Char('v') => ui.active_menu_item = TopMenuItem::View,
               KeyCode::Char('w') => ui.active_menu_item = TopMenuItem::Write,
               KeyCode::Char('e') => ui.active_menu_item = TopMenuItem::Settings,

               KeyCode::Char('i') => {
                  if ui.active_menu_item == TopMenuItem::View {
                     ui.folder_item = FolderItem::Inbox;
                     let mail_list = filter_chain(&chain, ui.folder_item);
                     ui.mail_table = MailTable::new(mail_list, &chain.handle_map);
                  }
               },
               KeyCode::Char('s') => {
                  if ui.active_menu_item == TopMenuItem::View {
                     ui.folder_item = FolderItem::Sent;
                     let mail_list = filter_chain(&chain, ui.folder_item);
                     ui.mail_table = MailTable::new(mail_list, &chain.handle_map);
                  }
               },
               KeyCode::Char('t') => {
                  if ui.active_menu_item == TopMenuItem::View {
                     ui.folder_item = FolderItem::Trash;
                     let mail_list = filter_chain(&chain, ui.folder_item);
                     ui.mail_table = MailTable::new(mail_list, &chain.handle_map);
                  }
               },
               KeyCode::Char('a') => {
                  if ui.active_menu_item == TopMenuItem::View {
                     ui.folder_item = FolderItem::All;
                     let mail_list = filter_chain(&chain, ui.folder_item);
                     ui.mail_table = MailTable::new(mail_list, &chain.handle_map);
                  }
               },

               /// Settings Menu
               KeyCode::Char('b') => {
                  if ui.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::BoostrapUrl;
                     app.input_mode = InputMode::Editing;
                     app.input = String::new();
                  }
               },
               KeyCode::Char('h') => {
                  if ui.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::Handle;
                     app.input_mode = InputMode::Editing;
                     app.input = chain.my_handle.clone();
                  }
               },
               KeyCode::Char('u') => {
                  if ui.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::Uid;
                     app.input_mode = InputMode::Editing;
                     app.input = ui.uid.clone();
                  }
               },
               KeyCode::Down => {
                  if ui.active_menu_item == TopMenuItem::View {
                     app.messages.insert(0, "MailTable NEXT".to_string());
                     ui.mail_table.next();
                  }
                  if ui.active_menu_item == TopMenuItem::Write {
                     app.messages.insert(0, "ContactsTable NEXT".to_string());
                     ui.contacts_table.next();
                  }
               }
               KeyCode::Up => {
                  if ui.active_menu_item == TopMenuItem::View {
                     app.messages.insert(0, "MailTable PREVIOUS".to_string());
                     ui.mail_table.previous();
                  }
                  if ui.active_menu_item == TopMenuItem::Write {
                     app.messages.insert(0, "ContactsTable PREVIOUS".to_string());
                     ui.contacts_table.previous();
                  }
               }
               KeyCode::Enter => {
                  if ui.active_menu_item == TopMenuItem::Write {
                     ui.contacts_table.toggle_selected();
                  }
               }
               _ => {}
            }
         },
         InputMode::Editing => {

            match key_code  {
               KeyCode::Esc => {
                  app.input_mode = InputMode::Normal;
               },
               KeyCode::Enter => {
                  app.input_mode = InputMode::Normal;
                  match app.input_variable {
                     InputVariable::Handle => {
                        let hash = snapmail_set_handle(conductor.clone(), app.input.clone())?;
                        app.messages.insert(0, format!("Handle entry hash: {}", hash.to_string()));
                     },
                     InputVariable::Uid => {
                        ui.uid = app.input.clone();
                        let config_path = Path::new(&*CONFIG_PATH).join(ui.sid.clone());
                        let app_filepath = config_path.join(APP_CONFIG_FILENAME);
                        std::fs::write(app_filepath, ui.uid.as_bytes()).unwrap();
                        // Must restart conductor
                        return Ok(());
                     },
                     _ => {},
                  }
               },
               KeyCode::Char('\n') => {
                  app.input_mode = InputMode::Normal;
               }
               KeyCode::Char(c) => {
                  app.input.push(c);
               }
               KeyCode::Backspace => {
                  app.input.pop();
               }
               _ => {},
            }
         },
      }
   }
}

///
fn filter_chain(chain: &SnapmailChain, folder: FolderItem) -> Vec<MailItem> {
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
