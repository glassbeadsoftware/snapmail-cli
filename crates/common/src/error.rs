use thiserror::Error;
//use serde;

pub type SnapmailResult<T> = Result<T, SnapmailError>;

#[derive(Error, Debug)]
pub enum SnapmailError {
   #[error("error reading the DB file: {0}")]
   ReadDBError(#[from] std::io::Error),
   #[error("error parsing the DB file: {0}")]
   ParseDBError(#[from] serde_json::Error),
   #[error("unknown data store error")]
   Unknown,
}