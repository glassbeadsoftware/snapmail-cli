use std::io;
use tui::{
   Frame,
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout},
   style::{Color, Modifier, Style},
   text::{Span, Spans},
   widgets::{
      Block, BorderType, Borders, Paragraph, Tabs,
   },
};
use crate::{
   tui2::*,
   tui2::menu::*,
   globals::*,
   tui2::render::*,
};

///
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
      .margin(0)
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

   let title = format!("Snapmail {} - {} - {} - {} - {}", SNAPMAIL_VERSION,
                       app.sid, app.uid, chain.my_handle.clone(), app.frame_count);
   let tabs = Tabs::new(top_menu)
      .select(app.active_menu_item.to_owned().into())
      .block(Block::default().title(title).borders(Borders::ALL))
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().fg(Color::Yellow))
      .divider(Span::raw("|"));
   main_rect.render_widget(tabs, chunks[0]);

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
