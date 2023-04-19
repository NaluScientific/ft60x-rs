use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    InvalidHandle = 1,
    DeviceNotFound = 2,
    DeviceNotOpened = 3,
    IoError = 4,
    InsufficientResources = 5,
    InvalidParameter = 6,
    InvalidBaudRate = 7,
    DeviceNotOpenedForErase = 8,
    DeviceNotOpenedForWrite = 9,
    FailedToWriteDevice = 10,
    EEPROMReadFailed = 11,
    EEPROMWriteFailed = 12,
    EEPROMEraseFailed = 13,
    EEPROMNotPresent = 14,
    EEPROMNotProgrammed = 15,
    InvalidArgs = 16,
    NotSupported = 17,

    NoMoreItems = 18,
    Timeout = 19,
    OperationAborted = 20,
    ReservedPipe = 21,
    InvalidControlRequestDirection = 22,
    InvalidControLRequestType = 23,
    IoPending = 24,
    IoIncomplete = 25,
    HandleEof = 26,
    Busy = 27,
    NoSystemResources = 28,
    DeviceListNotReady = 29,
    DeviceNotConnected = 30,
    IncorrectDevicePath = 31,

    OtherError = 32,
}

impl From<u32> for Error {
    /// Convert from a raw status value to a `D3xxError`.
    ///
    /// # Panics
    /// Panics if the given value is not a valid status value.
    fn from(id: u32) -> Self {
        match id {
            1 => Error::InvalidHandle,
            2 => Error::DeviceNotFound,
            3 => Error::DeviceNotOpened,
            4 => Error::IoError,
            5 => Error::InsufficientResources,
            6 => Error::InvalidParameter,
            7 => Error::InvalidBaudRate,
            8 => Error::DeviceNotOpenedForErase,
            9 => Error::DeviceNotOpenedForWrite,
            10 => Error::FailedToWriteDevice,
            11 => Error::EEPROMReadFailed,
            12 => Error::EEPROMWriteFailed,
            13 => Error::EEPROMEraseFailed,
            14 => Error::EEPROMNotPresent,
            15 => Error::EEPROMNotProgrammed,
            16 => Error::InvalidArgs,
            17 => Error::NotSupported,
            18 => Error::NoMoreItems,
            19 => Error::Timeout,
            20 => Error::OperationAborted,
            21 => Error::ReservedPipe,
            22 => Error::InvalidControlRequestDirection,
            23 => Error::InvalidControLRequestType,
            24 => Error::IoPending,
            25 => Error::IoIncomplete,
            26 => Error::HandleEof,
            27 => Error::Busy,
            28 => Error::NoSystemResources,
            29 => Error::DeviceListNotReady,
            30 => Error::DeviceNotConnected,
            31 => Error::IncorrectDevicePath,
            32 => Error::OtherError,
            _ => panic!("Unknown value {}", id),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match *self {
            Self::InvalidHandle => "InvalidHandle",
            Self::DeviceNotFound => "DeviceNotFound",
            Self::DeviceNotOpened => "DeviceNotOpened",
            Self::IoError => "IoError",
            Self::InsufficientResources => "InsufficientResources",
            Self::InvalidParameter => "InvalidParameter",
            Self::InvalidBaudRate => "InvalidBaudRate",
            Self::DeviceNotOpenedForErase => "DeviceNotOpenedForErase",
            Self::DeviceNotOpenedForWrite => "DeviceNotOpenedForWrite",
            Self::FailedToWriteDevice => "FailedToWriteDevice",
            Self::EEPROMReadFailed => "EEPROMReadFailed",
            Self::EEPROMWriteFailed => "EEPROMWriteFailed",
            Self::EEPROMEraseFailed => "EEPROMEraseFailed",
            Self::EEPROMNotPresent => "EEPROMNotPresent",
            Self::EEPROMNotProgrammed => "EEPROMNotProgrammed",
            Self::InvalidArgs => "InvalidArgs",
            Self::NotSupported => "NotSupported",
            Self::NoMoreItems => "NoMoreItems",
            Self::Timeout => "Timeout",
            Self::OperationAborted => "OperationAborted",
            Self::ReservedPipe => "ReservedPipe",
            Self::InvalidControlRequestDirection => "InvalidControlRequestDirection",
            Self::InvalidControLRequestType => "InvalidControLRequestType",
            Self::IoPending => "IoPending",
            Self::IoIncomplete => "IoIncomplete",
            Self::HandleEof => "HandleEof",
            Self::Busy => "Busy",
            Self::NoSystemResources => "NoSystemResources",
            Self::DeviceListNotReady => "DeviceListNotReady",
            Self::DeviceNotConnected => "DeviceNotConnected",
            Self::IncorrectDevicePath => "IncorrectDevicePath",
            Self::OtherError => "OtherError",
        };
        write!(f, "{} ({})", name, *self as u32)
    }
}

/// Returns `Ok(())` if the given status is not an error, otherwise
/// returns a corresponding `D3xxError`.
macro_rules! d3xx_error {
    ($status:expr) => {
        match $status {
            0 => Ok::<(), Error>(()),
            _ => Err(Error::from($status as u32)),
        }
    };
}

pub(crate) use d3xx_error;
