use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Attempted to access invalid memory.")]
    SegmentationFault(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Attempted to dereference a null pointer.")]
    NullPointer,
    #[error("Attempted to decode seemingly invalid memory. Perhaps the target has been garbage collected?")]
    Decode,
    #[error("Could not connect to remote process.")]
    RemoteProcessConnect(#[source] remoteprocess::Error),
}
