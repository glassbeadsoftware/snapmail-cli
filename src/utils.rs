
/// Print a msg with `snapmail: ` pre-pended
/// and ansi colors.
macro_rules! msg {
    ($($arg:tt)*) => ({
        use ansi_term::Color::*;
        print!("{} ", Blue.bold().paint("snapmail:"));
        println!($($arg)*);
    })
}
