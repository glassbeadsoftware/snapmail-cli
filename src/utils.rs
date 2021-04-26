use holochain_types::dna::*;
use snapmail::handle::GetAllHandlesOutput;

/// Print a msg with `snapmail: ` pre-pended
/// and ansi colors.
macro_rules! msg {
    ($($arg:tt)*) => ({
        use ansi_term::Color::*;
        print!("{} ", Blue.bold().paint("snapmail:"));
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

///
pub fn stohh(input: String) -> HeaderHash {
   let bytes = base64::decode_config(input[1..].to_string(), base64::URL_SAFE_NO_PAD).unwrap();
   //println!(" - bytes = {:?} ({})", bytes, bytes.len());
   let key: HeaderHash = HeaderHash::from_raw_39(bytes).unwrap();
   key
}

///
pub fn stoh(input: String) -> AgentPubKey {
   let bytes = base64::decode_config(input[1..].to_string(), base64::URL_SAFE_NO_PAD).unwrap();
   //println!(" - bytes = {:?} ({})", bytes, bytes.len());
   let key: AgentPubKey = AgentPubKey::from_raw_39(bytes).unwrap();
   key
}


/// Get username from AgentPubKey
pub fn get_name(handle_list: &GetAllHandlesOutput, candidate: &AgentPubKey) -> String {
   for pair in handle_list.0.iter() {
      if &pair.1 == candidate {
         return pair.0.clone();
      }
   }
   "<unknown>".to_string()
}