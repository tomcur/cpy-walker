use remoteprocess::ProcessMemory;
use std::convert::TryInto;
use thiserror::Error;

use crate::error::{Error, Result};

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Invalid size: {0}")]
    InvalidSize(String),
}

pub trait Memory {
    /// `address` and `size` are in bytes.
    fn get_vec(&self, address: usize, size: usize) -> Result<Vec<u8>>;

    fn get_u16_vec(&self, address: usize, size: usize) -> Result<Vec<u16>> {
        if size % 2 != 0 {
            return Err(Error::SegmentationFault(Box::new(
                MemoryError::InvalidSize("must be multiple of 2".to_string()),
            )));
        }
        let vec = self.get_vec(address, size)?;
        let mut result = Vec::with_capacity(size / 2);
        for idx in 0..size / 2 {
            result.push(u16::from_le_bytes([vec[idx], vec[idx + 1]]))
        }
        Ok(result)
    }

    /// Address is in bytes.
    fn get_u8(&self, address: usize) -> Result<u8> {
        Ok(self.get_vec(address, 1)?[0])
    }

    /// Address is in bytes.
    /// Reads and decodes a C String up to a null terminator (optionally of
    /// length `max_length`).
    fn get_c_str(&self, address: usize, max_length: Option<usize>) -> Result<String> {
        let mut chars = Vec::<char>::new();
        let length = match max_length {
            Some(length) => length,
            None => usize::MAX,
        };
        for offset in 0..length {
            let byte = self.get_u8(address + offset)?;
            if byte == 0 {
                break;
            }
            chars.push(byte.into());
        }

        Ok(chars.into_iter().collect::<String>())
    }

    // Address is in bytes.
    fn get_u64_array(&self, address: usize) -> Result<[u8; 8]> {
        Ok(self.get_vec(address, 8)?.try_into().unwrap())
    }

    // Address is in bytes.
    fn get_u64(&self, address: usize) -> Result<u64> {
        Ok(u64::from_le_bytes(self.get_u64_array(address)?))
    }

    // Address is in bytes.
    fn get_usize(&self, address: usize) -> Result<usize> {
        Ok(self.get_u64(address)? as usize)
    }

    // Address is in bytes.
    fn get_isize(&self, address: usize) -> Result<isize> {
        Ok(self.get_u64(address)? as isize)
    }
}

pub struct Process {
    process: remoteprocess::Process,
}

impl Process {
    pub fn new(process: remoteprocess::Process) -> Self {
        Self { process }
    }
}

impl Memory for Process {
    fn get_vec(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        self.process
            .copy(address, size)
            .map_err(|e| Error::SegmentationFault(e.into()))
    }
}
