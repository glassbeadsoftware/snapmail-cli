use holochain_types::dna::*;
use snapmail::handle::GetAllHandlesOutput;

/// Print a msg with `snapmail: ` pre-pended
/// and ansi colors.
macro_rules! msg {
    ($($arg:tt)*) => ({
        use ansi_term::Color::*;
        let now = chrono::Utc::now().format("%H:%M:%S");
        let prepend = format!("[{}] snapmail: ", now);
        print!("{}", Blue.bold().paint(prepend));
        println!($($arg)*);
    })
}

#[allow(unused_macros)]
macro_rules! dbg {
    ($($arg:tt)*) => ({
        use ansi_term::Color::*;
        print!("{} ", Yellow.bold().paint("snap-dbg:"));
        println!($($arg)*);
    })
}

#[allow(unused_macros)]
macro_rules! err_msg {
    ($($arg:tt)*) => ({
        use ansi_term::Color::*;
        print!("{} ", Red.bold().paint("snapmail error:"));
        println!($($arg)*);
    })
}

///
pub fn stoh<T: holochain_types::dna::PrimitiveHashType>(input: String) -> HoloHash<T> {
   let bytes = base64::decode_config(input[1..].to_string(), base64::URL_SAFE_NO_PAD).unwrap();
   let key: HoloHash<T> = HoloHash::<T>::from_raw_39(bytes).unwrap();
   key
}


/// Get username from AgentPubKey
pub fn get_name(handle_list: &GetAllHandlesOutput, candidate: &AgentPubKey) -> Option<String> {
   for pair in handle_list.0.iter() {
      if &pair.1 == candidate {
         return Some(pair.0.clone());
      }
   }
   None
}

/// Get username from AgentPubKey
pub fn get_agent_id(handle_list: &GetAllHandlesOutput, candidate: &str) -> Option<AgentPubKey> {
   for pair in handle_list.0.iter() {
      if &pair.0 == candidate {
         return Some(pair.1.clone());
      }
   }
   None
}