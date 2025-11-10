use thiserror::Error;

  #[derive(Debug, Error)]
  pub enum CodecError {
      #[error("invalid payload")]
      InvalidPayload,
      #[error("internal error: {0}")]
      Internal(String),
  }

  pub type Result<T> = std::result::Result<T, CodecError>;
