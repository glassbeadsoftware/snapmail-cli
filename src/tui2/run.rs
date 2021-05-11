use crossterm::{
   event::{self, Event as CEvent, KeyCode},
};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use snapmail::handle::*;

use tui::{
   backend::CrosstermBackend,
   Terminal,
};

use crate::{
   error::SnapmailError,
   tui2::*,
   tui2::menu::*,
   globals::*,
   app::*,
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
   /// - Startup holochain
   let conductor = start_conductor(sid.clone()).await;
   let chain = pull_source_chain(conductor.clone()).await;
   terminal.clear()?;

   /// - Setup UI
   let mut app = App::new(sid, &chain);

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
      app.frame_count += 1;
      /// Render
      terminal.draw(|main_rect| {
         draw(main_rect, &chain, &mut app);
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
               /// Top Menu
               KeyCode::Esc |
               KeyCode::Char('q') => return Ok(()),
               KeyCode::Char('v') => app.active_menu_item = TopMenuItem::View,
               KeyCode::Char('w') => app.active_menu_item = TopMenuItem::Write,
               KeyCode::Char('e') => app.active_menu_item = TopMenuItem::Settings,

               KeyCode::Char('i') => {
                  app.update_active_folder(&chain, FolderItem::Inbox)
               },
               KeyCode::Char('s') => {
                  app.update_active_folder(&chain, FolderItem::Sent)
               },
               KeyCode::Char('t') => {
                  app.update_active_folder(&chain, FolderItem::Trash)
               },
               KeyCode::Char('a') => {
                  app.update_active_folder(&chain, FolderItem::All)
               },

               KeyCode::Insert => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.send_mail(conductor.clone(), &chain);
                  }
               }

               /// Settings Menu
               KeyCode::Char('b') => {
                  if app.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::BoostrapUrl;
                     app.input_mode = InputMode::Editing;
                     app.input = String::new();
                  }
               },
               KeyCode::Char('h') => {
                  if app.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::Handle;
                     app.input_mode = InputMode::Editing;
                     app.input = chain.my_handle.clone();
                  }
               },
               KeyCode::Char('u') => {
                  if app.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::Uid;
                     app.input_mode = InputMode::Editing;
                     app.input = app.uid.clone();
                  }
               },
               KeyCode::Down => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.messages.insert(0, "MailTable NEXT".to_string());
                     app.mail_table.next();
                  }
                  if app.active_menu_item == TopMenuItem::Write &&
                     app.active_write_block == WriteBlock::Contacts {
                     app.messages.insert(0, "ContactsTable NEXT".to_string());
                     app.contacts_table.next();
                  }
               }
               KeyCode::Up => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.messages.insert(0, "MailTable PREVIOUS".to_string());
                     app.mail_table.previous();
                  }
                  if app.active_menu_item == TopMenuItem::Write  &&
                     app.active_write_block == WriteBlock::Contacts {
                     app.messages.insert(0, "ContactsTable PREVIOUS".to_string());
                     app.contacts_table.previous();
                  }
               },
               KeyCode::Tab => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.toggle_write_block();
                  }
               },
               KeyCode::Enter => {
                  if app.active_menu_item == TopMenuItem::Write &&
                     app.active_write_block == WriteBlock::Contacts {
                     app.contacts_table.toggle_selected();
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
               KeyCode::Tab => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.toggle_write_block();
                  }
               },
               KeyCode::Insert => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.send_mail(conductor.clone(), &chain);
                  }
               }
               KeyCode::Enter => {
                  app.input_mode = InputMode::Normal;
                  if app.active_menu_item == TopMenuItem::Settings {
                     match app.input_variable {
                        InputVariable::Handle => {
                           let hash = snapmail_set_handle(conductor.clone(), app.input.clone())?;
                           app.messages.insert(0, format!("Handle entry hash: {}", hash.to_string()));
                        },
                        InputVariable::Uid => {
                           app.uid = app.input.clone();
                           let config_path = Path::new(&*CONFIG_PATH).join(app.sid.clone());
                           let app_filepath = config_path.join(APP_CONFIG_FILENAME);
                           std::fs::write(app_filepath, app.uid.as_bytes()).unwrap();
                           // Must restart conductor
                           return Ok(());
                        },
                        _ => {},
                     }
                  }
                  if app.active_menu_item == TopMenuItem::Write {
                     match app.input_variable {
                        InputVariable::Attachment => {
                           let path = PathBuf::from(app.input.clone());
                           app.write_attachments.push(path);
                           app.input = String::new();
                        }
                        _ => {}
                        //InputVariable::Mail => { }
                     }
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

