use std::io;

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
};

use crate::{
   tui2::*,
   tui2::menu::*,
};

pub fn draw(
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   chain: &SnapmailChain,
   app: &mut App,
) {
   let menu_titles = vec!["View", "Write", "Edit Settings", "Quit"];

   /// Set vertical layout
   let size = main_rect.size();
   let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(0)// FIXME
      .constraints(
         [
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
         ].as_ref(),
      )
      .split(size);

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
                       app.sid, app.uid, chain.my_handle.clone(), app.frame_count);
   let tabs = Tabs::new(top_menu)
      .select(app.active_menu_item.to_owned().into())
      .block(Block::default().title(title).borders(Borders::ALL))
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().fg(Color::Yellow))
      .divider(Span::raw("|"));
   main_rect.render_widget(tabs, chunks[0]);

   //let app = g_app.read().unwrap();
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
   match app.active_menu_item {
      TopMenuItem::View => render_view(chain, main_rect, chunks[1], app),
      TopMenuItem::Write => render_write(chain, main_rect, chunks[1], app),
      TopMenuItem::Settings => render_settings(app, main_rect, chunks[1]),
   }
}


///
fn render_view(
   chain: &SnapmailChain,
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   area: Rect,
   app: &mut App,
) {
   /// -- Set top menu
   let menu_titles = vec!["Inbox", "Sent", "Trash", "All"];
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
      .select(app.active_folder_item.to_owned().into())
      .block(Block::default().title("Filebox").borders(Borders::ALL))
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().fg(Color::Yellow))
      .divider(Span::raw("|"));

   /// -- Set Mail Table
   let selected_style = Style::default().add_modifier(Modifier::REVERSED);
   let normal_style = Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD);

   //let header_cells = ["ID", "Username", "Subject", "Date", "Status"]
   let header_cells = ["", "From", "Subject", "Message", "Date"]
      .iter()
      .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
   let header = Row::new(header_cells)
      .style(normal_style)
      .height(1)
      .bottom_margin(0);

   let rows = app.mail_table.items.iter().map(|item| {
      let height = item
         .iter()
         .map(|content| content.chars().filter(|c| *c == '\n').count())
         .max()
         .unwrap_or(0)
         + 1;
      let cells = item.iter().map(|c| Cell::from(c.as_str()));
      Row::new(cells).height(height as u16).bottom_margin(0)
   });
   let table = Table::new(rows)
      .header(header)
      .block(Block::default().borders(Borders::NONE).title(""))
      .highlight_style(selected_style)
      //.highlight_symbol(">> ")
      .widths(&[
         //Constraint::Min(10),
         Constraint::Length(4),
         Constraint::Length(20),
         Constraint::Length(28),
         Constraint::Length(12),
         Constraint::Length(16),
      ]);


   /// -- Draw selected mail
   let mail_txt = if let Some(index) = app.mail_table.state.selected() {
      app.mail_table.get_mail_text(index, &chain)
   } else {
      "No Mail Selected".to_string()
   };
   let bottom = Paragraph::new(mail_txt)
      .alignment(Alignment::Left)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Mail")
            .border_type(BorderType::Plain),
      );

   let right = Paragraph::new("Attachments")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Attachments")
            .border_type(BorderType::Plain),
      );

   /// - Layout and render
   let vert_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
         [
            Constraint::Length(3),
            Constraint::Percentage(55),
            Constraint::Percentage(45),
         ].as_ref(),
      )
      .split(area);

   let hori_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
         [Constraint::Percentage(66), Constraint::Percentage(34)].as_ref(),
      )
      .split(vert_chunks[2]);

   main_rect.render_widget(tabs, vert_chunks[0]);
   main_rect.render_stateful_widget(table, vert_chunks[1], &mut app.mail_table.state);
   //main_rect.render_widget(top, vert_chunks[1]);
   main_rect.render_widget(bottom, hori_chunks[0]);
   main_rect.render_widget(right, hori_chunks[1]);

}



///
fn render_write(
   _chain: &SnapmailChain,
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   area: Rect,
   app: &mut App,
) {

   let subject_block = Paragraph::new("Subject")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match app.active_write_block {
               WriteBlock::Subject => Style::default().fg(Color::Yellow),
               _ => Style::default(),
            })
            .title("Subject")
            .border_type(BorderType::Plain),
      );

   let content_block = Paragraph::new("")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match app.active_write_block {
            WriteBlock::Content => Style::default().fg(Color::Yellow),
            _ => Style::default(),
         })
            .title("Content")
            .border_type(BorderType::Plain),
      );

   let attachment_block = Paragraph::new("Attachments")
      .alignment(Alignment::Center)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match app.active_write_block {
               WriteBlock::Attachments => Style::default().fg(Color::Yellow),
               _ => Style::default(),
            })
            .title("Attachments")
            .border_type(BorderType::Plain),
      );

   /// -- Set Mail Table

   let selected_style = Style::default().add_modifier(Modifier::REVERSED);
   //let normal_style = Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD);

   let rows = app.contacts_table.items.iter().map(|item| {
      let height = item
         .iter()
         .map(|content| content.chars().filter(|c| *c == '\n').count())
         .max()
         .unwrap_or(0)
         + 1;
      let cells = item.iter().map(|c| Cell::from(c.as_str()));
      Row::new(cells).height(height as u16).bottom_margin(0)//.horizontal_margin(2)
   });
   let table = Table::new(rows)
      //.header(header)
      .block(Block::default()
         .style(match app.active_write_block {
            WriteBlock::Contacts => Style::default().fg(Color::Yellow),
            _ => Style::default(),
         })
         .borders(Borders::ALL).title("Contacts"))
      .highlight_style(selected_style)
      //.highlight_symbol(">> ")
      .widths(&[
         //Constraint::Min(10),
         Constraint::Length(5),
         Constraint::Length(20),
      ]);


   /// - Cursor
   // match app.input_mode {
   //    InputMode::Normal =>
   //    // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
   //       {}
   //    InputMode::Editing => {
   //       // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
   //       main_rect.set_cursor(
   //          // Put cursor past the end of the input text
   //          settings_chunks[1].x + app.input.len() as u16 + 1,
   //          // Move one line down, from the border to the input line
   //          settings_chunks[1].y + 1,
   //       )
   //    }
   // }

   /// - Layout Render
   ///
   let hori_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
         [Constraint::Percentage(75), Constraint::Percentage(25)].as_ref(),
      )
      .split(area);

   let verti_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
         [
            Constraint::Length(3),
            Constraint::Percentage(75),
            Constraint::Percentage(25),
         ].as_ref(),
      )
      .split(hori_chunks[0]);

   main_rect.render_widget(subject_block, verti_chunks[0]);
   main_rect.render_widget(content_block, verti_chunks[1]);
   main_rect.render_widget(attachment_block, verti_chunks[2]);
   //main_rect.render_widget(right, write_chunks[1]);
   main_rect.render_stateful_widget(table, hori_chunks[1], &mut app.contacts_table.state);
}



///
fn render_settings(app: &App, main_rect: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
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

   let bottom = Paragraph::new(app.input.clone())
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match app.input_mode {
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
