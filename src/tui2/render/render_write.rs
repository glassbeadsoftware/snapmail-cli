use std::io;
use tui::{
   Frame,
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout, Rect},
   style::{Color, Modifier, Style},
   widgets::{
      Block, BorderType, Borders, Cell, Paragraph, Row, Table,
   },
};
use crate::{
   tui2::*,
   tui2::menu::*,
};

///
pub fn render_write(
   _chain: &SnapmailChain,
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   area: Rect,
   app: &mut App,
) {
   /// Subject Block
   let current_subject = if app.active_write_block == WriteBlock::Subject {
      app.input.clone()
   } else {
      app.write_subject.clone()
   };
   let subject_block = Paragraph::new(current_subject)
      .alignment(Alignment::Left)
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

   /// Attachment Block
   let current_content = if app.active_write_block == WriteBlock::Content {
      app.input.clone()
   } else {
      app.write_content.clone()
   };
   let content_block = Paragraph::new(current_content)
      .alignment(Alignment::Left)
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

   /// Attachment Block
   // let empty_path = PathBuf::new();
   // let current_attachment = if app.active_write_block == WriteBlock::Attachments {
   //    PathBuf::from(app.input.clone())
   // } else {
   //    app.write_attachments.first().unwrap_or(&empty_path).to_owned()
   // };
   // current_attachment.as_os_str().to_str().unwrap()

   let current_attachment = if app.active_write_block == WriteBlock::Attachments {
      app.input.clone()
   } else {
      app.write_attachment.clone()
   };
   let attachment_block = Paragraph::new(current_attachment)
      .alignment(Alignment::Left)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match app.active_write_block {
               WriteBlock::Attachments => Style::default().fg(Color::Yellow),
               _ => Style::default(),
            })
            .title("Attachment")
            .border_type(BorderType::Plain),
      );

   /// - Contacts Table
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

   /// - Layout
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
            Constraint::Min(10),
            Constraint::Length(3),
         ].as_ref(),
      )
      .split(hori_chunks[0]);

   /// - Cursor
   match app.input_mode {
      InputMode::Normal => {},
      InputMode::Editing => {
         if app.active_write_block != WriteBlock::Contacts {
            let (x_offset, y_offset) = if app.active_write_block == WriteBlock::Content {
               //             app.input.as_bytes().iter().filter(|&&c| c == b'\n').count() as u16
               let lines: Vec<&str> = app.input.lines().collect();
               let x = if let Some(line) = lines.last() {
                  line.len() as u16
               } else { 0 };
               let y = std::cmp::max(0, lines.len() as i32 - 1) as u16;
               (x, y)
            } else {
               (app.input.len() as u16, 0)
            };

            let index =
               match app.active_write_block {
                  WriteBlock::Subject => 0,
                  WriteBlock::Content => 1,
                  WriteBlock::Attachments => 2,
                  _ => 0,
               };

            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            main_rect.set_cursor(
               // Put cursor past the end of the input text
               verti_chunks[index].x + 1 + x_offset,
               // Move one line down, from the border to the input line
               verti_chunks[index].y + 1 + y_offset,
            )
         }
      }
   }

   /// - Render
   main_rect.render_widget(subject_block, verti_chunks[0]);
   main_rect.render_widget(content_block, verti_chunks[1]);
   main_rect.render_widget(attachment_block, verti_chunks[2]);
   main_rect.render_stateful_widget(table, hori_chunks[1], &mut app.contacts_table.state);
}
