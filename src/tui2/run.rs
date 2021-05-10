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



///
pub async fn run(
   terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
   sid: String,
) -> Result<(), Box<dyn std::error::Error>> {
   let conductor = start_conductor(sid.clone()).await;
   let chain = pull_source_chain(conductor.clone()).await;
   let mail_list = filter_chain(&chain, FolderItem::Inbox);
   let mut mail_table = MailTable::new(mail_list, &chain.handle_map);
   terminal.clear()?;

   let path = CONFIG_PATH.as_path().join(sid.clone());
   let app_filepath = path.join(APP_CONFIG_FILENAME);
   let mut uid = std::fs::read_to_string(app_filepath)
      .expect("Something went wrong reading APP CONFIG file");

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

   /// Set Menu
   let mut active_menu_item = TopMenuItem::View;
   let mut folder_item = FolderItem::Inbox;
   let mut frame_count = 0;

   /// Render loop
   loop {
      frame_count += 1;
      /// Render
      terminal.draw(|main_rect| {
         draw(
            main_rect,
            &chain,
            &mut mail_table,
            &sid,
            uid.clone(),
            chain.my_handle.clone(),
            &mut active_menu_item,
            &mut folder_item,
            frame_count,
         )
      })?;

      /// Check if input received
      let event = rx.recv()?;
      let key_code =
         if let Event::Input(key_event) = event {
            key_event.code
         } else { KeyCode::Null };

      let input_mode = g_app.read().unwrap().input_mode.clone();
      match input_mode {
         InputMode::Normal => {
            match key_code  {
               KeyCode::Esc => return Ok(()),
               /// Top Menu
               KeyCode::Char('q') => return Ok(()),
               KeyCode::Char('v') => active_menu_item = TopMenuItem::View,
               KeyCode::Char('w') => active_menu_item = TopMenuItem::Write,
               KeyCode::Char('e') => active_menu_item = TopMenuItem::Settings,

               KeyCode::Char('i') => folder_item = FolderItem::Inbox,
               KeyCode::Char('s') => folder_item = FolderItem::Sent,
               KeyCode::Char('t') => folder_item = FolderItem::Trash,
               KeyCode::Char('a') => folder_item = FolderItem::All,

               /// Settings Menu
               KeyCode::Char('b') => {
                  if active_menu_item == TopMenuItem::Settings {
                     let mut app = g_app.write().unwrap();
                     app.input_variable = InputVariable::BoostrapUrl;
                     app.input_mode = InputMode::Editing;
                     app.input = String::new();
                  }
               },
               KeyCode::Char('h') => {
                  if active_menu_item == TopMenuItem::Settings {
                     let mut app = g_app.write().unwrap();
                     app.input_variable = InputVariable::Handle;
                     app.input_mode = InputMode::Editing;
                     app.input = chain.my_handle.clone();
                  }
               },
               KeyCode::Char('u') => {
                  if active_menu_item == TopMenuItem::Settings {
                     let mut app = g_app.write().unwrap();
                     app.input_variable = InputVariable::Uid;
                     app.input_mode = InputMode::Editing;
                     app.input = uid.clone();
                  }
               },

               // KeyCode::Down => {
               //    if let Some(selected) = pet_list_state.selected() {
               //       let amount_pets = read_db().expect("can fetch pet list").len();
               //       if selected >= amount_pets - 1 {
               //          pet_list_state.select(Some(0));
               //       } else {
               //          pet_list_state.select(Some(selected + 1));
               //       }
               //    }
               // }
               // KeyCode::Up => {
               //    if let Some(selected) = pet_list_state.selected() {
               //       let amount_pets = read_db().expect("can fetch pet list").len();
               //       if selected > 0 {
               //          pet_list_state.select(Some(selected - 1));
               //       } else {
               //          pet_list_state.select(Some(amount_pets - 1));
               //       }
               //    }
               // }
               _ => {}
            }
         },
         InputMode::Editing => {

            match key_code  {
               KeyCode::Esc => {
                  g_app.write().unwrap().input_mode = InputMode::Normal;
                  //events.enable_exit_key();
               },
               KeyCode::Enter => {
                  let mut app = g_app.write().unwrap();
                  app.input_mode = InputMode::Normal;
                  match app.input_variable {
                     InputVariable::Handle => {
                        let hash = snapmail_set_handle(conductor.clone(), app.input.clone())?;
                        app.messages.insert(0, format!("Handle entry hash: {}", hash.to_string()));
                     },
                     InputVariable::Uid => {
                        uid = app.input.clone();
                        let config_path = Path::new(&*CONFIG_PATH).join(sid.clone());
                        let app_filepath = config_path.join(APP_CONFIG_FILENAME);
                        std::fs::write(app_filepath, uid.as_bytes()).unwrap();
                        // Must restart conductor
                        return Ok(());
                     },
                     _ => {},
                  }
               },
               KeyCode::Char('\n') => {
                  g_app.write().unwrap().input_mode = InputMode::Normal;
               }
               KeyCode::Char(c) => {
                  g_app.write().unwrap().input.push(c);
               }
               KeyCode::Backspace => {
                  g_app.write().unwrap().input.pop();
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
