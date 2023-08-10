use std::fmt::{Debug, Display};

use crate::ffi::types::FT_STATUS;

/// Error type corresponding to possible [`FT_STATUS`] errors
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum D3xxError {
    // Errors defined by the D3XX library
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

    // Errors not defined by the D3XX library
    LibraryLoadFailed,
}

impl From<FT_STATUS> for D3xxError {
    /// Convert from a raw status value to a `D3xxError`.
    ///
    /// # Panics
    /// Panics if the given value is not a valid status value.
    fn from(id: FT_STATUS) -> Self {
        match id {
            1 => D3xxError::InvalidHandle,
            2 => D3xxError::DeviceNotFound,
            3 => D3xxError::DeviceNotOpened,
            4 => D3xxError::IoError,
            5 => D3xxError::InsufficientResources,
            6 => D3xxError::InvalidParameter,
            7 => D3xxError::InvalidBaudRate,
            8 => D3xxError::DeviceNotOpenedForErase,
            9 => D3xxError::DeviceNotOpenedForWrite,
            10 => D3xxError::FailedToWriteDevice,
            11 => D3xxError::EEPROMReadFailed,
            12 => D3xxError::EEPROMWriteFailed,
            13 => D3xxError::EEPROMEraseFailed,
            14 => D3xxError::EEPROMNotPresent,
            15 => D3xxError::EEPROMNotProgrammed,
            16 => D3xxError::InvalidArgs,
            17 => D3xxError::NotSupported,
            18 => D3xxError::NoMoreItems,
            19 => D3xxError::Timeout,
            20 => D3xxError::OperationAborted,
            21 => D3xxError::ReservedPipe,
            22 => D3xxError::InvalidControlRequestDirection,
            23 => D3xxError::InvalidControLRequestType,
            24 => D3xxError::IoPending,
            25 => D3xxError::IoIncomplete,
            26 => D3xxError::HandleEof,
            27 => D3xxError::Busy,
            28 => D3xxError::NoSystemResources,
            29 => D3xxError::DeviceListNotReady,
            30 => D3xxError::DeviceNotConnected,
            31 => D3xxError::IncorrectDevicePath,
            32 => D3xxError::OtherError,
            _ => panic!("Unknown value {}", id),
        }
    }
}

impl Display for D3xxError {
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

            Self::LibraryLoadFailed => "LibraryLoadFailed",
        };
        write!(f, "{} (error code {})", name, *self as u32)
    }
}
