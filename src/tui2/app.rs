use std::str::FromStr;
use std::string::ToString;

#[derive(AsStaticStr, ToString, Copy, Clone, Debug, PartialEq)]
pub enum InputMode {
   Normal,
   Editing,
}

// impl FromStr for InputMode {
//    type Err = ();
//    fn from_str(input: &str) -> Result<EntryKind, Self::Err> {
//       match input {}
//    }
// }

/// App holds the state of the application
pub struct App {
   /// Current value of the input box
   pub input: String,
   /// Current input mode
   pub input_mode: InputMode,
   /// History of recorded messages
   pub messages: Vec<String>,
}

impl Default for App {
   fn default() -> App {
      App {
         input: String::new(),
         input_mode: InputMode::Normal,
         messages: Vec::new(),
      }
   }
}
