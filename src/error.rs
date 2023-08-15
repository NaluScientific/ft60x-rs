use std::fmt::{Debug, Display};

use crate::ffi::types::FT_STATUS;

/// Error type corresponding to possible [`FT_STATUS`] errors
#[derive(thiserror::Error, Debug)]
pub enum D3xxError {
    // Errors defined by the D3XX library
    InvalidHandle,
    DeviceNotFound,
    DeviceNotOpened,
    IoError,
    InsufficientResources,
    InvalidParameter,
    InvalidBaudRate,
    DeviceNotOpenedForErase,
    DeviceNotOpenedForWrite,
    FailedToWriteDevice,
    EEPROMReadFailed,
    EEPROMWriteFailed,
    EEPROMEraseFailed,
    EEPROMNotPresent,
    EEPROMNotProgrammed,
    InvalidArgs,
    NotSupported,

    NoMoreItems,
    Timeout,
    OperationAborted,
    ReservedPipe,
    InvalidControlRequestDirection,
    InvalidControLRequestType,
    IoPending,
    IoIncomplete,
    HandleEof,
    Busy,
    NoSystemResources,
    DeviceListNotReady,
    DeviceNotConnected,
    IncorrectDevicePath,

    OtherError,

    // Errors not defined by the D3XX library
    LibraryAccessFailed(#[from] libloading::Error),
    UnpackingFailed(#[from] std::io::Error),
    LibraryAlreadyLoaded,
    LibraryNotLoaded,
}

impl D3xxError {
    pub fn error_code(&self) -> Option<u32> {
        match self {
            D3xxError::InvalidHandle => Some(1),
            D3xxError::DeviceNotFound => Some(2),
            D3xxError::DeviceNotOpened => Some(3),
            D3xxError::IoError => Some(4),
            D3xxError::InsufficientResources => Some(5),
            D3xxError::InvalidParameter => Some(6),
            D3xxError::InvalidBaudRate => Some(7),
            D3xxError::DeviceNotOpenedForErase => Some(8),
            D3xxError::DeviceNotOpenedForWrite => Some(9),
            D3xxError::FailedToWriteDevice => Some(10),
            D3xxError::EEPROMReadFailed => Some(11),
            D3xxError::EEPROMWriteFailed => Some(12),
            D3xxError::EEPROMEraseFailed => Some(13),
            D3xxError::EEPROMNotPresent => Some(14),
            D3xxError::EEPROMNotProgrammed => Some(15),
            D3xxError::InvalidArgs => Some(16),
            D3xxError::NotSupported => Some(17),
            D3xxError::NoMoreItems => Some(18),
            D3xxError::Timeout => Some(19),
            D3xxError::OperationAborted => Some(20),
            D3xxError::ReservedPipe => Some(21),
            D3xxError::InvalidControlRequestDirection => Some(22),
            D3xxError::InvalidControLRequestType => Some(23),
            D3xxError::IoPending => Some(24),
            D3xxError::IoIncomplete => Some(25),
            D3xxError::HandleEof => Some(26),
            D3xxError::Busy => Some(27),
            D3xxError::NoSystemResources => Some(28),
            D3xxError::DeviceListNotReady => Some(29),
            D3xxError::DeviceNotConnected => Some(30),
            D3xxError::IncorrectDevicePath => Some(31),
            D3xxError::OtherError => Some(32),
            _ => None,
        }
    }
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
        let name = match self {
            Self::InvalidHandle => "InvalidHandle".to_owned(),
            Self::DeviceNotFound => "DeviceNotFound".to_owned(),
            Self::DeviceNotOpened => "DeviceNotOpened".to_owned(),
            Self::IoError => "IoError".to_owned(),
            Self::InsufficientResources => "InsufficientResources".to_owned(),
            Self::InvalidParameter => "InvalidParameter".to_owned(),
            Self::InvalidBaudRate => "InvalidBaudRate".to_owned(),
            Self::DeviceNotOpenedForErase => "DeviceNotOpenedForErase".to_owned(),
            Self::DeviceNotOpenedForWrite => "DeviceNotOpenedForWrite".to_owned(),
            Self::FailedToWriteDevice => "FailedToWriteDevice".to_owned(),
            Self::EEPROMReadFailed => "EEPROMReadFailed".to_owned(),
            Self::EEPROMWriteFailed => "EEPROMWriteFailed".to_owned(),
            Self::EEPROMEraseFailed => "EEPROMEraseFailed".to_owned(),
            Self::EEPROMNotPresent => "EEPROMNotPresent".to_owned(),
            Self::EEPROMNotProgrammed => "EEPROMNotProgrammed".to_owned(),
            Self::InvalidArgs => "InvalidArgs".to_owned(),
            Self::NotSupported => "NotSupported".to_owned(),
            Self::NoMoreItems => "NoMoreItems".to_owned(),
            Self::Timeout => "Timeout".to_owned(),
            Self::OperationAborted => "OperationAborted".to_owned(),
            Self::ReservedPipe => "ReservedPipe".to_owned(),
            Self::InvalidControlRequestDirection => "InvalidControlRequestDirection".to_owned(),
            Self::InvalidControLRequestType => "InvalidControLRequestType".to_owned(),
            Self::IoPending => "IoPending".to_owned(),
            Self::IoIncomplete => "IoIncomplete".to_owned(),
            Self::HandleEof => "HandleEof".to_owned(),
            Self::Busy => "Busy".to_owned(),
            Self::NoSystemResources => "NoSystemResources".to_owned(),
            Self::DeviceListNotReady => "DeviceListNotReady".to_owned(),
            Self::DeviceNotConnected => "DeviceNotConnected".to_owned(),
            Self::IncorrectDevicePath => "IncorrectDevicePath".to_owned(),
            Self::OtherError => "OtherError".to_owned(),

            Self::LibraryAccessFailed(e) => format!("LibraryAccessFailed - {}", e),
            Self::UnpackingFailed(e) => format!("UnpackingFailed - {}", e),
            Self::LibraryAlreadyLoaded => "LibraryAlreadyLoaded".to_owned(),
            Self::LibraryNotLoaded => "LibraryNotLoaded".to_owned(),
        };
        let code = self
            .error_code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_owned());
        write!(f, "{} (error code {})", name, code)
    }
}
