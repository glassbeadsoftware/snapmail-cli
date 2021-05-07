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
use std::sync::RwLock;

use tui::{
   Frame,
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout, Rect},
   style::{Color, Modifier, Style},
   text::{Span, Spans},
   widgets::{
      Widget,
      Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
   },
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

pub fn draw(
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   sid: &str,
   uid: String,
   handle: String,
   active_menu_item: &mut TopMenuItem,
   frame_count: u32,
) {
   let menu_titles = vec!["View", "Write", "Settings", "Quit"];

   /// Set vertical layout
   let size = main_rect.size();
   let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(1)
      .constraints(
         [
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
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
   let title = format!("Snapmail v0.0.4 - {} - {} - {} - {}",
                       sid, uid, handle.clone(), frame_count);
   let tabs = Tabs::new(top_menu)
      .select(active_menu_item.to_owned().into())
      .block(Block::default().title(title).borders(Borders::ALL))
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().fg(Color::Yellow))
      .divider(Span::raw("|"));
   main_rect.render_widget(tabs, chunks[0]);

   let app = g_app.read().unwrap();
   //let input_mode = app.input_mode.clone();
   let feedback = Paragraph::new(app.messages[0].clone())
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Feedback")
            .border_type(BorderType::Plain),
      );
   main_rect.render_widget(feedback, chunks[2]);

   /// Render main block according to active menu item
   match active_menu_item {
      TopMenuItem::View => render_view(main_rect, chunks[1]),
      TopMenuItem::Write => render_write(main_rect, chunks[1]),
      TopMenuItem::Settings => render_settings(main_rect, chunks[1]),
   }
}


///
fn render_view(main_rect: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
   let top = Paragraph::new("Folder")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Folder")
            .border_type(BorderType::Plain),
      );

   let bottom = Paragraph::new("Mail")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Mail")
            .border_type(BorderType::Plain),
      );

   // let left = Paragraph::new("MailItem")
   //    .alignment(Alignment::Center)
   //    .block(
   //       Block::default()
   //          .borders(Borders::NONE)
   //          .style(Style::default().fg(Color::White))
   //          .title("MailItem")
   //          .border_type(BorderType::Plain),
   //    );

   let right = Paragraph::new("Attachments")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Attachments")
            .border_type(BorderType::Plain),
      );

   let vert_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
         [Constraint::Percentage(66), Constraint::Percentage(34)].as_ref(),
      )
      .split(area);

   let hori_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
         [Constraint::Percentage(66), Constraint::Percentage(34)].as_ref(),
      )
      .split(vert_chunks[1]);

   main_rect.render_widget(top, vert_chunks[0]);
   //main_rect.render_widget(bottom, vert_chunks[1]);
   main_rect.render_widget(bottom, hori_chunks[0]);
   main_rect.render_widget(right, hori_chunks[1]);

}



///
fn render_write(main_rect: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
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

   main_rect.render_widget(left, write_chunks[0]);
   main_rect.render_widget(right, write_chunks[1]);
   //rect.render_stateful_widget(right, write_chunks[1], &mut pet_list_state);
}

///
fn render_settings(main_rect: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {

   let app = g_app.read().unwrap();

   let settings_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
         [Constraint::Min(6), Constraint::Length(3)].as_ref(),
      )
      .split(area);

   let items = vec!["Handle", "UID", "Proxy URL", "Bootstrap URL"];

   let items: Vec<Spans> = items
      .iter()
      .map(|&item| {
         let (first, rest) = item.split_at(1);
         let span =
            Spans::from(vec![
               Span::styled(
                  first,
                  Style::default()
                     .fg(Color::Yellow)
                     .add_modifier(Modifier::UNDERLINED),
               ),
               Span::styled(rest, Style::default().fg(Color::White)),
            ]);
         //ListItem::new(span)
         span
      })
      .collect();

   //List::new(items)
   let top = Paragraph::new(items)
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Settings")
            .border_type(BorderType::Plain),
      );

   let bottom = Paragraph::new(g_app.read().unwrap().input.clone())
      //.alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match g_app.read().unwrap().input_mode {
               InputMode::Normal => Style::default(),
               InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .title(app.input_variable.to_string())
            .border_type(BorderType::Plain),
      );


   match app.input_mode {
      InputMode::Normal =>
      // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
         {}
      InputMode::Editing => {
         // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
         main_rect.set_cursor(
            // Put cursor past the end of the input text
            settings_chunks[1].x + app.input.len() as u16 + 1,
            // Move one line down, from the border to the input line
            settings_chunks[1].y + 1,
         )
      }
   }

   main_rect.render_widget(top, settings_chunks[0]);
   main_rect.render_widget(bottom, settings_chunks[1]);

}
