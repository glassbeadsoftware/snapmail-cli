use std::io;
use tui::{
   Frame,
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout, Rect},
   style::{Color, Modifier, Style},
   text::{Span, Spans},
   widgets::{
      Block, BorderType, Borders, Paragraph,
   },
};
use crate::app::{
   InputMode, App,
};

///
pub fn render_settings(
   app: &App,
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   area: Rect,
) {
   let settings_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
         [Constraint::Min(6), Constraint::Length(3)].as_ref(),
      )
      .split(area);

   let items = vec!["Handle", "UID", "Proxy URL", "Bootstrap URL", "Download Folder"];

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
               Span::styled(rest, Style::default()),
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
            .title("Settings")
            .border_type(BorderType::Plain),
      );

   let bottom = Paragraph::new(app.input.clone())
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(match app.input_mode {
               InputMode::Editing => Style::default().fg(Color::Yellow),
               _ => Style::default(),
            })
            .title(app.input_variable.to_string())
            .border_type(BorderType::Plain),
      );


   match app.input_mode {
      InputMode::Scrolling => {},
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

   /// Render
   main_rect.render_widget(top, settings_chunks[0]);
   main_rect.render_widget(bottom, settings_chunks[1]);
}
