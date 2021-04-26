use holochain_types::dna::*;

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
pub fn stoh(input: String) -> AgentPubKey {
   let bytes = base64::decode_config(input[1..].to_string(), base64::URL_SAFE_NO_PAD).unwrap();
   //println!(" - bytes = {:?} ({})", bytes, bytes.len());
   let key: AgentPubKey = AgentPubKey::from_raw_39(bytes).unwrap();
   key
}