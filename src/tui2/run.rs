use crossterm::{
   event::{self, Event as CEvent, KeyCode},
};
use std::path::{Path/*, PathBuf*/};
use std::sync::mpsc;
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use snapmail::handle::*;
use std::path::PathBuf;

use tui::{
   backend::CrosstermBackend,
   Terminal,
   style::Color,
};

use crate::{
   //error::SnapmailError,
   tui2::*,
   tui2::menu::*,
   globals::*,
   //app::*,
   //holochain::*,
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
   let (conductor, dna_hash) = start_conductor_or_abort(sid.clone()).await;
   let mut chain = pull_source_chain(conductor.clone()).await;
   terminal.clear()?;

   /// - Setup UI
   let mut app = App::new(sid, &chain);
   app.feedback(&format!("Welcome to Snapmail ! {:?}", dna_hash));

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
      /// Process Command
      let can_update_chain = app.process_command(conductor.clone(), &chain);
      if can_update_chain {
         // app.feedback("Updating source chain and app tables");
         chain = pull_source_chain(conductor.clone()).await;
         app.update_data(&chain);
      }
      /// Check if input received
      let event = rx.recv()?;
      let key_code =
         if let Event::Input(key_event) = event {
            //app.feedback(&format!("Key pressed: {:?}", key_event.code));
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
               KeyCode::Char('1') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 0)
                  }
               },
               KeyCode::Char('2') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 1)
                  }
               },
               KeyCode::Char('3') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 2)
                  }
               },
               KeyCode::Char('4') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 3)
                  }
               },
               KeyCode::Char('5') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 4)
                  }
               },
               KeyCode::Char('6') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 5)
                  }
               },
               KeyCode::Char('7') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 6)
                  }
               },
               KeyCode::Char('8') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 7)
                  }
               },
               KeyCode::Char('9') => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.try_download(conductor.clone(), 8)
                  }
               },
               /// View Menu
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
               /// Write Screen
               KeyCode::Insert => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.feedback_ext("Sending mail...", Color::White, Color::Blue);
                     app.command = AppCommand::SendMail;
                  }
               }
               KeyCode::Tab => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.toggle_write_block();
                  }
               },
               /// Settings Screen
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
               KeyCode::Char('d') => {
                  if app.active_menu_item == TopMenuItem::Settings {
                     app.input_variable = InputVariable::DownloadFolder;
                     app.input_mode = InputMode::Editing;
                     app.input = app.download_folder.clone().into_os_string().into_string().unwrap();
                  }
               },
               /// View Screen
               KeyCode::Down => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.next_mail(&chain);

                  }
               }
               KeyCode::Up => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.previous_mail(&chain);
                  }
               },
               /// Misc
               KeyCode::Enter => {},
               KeyCode::PageUp => {
                  app.feedback_index = std::cmp::max(0 as i32, app.feedback_index as i32 - 1) as u32;
               },
               KeyCode::PageDown => {
                  app.feedback_index = std::cmp::min(app.feedbacks.len() as i32 - 1, app.feedback_index as i32 + 1) as u32;
               },
               _ => {}
            }
         },
         InputMode::Editing => {
            match key_code  {
               KeyCode::Esc => {
                  app.input_mode = InputMode::Normal;
                  if app.active_menu_item == TopMenuItem::Write {
                     app.set_write_block(WriteBlock::None)
                     //app.set_write_block(WriteBlock::Contacts);
                     //app.active_menu_item = TopMenuItem::View;
                  }
               },
               KeyCode::Tab => {
                  if app.active_menu_item == TopMenuItem::Write {
                     app.toggle_write_block();
                  }
               },
               KeyCode::Down => {
                  if app.active_menu_item == TopMenuItem::Write &&
                     app.active_write_block == WriteBlock::Contacts {
                     app.feedback("ContactsTable NEXT");
                     app.contacts_table.next();
                  }
               }
               KeyCode::Up => {
                  if app.active_menu_item == TopMenuItem::Write  &&
                     app.active_write_block == WriteBlock::Contacts {
                     app.feedback("ContactsTable PREVIOUS");
                     app.contacts_table.previous();
                  }
               },
               KeyCode::Enter => {
                  if app.active_menu_item == TopMenuItem::Settings {
                     app.input_mode = InputMode::Normal;
                     match app.input_variable {
                        InputVariable::Handle => {
                           let hash = snapmail_set_handle(conductor.clone(), app.input.clone())?;
                           app.feedback(&format!("Handle entry hash: {}", hash.to_string()));
                        },
                        InputVariable::Uid => {
                           app.uid = app.input.clone();
                           let config_path = Path::new(&*CONFIG_PATH).join(app.sid.clone());
                           let app_filepath = config_path.join(APP_CONFIG_FILENAME);
                           std::fs::write(app_filepath, app.uid.as_bytes()).unwrap();
                           // Must restart conductor
                           return Ok(());
                        },
                        InputVariable::DownloadFolder => {
                           app.download_folder = PathBuf::from(app.input.clone());
                           let config_path = Path::new(&*CONFIG_PATH).join(app.sid.clone());
                           let app_filepath = config_path.join(APP_DL_CONFIG_FILENAME);
                           std::fs::write(app_filepath, app.input.as_bytes()).unwrap();
                        },
                        _ => {},
                     }
                  }
                  if app.active_menu_item == TopMenuItem::Write {
                     match app.active_write_block {
                        WriteBlock::Contacts => {
                           app.contacts_table.toggle_selected();
                        },
                        WriteBlock::Content => {
                           app.input.push('\n');
                        },
                        // WriteBlock::Attachments => {
                        //    let path = PathBuf::from(app.input.clone());
                        //    app.write_attachments.push(path);
                        //    app.input = String::new();
                        // },
                        _ => {}
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

