use std::io;
use tui::{
   Frame,
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout, Rect},
   style::{Color, Modifier, Style},
   text::{Span, Spans},
   widgets::{
      Block, BorderType, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap,
   },
};
use crate::{
   snapmail_chain::*,
   app::App,
   app::InputMode,
};

///
pub fn render_view(
   chain: &SnapmailChain,
   main_rect: &mut Frame<CrosstermBackend<io::Stdout>>,
   area: Rect,
   app: &mut App,
) {

   /// - Layout
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
   let filebox_title = format!("Filebox: {} / {}", app.mail_table.items.len(), chain.mail_map.len());
   let tabs = Tabs::new(top_menu)
      .select(app.active_folder_item.to_owned().into())
      .block(Block::default().title(filebox_title).borders(Borders::ALL))
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().fg(Color::Yellow))
      .divider(Span::raw("|"));

   /// -- Set Mail Table
   let selected_style = Style::default().add_modifier(Modifier::REVERSED);
   let normal_style = Style::default().bg(Color::Magenta);

   //let header_cells = ["ID", "Username", "Subject", "Date", "Status"]
   let header_cells = ["", "From", "Subject", "Message", "Date"]
      .iter()
      .map(|h| Cell::from(*h).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
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
   let msg_width = area.width.saturating_sub(4 + 20 + 28 + 16 + 5);
   let mail_table_widths = [
      Constraint::Length(4),
      Constraint::Length(20),
      Constraint::Length(28),
      Constraint::Length(msg_width),
      Constraint::Length(16),
   ];
   let table = Table::new(rows)
      .header(header)
      .block(Block::default().borders(Borders::NONE).title(""))
      .highlight_style(selected_style)
      //.highlight_symbol(">> ")
      .widths(&mail_table_widths);

   /// -- Draw selected mail
   let mail_txt = if let Some(index) = app.mail_table.state.selected() {
      app.mail_table.get_mail_text(index, &chain)
   } else {
      "<No Mail Selected>".to_string()
   };
   let mail_content_block = Paragraph::new(mail_txt)
      .wrap(Wrap {trim: false})
      .scroll((app.scroll_y, 0))
      .alignment(Alignment::Left)
      .block(
         Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(match app.input_mode {
               InputMode::Scrolling => Color::Yellow,
               _ => Color::White,}
            ))
            .title("Mail")
            .border_type(BorderType::Plain),
      );

   /// -- Draw Attachments table
   let selected_style = Style::default().add_modifier(Modifier::REVERSED);
   let att_rows = app.attachments_table.items.iter().map(|item| {
      let height = item
         .iter()
         .map(|content| content.chars().filter(|c| *c == '\n').count())
         .max()
         .unwrap_or(0)
         + 1;
      let cells = item.iter()
         .map(|c|
            Cell::from(c.as_str())
            .style(Style::default().fg(if c.chars().last() == Some('.') {Color::Yellow} else {Color::White})
            ));
      Row::new(cells).height(height as u16).bottom_margin(0)
   });
   let att_name_len = hori_chunks[1].width.saturating_sub(3 + 9 + 2);
   let att_table_widths = [
      Constraint::Length(3),
      Constraint::Length(att_name_len),
      Constraint::Length(9),
   ];
   let att_table = Table::new(att_rows)
      //.header(header)
      .block(Block::default()
         .borders(Borders::ALL).title("Attachments"))
      .highlight_style(selected_style)
      //.highlight_symbol(">> ")
      .widths(&att_table_widths);

   /// - Render
   main_rect.render_widget(tabs, vert_chunks[0]);
   main_rect.render_stateful_widget(table, vert_chunks[1], &mut app.mail_table.state);
   main_rect.render_widget(mail_content_block, hori_chunks[0]);
   main_rect.render_widget(att_table, hori_chunks[1]);

   /// - Notify resize event
   if msg_width != app.content_width as u16 {
      app.resize_width(msg_width, chain)
   }
}
