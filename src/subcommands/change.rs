use structopt::StructOpt;
use std::path::PathBuf;
use crate::{
   globals::*,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

/// Change conductor config
#[derive(Debug, StructOpt, Clone)]
pub struct ChangeCommand {
   #[structopt(name = "proxy", long, parse(from_os_str))]
   /// Url of proxy server to use
   maybe_proxy: Option<PathBuf>,
   #[structopt(name = "bootstrap", long, parse(from_os_str))]
   /// Url of bootstrap server to use
   maybe_bootstrap: Option<PathBuf>,
   #[structopt(name = "uid", long)]
   /// Network ID that this session will use (String)
   maybe_uid: Option<String>,
}

impl ChangeCommand {
   ///
   pub fn run(&self, sid: PathBuf) {
      if let Some(uid) = &self.maybe_uid {
         self.update_uid(sid.clone(), uid);
      }
      if self.maybe_bootstrap.is_some() || self.maybe_proxy.is_some() {
         self.update_conductor_config(sid);
      }
   }

   ///
   fn update_uid(&self, sid: PathBuf, uid: &str) {
      let config_path = Path::new(&*CONFIG_PATH).join(sid.clone());
      let app_filepath = config_path.join(APP_CONFIG_FILENAME);
      std::fs::write(app_filepath, uid.as_bytes()).unwrap();
   }

   ///
   fn update_conductor_config(&self, sid: PathBuf) {
      lazy_static! {
        //static ref PORT_RE: Regex = Regex::new(r"port: (.*)").unwrap();
        static ref BOOTSTRAP_RE: Regex = Regex::new(r"bootstrap_service: (.*)").unwrap();
        static ref PROXY_RE: Regex = Regex::new(r"proxy_url: (.*)").unwrap();
      }
      /// Load config
      let path = CONFIG_PATH.as_path().join(sid);
      let config_filepath = path.join(CONDUCTOR_CONFIG_FILENAME);
      let mut file_str = std::fs::read_to_string(config_filepath.clone())
         .expect("Something went wrong reading CONDUCTOR CONFIG file");
      /// Get proxy server URL
      if let Some(proxy_url) = &self.maybe_proxy {
         let new_entry = format!("proxy_url: {}", proxy_url.to_string_lossy());
         file_str = PROXY_RE.replace_all(&file_str, new_entry).to_string();
         msg!("New config:\n{}", file_str);
      }
      /// Get bootstrap server URL
      if let Some(url) = &self.maybe_bootstrap {
         let new_entry = format!("bootstrap_service: \"{}\"", url.to_string_lossy());
         file_str = BOOTSTRAP_RE.replace_all(&file_str, new_entry).to_string();
      }
      /// Write config
      msg!("New config:\n{}", file_str);
      std::fs::write(config_filepath.clone(), file_str).unwrap();
   }
}
