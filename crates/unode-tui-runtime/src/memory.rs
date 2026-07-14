use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use unode_sdk::abi::AbiError;
use unode_sdk::{decode_json_bytes, encode_json_bytes};

#[derive(Debug, Error)]
pub enum TuiMemoryError {
    #[error("linear memory access out of bounds: ptr={ptr}, len={len}, memory_len={memory_len}")]
    OutOfBounds {
        ptr: usize,
        len: usize,
        memory_len: usize,
    },
    #[error(transparent)]
    Abi(#[from] AbiError),
}

pub fn read_bytes(memory: &[u8], ptr: u32, len: u32) -> Result<Vec<u8>, TuiMemoryError> {
    let start = ptr as usize;
    let count = len as usize;
    let end = start
        .checked_add(count)
        .ok_or(TuiMemoryError::OutOfBounds {
            ptr: start,
            len: count,
            memory_len: memory.len(),
        })?;

    if end > memory.len() {
        return Err(TuiMemoryError::OutOfBounds {
            ptr: start,
            len: count,
            memory_len: memory.len(),
        });
    }

    Ok(memory[start..end].to_vec())
}

pub fn read_json<T: DeserializeOwned>(memory: &[u8], ptr: u32, len: u32) -> Result<T, TuiMemoryError> {
    let bytes = read_bytes(memory, ptr, len)?;
    decode_json_bytes(&bytes).map_err(TuiMemoryError::from)
}

pub fn write_bytes(memory: &mut Vec<u8>, ptr: u32, bytes: &[u8]) -> Result<(), TuiMemoryError> {
    let start = ptr as usize;
    let end = start
        .checked_add(bytes.len())
        .ok_or(TuiMemoryError::OutOfBounds {
            ptr: start,
            len: bytes.len(),
            memory_len: memory.len(),
        })?;

    if end > memory.len() {
        memory.resize(end, 0);
    }

    memory[start..end].copy_from_slice(bytes);
    Ok(())
}

pub fn write_json<T: Serialize>(memory: &mut Vec<u8>, ptr: u32, value: &T) -> Result<usize, TuiMemoryError> {
    let bytes = encode_json_bytes(value)?;
    let len = bytes.len();
    write_bytes(memory, ptr, &bytes)?;
    Ok(len)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{read_bytes, read_json, write_bytes, write_json, TuiMemoryError};

    #[test]
    fn reads_and_writes_linear_memory() {
        let mut memory = vec![0; 8];
        write_bytes(&mut memory, 2, b"abcd").expect("write bytes");
        assert_eq!(read_bytes(&memory, 2, 4).expect("read bytes"), b"abcd");
    }

    #[test]
    fn grows_memory_when_writing_past_end() {
        let mut memory = vec![0; 2];
        write_bytes(&mut memory, 4, b"hi").expect("write bytes");
        assert_eq!(memory.len(), 6);
        assert_eq!(&memory[4..6], b"hi");
    }

    #[test]
    fn roundtrips_json_values() {
        let mut memory = vec![];
        let len = write_json(&mut memory, 0, &json!({ "ok": true })).expect("write json");
        let value = read_json::<serde_json::Value>(&memory, 0, len as u32).expect("read json");
        assert_eq!(value["ok"], true);
    }

    #[test]
    fn errors_on_out_of_bounds_reads() {
        let memory = vec![1, 2, 3];
        assert!(matches!(
            read_bytes(&memory, 2, 4),
            Err(TuiMemoryError::OutOfBounds { .. })
        ));
    }
}
