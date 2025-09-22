use serde::{de::DeserializeOwned, Serialize};

use crate::Error;

pub fn encode_bytes<T: Serialize>(obj: &T, buffer: &mut [u8]) -> Result<usize, Error> {
    if let Ok(encoded) = postcard::to_slice_cobs(obj, buffer) {
        if let Some(pos) = encoded.iter().position(|&b| b == 0) {
            Ok(pos)
        } else {
            Err(Error::BufferTooSmall)
        }
    } else {
        Err(Error::BufferTooSmall)
    }
}

pub fn decode_bytes<T: DeserializeOwned>(data: &[u8]) -> Result<T, Error> {
    let mut cloned_data: [u8; 256] = [0; 256];
    cloned_data.clone_from_slice(data);
    postcard::from_bytes_cobs(&mut cloned_data).map_err(|_| Error::InvalidCommand)
}
