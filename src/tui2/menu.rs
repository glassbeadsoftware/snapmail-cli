use std::str::FromStr;

use tui::{
   backend::CrosstermBackend,
   layout::{Alignment, Constraint, Direction, Layout},
   style::{Color, Modifier, Style},
   text::{Span, Spans},
   widgets::{
      Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
   },
   Terminal,
};

#[derive(Copy, Clone, Debug)]
pub enum TopMenuItem {
   View,
   Write,
   Settings,
}

impl From<TopMenuItem> for usize {
   fn from(input: TopMenuItem) -> usize {
      match input {
         TopMenuItem::View => 0,
         TopMenuItem::Write => 1,
         TopMenuItem::Settings => 2,
      }
   }
}


#[derive(Copy, Clone, Debug)]
pub enum WriteMenuItem {
   Clear,
   AddAttachment,
   Send,
}

impl From<WriteMenuItem> for usize {
   fn from(input: WriteMenuItem) -> usize {
      match input {
         WriteMenuItem::Clear => 0,
         WriteMenuItem::AddAttachment => 1,
         WriteMenuItem::Send => 2,
      }
   }
}


#[derive(Copy, Clone, Debug)]
pub enum ViewMenuItem {
   Delete,
   Reply,
   Download,
}

impl From<ViewMenuItem> for usize {
   fn from(input: ViewMenuItem) -> usize {
      match input {
         ViewMenuItem::Delete => 0,
         ViewMenuItem::Reply => 1,
         ViewMenuItem::Download => 2,
      }
   }
}