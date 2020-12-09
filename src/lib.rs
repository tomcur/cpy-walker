//! # Example
//!
//! ```
//! use std::io::{BufRead, BufReader};
//! use std::path::PathBuf;
//! use std::process::{Command, Stdio};
//!
//! use cpy_walker::cpython27::*;
//! use cpy_walker::interpreter::*;
//! use cpy_walker::memory::{Memory, Process};
//! use cpy_walker::walker::walk;
//!
//! /// This spawns a Python process, but you can connect to pre-existing processes as well.
//! fn spawn_child() -> Result<(i32, usize), Box<dyn std::error::Error>> {
//!     let child = Command::new(
//!         [env!("CARGO_MANIFEST_DIR"), "test-programs", "python27.py"]
//!             .iter()
//!             .collect::<PathBuf>(),
//!     )
//!     .stdin(Stdio::piped())
//!     .stdout(Stdio::piped())
//!     .stderr(Stdio::null())
//!     .spawn()?;
//!
//!     let pid = child.id();
//!     let stdout = child.stdout.unwrap();
//!
//!     let mut line = String::new();
//!     BufReader::new(stdout).read_line(&mut line)?;
//!     let pointer: usize = line.trim().parse().expect("memory address");
//!
//!     Ok((pid as i32, pointer))
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (pid, pointer) = spawn_child()?;
//!     let mem = cpy_walker::connect(pid)?;
//!     let ptr = Pointer::new(pointer);
//!
//!     println!(
//!         "Data graph: {:#x?}",
//!         walk::<Cpython2_7, _>(&mem, ptr)
//!     );
//!
//!     Ok(())
//! }
//! ```

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
