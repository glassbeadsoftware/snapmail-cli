use crossterm::{
   event::{self, Event as CEvent, KeyCode},
};
use std::path::Path;
use std::sync::mpsc;
use std::io;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use tui::{
   backend::CrosstermBackend,
   Terminal,
   style::Color,
};
use snapmail_common::{
   globals::*,
   conductor::*,
};
use crate::{
   menu::*,
   app::InputMode, app::InputVariable,
   app::AppCommand, app::App,
   snapmail_chain::*,
   listen_signal::listen_signal,
   render::draw,
};

///
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
   app.feedback(&format!("Welcome to Snapmail! - {:?}", dna_hash));

   /// Setup input loop
   let (input_tx, input_rx) = mpsc::channel();
   let tick_rate = Duration::from_millis(200);
   tokio::spawn( async move {
      let mut last_tick = Instant::now();
      loop {
         let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
         if event::poll(timeout).expect("poll works") {
            if let CEvent::Key(key) = event::read().expect("can read events") {
               let _res = input_tx.send(Event::Input(key));
            }
         }
         if last_tick.elapsed() >= tick_rate {
            let res = input_tx.send(Event::Tick);
            if let Ok(_) = res {
               last_tick = Instant::now();
            }
         }
      }
   });

   /// Setup Signal receive loop
   let (signal_tx, signal_rx) = mpsc::channel();
   let conductor_c = conductor.clone();
   tokio::spawn(async move {
      let _res = listen_signal(conductor_c,  signal_tx).await;
   });

   /// Render loop
   loop {
      app.frame_count += 1;
      app.peer_count = dump_state(conductor.clone());

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
      let event = input_rx.recv()?;
      let key_code =
         if let Event::Input(key_event) = event {
            //app.feedback(&format!("Key pressed: {:?}", key_event.code));
            key_event.code
         } else { KeyCode::Null };
      let input_mode = app.input_mode.clone();

      /// Check if Signal received
      // let maybe_signal_msg = signal_rx.recv();
      if let Ok(signal_msg) = signal_rx.recv() {
         app.feedback_ext(&signal_msg, Color::White, Color::Blue);
      }

      match input_mode {
         InputMode::Navigation => {
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
                     app.input_mode = InputMode::Scrolling;
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
               },
               KeyCode::Up => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.previous_mail(&chain);
                  }
               },
               KeyCode::Delete => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.command = AppCommand::DeleteMail;
                  }
               },
               /// Misc
               KeyCode::Enter => {
                  if app.active_menu_item == TopMenuItem::View {
                     app.input_mode = InputMode::Scrolling;
                  }
                  if app.active_menu_item == TopMenuItem::Write {
                     app.feedback_ext("Sending mail...", Color::White, Color::Blue);
                     app.command = AppCommand::SendMail;
                     app.input_mode = InputMode::Scrolling;
                  }
               },
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
                  app.input_mode = InputMode::Navigation;
                  if app.active_menu_item == TopMenuItem::Write {
                     app.set_write_block(WriteBlock::None)
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
                     app.contacts_table.next();
                     app.show_selected_contact();
                  }
               }
               KeyCode::Up => {
                  if app.active_menu_item == TopMenuItem::Write
                     && app.active_write_block == WriteBlock::Contacts {
                     app.contacts_table.previous();
                     app.show_selected_contact();
                  }
               },
               KeyCode::Enter => {
                  if app.active_menu_item == TopMenuItem::Settings {
                     app.input_mode = InputMode::Navigation;
                     match app.input_variable {
                        InputVariable::Handle => {
                           app.command = AppCommand::UpdateHandle;
                           app.feedback_ext("Updating Handle", Color::White, Color::Blue);
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
                        _ => { app.set_write_block(WriteBlock::None); }
                     }
                  }
               },
               KeyCode::Char('\n') => {
                  app.input_mode = InputMode::Navigation;
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
         InputMode::Scrolling => {
            match key_code {
               KeyCode::Esc => {
                  app.input_mode = InputMode::Navigation;
                  app.scroll_y = 0;
               },
               KeyCode::Down => app.scroll_y = app.scroll_y.saturating_add(1),
               KeyCode::Up =>   app.scroll_y = app.scroll_y.saturating_sub(1),
               KeyCode::PageDown => app.scroll_y = app.scroll_y.saturating_add(10),
               KeyCode::PageUp => app.scroll_y = app.scroll_y.saturating_sub(10),
               _ => {},
            }
            // app.feedback(&format!("scroll_y = {}", app.scroll_y));
         },
      }
   }
}

