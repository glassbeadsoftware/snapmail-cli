
#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FolderItem {
   Inbox,
   Sent,
   Trash,
   All,
}

impl From<FolderItem> for usize {
   fn from(input: FolderItem) -> usize {
      match input {
         FolderItem::Inbox => 0,
         FolderItem::Sent => 1,
         FolderItem::Trash => 2,
         FolderItem::All => 3,
      }
   }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WriteBlock {
   Content,
   Contacts,
   Attachments,
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