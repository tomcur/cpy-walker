pub use remoteprocess::Pid;

pub mod cpython27;
pub mod error;
pub mod interpreter;
pub mod memory;
pub mod walker;

use error::{Error, Result};

pub fn connect(pid: Pid) -> Result<memory::Process> {
    Ok(memory::Process::new(
        remoteprocess::Process::new(pid).map_err(Error::RemoteProcessConnect)?,
    ))
}
