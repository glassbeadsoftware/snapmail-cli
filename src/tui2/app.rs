use std::string::ToString;
use std::sync::RwLock;

#[derive(AsStaticStr, ToString, Copy, Clone, Debug, PartialEq)]
pub enum InputMode {
   Normal,
   Editing,
}

#[derive(AsStaticStr, ToString, Copy, Clone, Debug, PartialEq)]
pub enum InputVariable {
   BoostrapUrl,
   ProxyUrl,
   Handle,
   Uid,
   Mail,
   Attachment,
}

/// App holds the state of the application
pub struct App {
   /// Current value of the input box
   pub input: String,
   /// Current input mode
   pub input_mode: InputMode,
   /// Current settings to change
   pub input_variable: InputVariable,
   /// History of recorded messages
   pub messages: Vec<String>,
}

impl Default for App {
   fn default() -> App {

      App {
         input: String::new(),
         input_mode: InputMode::Normal,
         input_variable: InputVariable::Mail,
         messages: vec!["Welcome to Snapmail".to_string()],
      }
   }
}

// lazy_static! {
//    /// Create default app state
//    pub static ref g_app: RwLock<App> = RwLock::new(App::default());
// }
