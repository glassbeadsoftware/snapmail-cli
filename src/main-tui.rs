#![allow(unused_doc_comments)]

pub mod error;
pub mod tui2;

use chrono::prelude::*;
use crossterm::{
   event::{self, Event as CEvent, KeyCode},
   terminal::{disable_raw_mode, enable_raw_mode},
};
//use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
   Frame,
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout, Rect},
   style::{Color, Modifier, Style},
   text::{Span, Spans},
   widgets::{
      Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
   },
   Terminal,
};

use crate::{
   error::SnapmailError,
   tui2::*,
   tui2::menu::*,
   app::*,
};


enum Event<I> {
   Input(I),
   Tick,
}


///
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   /// Set raw mode ('Enter' not required)
   enable_raw_mode().expect("can run in raw mode");

   /// Setup terminal
   let stdout = io::stdout();
   let backend = CrosstermBackend::new(stdout);
   let mut terminal = Terminal::new(backend)?;
   terminal.clear()?;


   //let _conductor = start_conductor(sid_str).await;

   let _res = run(&mut terminal);

   /// Shutdown terminal
   disable_raw_mode()?;
   terminal.show_cursor()?;
   Ok(())
}

///
fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {

   /// Create default app state
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

   /// Set Menu
   let menu_titles = vec!["View", "Write", "Settings", "Quit"];
   let mut active_menu_item = TopMenuItem::View;
   let mut pet_list_state = ListState::default();
   pet_list_state.select(Some(0));
   /// Render loop
   loop {

      /// Render
      terminal.draw(|rect| {
         /// Set vertical layout
         let size = rect.size();
         let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
               [
                  Constraint::Length(3),
                  Constraint::Min(10),
               ]
                  .as_ref(),
            )
            .split(size);

         // let mail = Paragraph::new("")
         //    .style(Style::default().fg(Color::LightCyan))
         //    .alignment(Alignment::Center)
         //    .block(
         //       Block::default()
         //          .borders(Borders::ALL)
         //          .style(Style::default().fg(Color::White))
         //          .title("")
         //          //.borders(Borders::NONE),
         //          .border_type(BorderType::Double),
         //    );
         // rect.render_widget(mail, chunks[2]);

         /// Set top menu
         let top_menu = menu_titles
            .iter()
            .map(|t| {
               let (first, rest) = t.split_at(1);
               Spans::from(vec![
                  Span::styled(
                     first,
                     Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                  ),
                  Span::styled(rest, Style::default().fg(Color::White)),
               ])
            })
            .collect();
         let tabs = Tabs::new(top_menu)
            .select(active_menu_item.into())
            .block(Block::default().title("Snapmail v0.0.4 - <network> - <handle>").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider(Span::raw("|"));
         rect.render_widget(tabs, chunks[0]);

         /// Render main block according to active menu item
         match active_menu_item {
            TopMenuItem::View => rect.render_widget(render_view(), chunks[1]),
            TopMenuItem::Write => render_write(rect, chunks[1]),
            TopMenuItem::Settings => rect.render_widget(render_settings(), chunks[1]),
         }
      })?;

      /// Check if input received
      let event = rx.recv()?;
      let key_code =
      if let Event::Input(key_event) = event {
         key_event.code
      } else { KeyCode::Null };

      match app.input_mode {
         InputMode::Normal => {
            match key_code  {
               KeyCode::Char('q') => return Ok(()),
               KeyCode::Char('v') => active_menu_item = TopMenuItem::View,
               KeyCode::Char('w') => active_menu_item = TopMenuItem::Write,
               KeyCode::Char('s') => active_menu_item = TopMenuItem::Settings,
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
                  app.input_mode = InputMode::Normal;
                  //events.enable_exit_key();
               }
               _ => {}
            }
         },
      }
   }
}

fn render_write(rect: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
   let left = Paragraph::new("Write")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Write")
            .border_type(BorderType::Plain),
      );

   let right = Paragraph::new("Users")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Users")
            .border_type(BorderType::Plain),
      );

   let write_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
         [Constraint::Percentage(66), Constraint::Percentage(34)].as_ref(),
      )
      .split(area);

   rect.render_widget(left, write_chunks[0]);
   rect.render_widget(right, write_chunks[1]);
   //rect.render_stateful_widget(right, write_chunks[1], &mut pet_list_state);

}

fn render_settings<'a>() -> Paragraph<'a> {
   Paragraph::new("Settings")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Settings")
            .border_type(BorderType::Plain),
      )
}


///
fn render_view<'a>() -> Paragraph<'a> {
   let home = Paragraph::new(vec![
      Spans::from(vec![Span::raw("")]),
      Spans::from(vec![Span::raw("Welcome")]),
      Spans::from(vec![Span::raw("")]),
      Spans::from(vec![Span::raw("to")]),
      Spans::from(vec![Span::raw("")]),
      Spans::from(vec![Span::styled(
         "snapmail-TUI",
         Style::default().fg(Color::LightBlue),
      )]),
      Spans::from(vec![Span::raw("")]),
      Spans::from(vec![Span::raw("Press 'v' to access Mails, 'w' to add write a new mail and 's' to change settings.")]),
   ])
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("View")
            .border_type(BorderType::Plain),
      );
   home
}

