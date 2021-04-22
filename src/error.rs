//use std::path::PathBuf;
use thiserror::Error;
use holochain_serialized_bytes::SerializedBytesError;
use holochain::core::ribosome::error::RibosomeError;

pub type SnapmailResult<T> = Result<T, SnapmailError>;

#[derive(Error, Debug)]
pub enum SnapmailError {
   #[error("data store disconnected")]
   Disconnect(#[from] std::io::Error),
   #[error("Internal serialization error: {0}")]
   SerializedBytesError(#[from] SerializedBytesError),
   #[error(transparent)]
   RibosomeError(#[from] RibosomeError),
   #[error("Holochain call timed out")]
   Timeout,
   #[error("Unauthorized zome call")]
   Unauthorized,
   #[error("Network error: {0}")]
   NetworkError(String),
   #[error("unknown data store error")]
   Unknown,
}