use crate::globals::*;
use std::fs;

///
pub fn clear() {
   let result = fs::remove_dir_all(CONFIG_PATH.as_path());
   if let Err(e) = result {
      msg!("Clear failed: {}", e);
      return;
   }
   msg!("Clear done");
}
