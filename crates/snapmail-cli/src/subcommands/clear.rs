use snapmail_common::globals::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

///
pub fn clear(uid: PathBuf) {
   let dir = Path::new(&*CONFIG_PATH).join(uid);
   let result = fs::remove_dir_all(dir.as_path());
   if let Err(e) = result {
      msg!("Clear failed: {}", e);
      return;
   }
   msg!("Clear done");
}
