#[derive(thiserror::Error, Debug)]
pub enum Ft60xError {
    #[error("no matching device found")]
    NoMatchingDevice,
    #[error("unknown error")]
    Unknown,
    #[error("invalid fifo mode configuration")]
    InvalidFifoMode,
    #[error("invalid fifo clock configuration")]
    InvalidFifoClock,
    #[error("invalid channel configuration")]
    InvalidChannel,
    #[error("failed to read data")]
    ReadError,
    #[error("failed to write data")]
    WriteError,
}
