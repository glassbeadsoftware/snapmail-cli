use thiserror::Error;

pub type SnapmailResult<T> = Result<T, SnapmailError>;

#[derive(Error, Debug)]
pub enum SnapmailError {
   #[error("unknown data store error")]
   Unknown,
}