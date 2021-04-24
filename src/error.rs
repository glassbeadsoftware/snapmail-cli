//use std::path::PathBuf;
use thiserror::Error;
// use holochain_serialized_bytes::SerializedBytesError;
// use holochain::core::ribosome::error::RibosomeError;
// use holochain::conductor::error::*;

pub type SnapmailResult<T> = Result<T, SnapmailError>;

#[derive(Error, Debug)]
pub enum SnapmailError {
   #[error("unknown data store error")]
   Unknown,
}