use chrono::prelude::*;
use crossterm::{
   event::{self, Event as CEvent, KeyCode},
   terminal::{disable_raw_mode, enable_raw_mode},
};

use std::sync::mpsc;
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use snapmail::mail::*;
use snapmail::handle::*;

use tui::{
   backend::CrosstermBackend,
   // layout::{Alignment, Constraint, Direction, Layout, Rect},
   // style::{Color, Modifier, Style},
   // text::{Span, Spans},
   // widgets::{
   //    Widget,
   //    Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
   // },
   Terminal,
};

use crate::{
   error::SnapmailError,
   tui2::*,
   tui2::menu::*,
   app::*,
   app::InputVariable,
   //app::g_app,
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
   let handle = snapmail_get_my_handle(conductor.clone(), ())?;

   let path = CONFIG_PATH.as_path().join(sid.clone());
   let app_filepath = path.join(APP_CONFIG_FILENAME);
   let uid = std::fs::read_to_string(app_filepath)
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
   let mut frame_count = 0;

   /// Render loop
   loop {
      frame_count += 1;
      /// Render
      terminal.draw(|main_rect| {
         draw(main_rect, &sid, uid.clone(), handle.clone(), &mut active_menu_item, frame_count)
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
               KeyCode::Char('s') => active_menu_item = TopMenuItem::Settings,
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
                     app.input = handle.clone();
                  }
               },
               KeyCode::Char('u') => {
                  if active_menu_item == TopMenuItem::Settings {
                     let mut app = g_app.write().unwrap();
                     app.input_variable = InputVariable::Uid;
                     app.input_mode = InputMode::Editing;
                     app.input = "FIXME".to_string();
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

