use url2::Url2;
use directories::ProjectDirs;
use std::path::PathBuf;
//use std::path::Path;

pub const SNAPMAIL_VERSION: &str           = "v0.0.4";
pub const SNAPMAIL_APP_ID: &str            = "snapmail-app";
pub const SNAPMAIL_APP_NAME: &str          = "snapmail-cli";
pub const LAIR_MAGIC_READY_STRING: &str    = "#lair-keystore-ready#";
pub const CONDUCTOR_CONFIG_FILENAME: &str  = "conductor-config.yaml";
pub const APP_CONFIG_FILENAME: &str        = "app-config.txt";

lazy_static! {
   pub static ref DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(9);
   pub static ref CONFIG_PATH: PathBuf = ProjectDirs::from("", "", SNAPMAIL_APP_NAME).unwrap().config_dir().to_owned();
   //pub static ref CONDUCTOR_CONFIG_FILEPATH: PathBuf = Path::new(&*CONFIG_PATH).join(CONDUCTOR_CONFIG_FILENAME);
   pub static ref DEFAULT_PROXY_URL: Url2 =  url2!("kitsune-proxy://VYgwCrh2ZCKL1lpnMM1VVUee7ks-9BkmW47C_ys4nqg/kitsune-quic/h/kitsune-proxy.harris-braun.com/p/4010/--");
   pub static ref DEFAULT_BOOTSTRAP_URL: Url2 =  url2!("https://bootstrap-staging.holo.host");
}
