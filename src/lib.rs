mod zigzag;
mod varint;
mod freq_map;
mod codec_core;
mod errors;

use pyo3::prelude::*;

  #[pyclass]
  pub struct Codec;

  #[pymethods]
  impl Codec {
      #[new]
      pub fn new() -> Self {
          Codec
      }

      pub fn ping(&self) -> PyResult<String> {
          Ok("pong".to_string())
      }

      pub fn encode_token_ids(&self, _token_ids: Vec<i32>, _gzip: bool) -> PyResult<Vec<u8>> {
          unimplemented!()
      }

      pub fn decode_token_ids(&self, _payload: Vec<u8>, _gzip: bool) -> PyResult<Vec<i32>> {
          unimplemented!()
      }
  }

  #[pymodule]
  fn miso(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
      m.add_class::<Codec>()?;
      Ok(())
  }