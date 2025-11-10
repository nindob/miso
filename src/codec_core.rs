use crate::errors::Result;

  pub struct CodecCore;

  impl CodecCore {
      pub fn encode_token_ids(ids: &[i32], gzip: bool) -> Result<Vec<u8>> {
          let _ = gzip;
          unimplemented!()
      }

      pub fn decode_token_ids(payload: &[u8], gzip: bool) -> Result<Vec<i32>> {
          let _ = gzip;
          unimplemented!()
      }
  }