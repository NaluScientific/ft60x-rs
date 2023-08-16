//! Provides low(er) level access to the D3XX dynamic library.
//!
//! This module provides a semi-safe wrapper around the D3XX dynamic library. The
//! functions and types defined in this module are not intended to be used directly
//! by the users of this crate. Instead, the functions and types defined in the
//! [top level](crate) should be used.
//!
//! D3xx functions can be found in the [`lib`](lib) module. The initial call to these
//! functions will attempt to load the D3xx dynamic library, and subsequent calls will
//! use the cached library. If the library cannot be loaded, the functions will return
//! [`D3xxError::LibraryLoadFailed`]. Types passed between the D3xx library are defined
//! in the [`types`](types) module, and constants are defined in the
//! [`constants`](constants) module.
//!
//! Note that all D3xx functions are marked unsafe, as they may cause undefined behavior
//! if used incorrectly. Care should be taken to ensure that the functions are used
//! according to the [Programmer's Guide](https://ftdichip.com/wp-content/uploads/2020/07/AN_379-D3xx-Programmers-Guide-1.pdf)

use libloading::Symbol;

use crate::assets::d3xx_lib;
use crate::Result;

/// Helper function for loading a D3xx function by name from the library.
///
/// # Errors
/// Returns [`D3xxError::LibraryNotLoaded`] if the library could not be loaded.
fn d3xx_fn<T>(name: &str) -> Result<Symbol<T>> {
    let library = d3xx_lib()?;
    let function = unsafe { library.get::<T>(name.as_bytes())? };
    Ok(function)
}

/// Helpful function for casting a mutable reference to a mutable pointer of a compatible type.
#[inline]
pub(crate) fn ptr_mut<T, U>(x: &mut T) -> *mut U {
    x as *mut _ as *mut U
}

/// Bindings to D3XX functions.
///
/// Prototypes for these functions are defined in the `FTD3XX.h` header file.
#[allow(non_snake_case, unused)]
pub(crate) mod lib {
    use libc::{c_uchar, c_ulong, c_ushort, c_void};
    use libloading::{Library, Symbol};
    use once_cell::sync::OnceCell;

    use super::d3xx_fn;
    use super::types::{
        FT_DEVICE_DESCRIPTOR, FT_DEVICE_LIST_INFO_NODE, FT_HANDLE, FT_PIPE_INFORMATION, FT_STATUS,
    };
    use crate::{D3xxError, Result};

    /// Macro for generating a wrapper function for a D3XX function.
    ///
    /// # Examples
    ///
    /// ```
    /// wrap_d3xx!(FT_ListDevices, pArg1: *mut c_void, pArg2: *mut c_void, flags: c_ulong);
    /// ```
    ///
    /// This will generate the following function:
    ///
    /// ```
    /// unsafe fn FT_ListDevices(pArg1: *mut c_void, pArg2: *mut c_void, flags: c_ulong) -> Result<()> {
    ///     type F = unsafe extern "C" fn(*mut c_void, *mut c_void, c_ulong) -> FT_STATUS;
    ///     static SYMBOL: OnceCell<Symbol<F>> = OnceCell::new();
    ///
    ///     let func = SYMBOL.get_or_try_init(|| d3xx_fn::<F>(stringify!($name)))?;
    ///     let res = unsafe { func(pArg1, pArg2, flags) };
    ///     if res != 0 {
    ///        return Err(D3xxError::from(res as u32).into());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    macro_rules! wrap_d3xx {
        ($name:ident, $($arg:ident: $ty:ty),*) => {
            pub(crate) unsafe fn $name($($arg: $ty),*) -> Result<()> {
                type F = unsafe extern "C" fn($($ty),*) -> FT_STATUS;
                static SYMBOL: OnceCell<Symbol<F>> = OnceCell::new();

                let func = SYMBOL.get_or_try_init(|| d3xx_fn::<F>(stringify!($name)))?;
                let res = unsafe { func($($arg),*) };
                if res != 0 {
                    return Err(D3xxError::from(res));
                }
                Ok(())
            }
        }
    }

    wrap_d3xx!(
        FT_ListDevices,
        pArg1: *mut c_void,
        pArg2: *mut c_void,
        flags: c_ulong
    );
    wrap_d3xx!(FT_CreateDeviceInfoList, lpdwNumDevs: *mut c_ulong);
    wrap_d3xx!(
        FT_GetDeviceInfoList,
        ptDest: *mut FT_DEVICE_LIST_INFO_NODE,
        lpdwNumDevs: *mut c_ulong
    );
    wrap_d3xx!(
        FT_GetDeviceInfoDetail,
        dwIndex: c_ulong,
        lpdwFlags: *mut c_ulong,
        lpdwType: *mut c_ulong,
        lpdwID: *mut c_ulong,
        lpdwLocId: *mut c_ulong,
        lpSerialNumber: *mut c_void,
        lpDescription: *mut c_void,
        pftHandle: *mut FT_HANDLE
    );
    wrap_d3xx!(
        FT_Create,
        pvArg: *mut c_void,
        dwFlags: c_ulong,
        pftHandle: *mut FT_HANDLE
    );
    wrap_d3xx!(FT_Close, ftHandle: FT_HANDLE);
    wrap_d3xx!(
        FT_GetDriverVersion,
        handle: FT_HANDLE,
        lpdwVersion: *mut c_ulong
    );
    wrap_d3xx!(
        FT_WritePipeEx,
        handle: FT_HANDLE,
        ucPipeId: u8,
        pucBuffer: *const c_uchar,
        ulBufferLength: c_ulong,
        pulBytesTransferred: *mut c_ulong,
        pOverlapped: *mut c_void
    );
    wrap_d3xx!(
        FT_ReadPipe,
        handle: FT_HANDLE,
        ucPipeId: u8,
        pucBuffer: *mut c_uchar,
        ulBufferLength: c_ulong,
        pulBytesTransferred: *mut c_ulong,
        pOverlapped: *mut c_void
    );
    wrap_d3xx!(
        FT_ReadPipeEx,
        handle: FT_HANDLE,
        ucPipeId: u8,
        pucBuffer: *mut c_uchar,
        ulBufferLength: c_ulong,
        pulBytesTransferred: *mut c_ulong,
        pOverlapped: *mut c_void
    );
    wrap_d3xx!(FT_FlushPipe, handle: FT_HANDLE, ucPipeID: c_uchar);
    wrap_d3xx!(
        FT_SetPipeTimeout,
        handle: FT_HANDLE,
        ucPipeID: c_uchar,
        ulTimeoutInMs: c_ulong
    );
    wrap_d3xx!(
        FT_GetPipeTimeout,
        handle: FT_HANDLE,
        ucPipeId: c_uchar,
        pTimeoutInMs: *mut c_ulong
    );
    wrap_d3xx!(
        FT_GetVIDPID,
        handle: FT_HANDLE,
        puwVID: *mut c_ushort,
        puwPID: *mut c_ushort
    );
    wrap_d3xx!(
        FT_GetDeviceDescriptor,
        handle: FT_HANDLE,
        pDescriptor: *mut FT_DEVICE_DESCRIPTOR
    );
    wrap_d3xx!(
        FT_SetStreamPipe,
        handle: FT_HANDLE,
        bAllWritePipes: c_uchar,
        bAllReadPipes: c_uchar,
        ucPipeID: c_uchar,
        ulStreamSize: c_ulong
    );
    wrap_d3xx!(
        FT_ClearStreamPipe,
        handle: FT_HANDLE,
        bAllWritePipes: c_uchar,
        bAllReadPipes: c_uchar,
        ucPipeID: c_uchar
    );
    wrap_d3xx!(FT_AbortPipe, handle: FT_HANDLE, ucPipeID: c_uchar);
    wrap_d3xx!(FT_CycleDevicePort, handle: FT_HANDLE);
    wrap_d3xx!(
        FT_GetPipeInformation,
        handle: FT_HANDLE,
        ucInterfaceIndex: c_uchar,
        ucPipeIndex: c_uchar,
        pPipeInformation: *mut FT_PIPE_INFORMATION
    );
    wrap_d3xx!(FT_GetLibraryVersion, version: *mut c_ulong);
}

/// Types used by D3XX functions.
///
/// These types are Rust-friendly versions of the C types defined in `FTD3XX.h`.
#[allow(non_camel_case_types)]
pub mod types {
    use libc::*;
    use std::{ffi::CStr, fmt::Debug};

    use crate::{error::D3xxError, Result};

    #[allow(non_snake_case)]
    #[repr(C)]
    #[derive(Debug, Default, Clone)]
    pub(crate) struct FT_DEVICE_DESCRIPTOR {
        pub(crate) bLength: c_uchar,
        pub(crate) bDescriptorType: c_uchar,
        pub(crate) bcdUSB: c_ushort,
        pub(crate) bDeviceClass: c_uchar,
        pub(crate) bDeviceSubClass: c_uchar,
        pub(crate) bDeviceProtocol: c_uchar,
        pub(crate) bMaxPacketSize0: c_uchar,
        pub(crate) idVendor: c_ushort,
        pub(crate) idProduct: c_ushort,
        pub(crate) bcdDevice: c_ushort,
        pub(crate) iManufacturer: c_uchar,
        pub(crate) iProduct: c_uchar,
        pub(crate) iSerialNumber: c_uchar,
        pub(crate) bNumConfigurations: c_uchar,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    #[derive(Clone)]
    pub(crate) struct FT_DEVICE_LIST_INFO_NODE {
        pub(crate) Flags: c_ulong,
        pub(crate) Type: c_ulong,
        pub(crate) ID: c_ulong,
        pub(crate) LocId: c_ulong,
        pub(crate) SerialNumber: [c_uchar; 16],
        pub(crate) Description: [c_uchar; 32],
        pub(crate) ftHandle: FT_HANDLE,
    }

    impl Debug for FT_DEVICE_LIST_INFO_NODE {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("FT_DEVICE_LIST_INFO_NODE")
                .field("Flags", &self.Flags)
                .field("Type", &self.Type)
                .field("ID", &self.ID)
                .field("LocId", &self.LocId)
                .field("SerialNumber", &c_str_to_string(&self.SerialNumber))
                .field("Description", &c_str_to_string(&self.Description))
                .field("ftHandle", &self.ftHandle)
                .finish()
        }
    }

    impl Default for FT_DEVICE_LIST_INFO_NODE {
        fn default() -> Self {
            Self {
                Flags: 0,
                Type: 0,
                ID: 0,
                LocId: 0,
                SerialNumber: Default::default(),
                Description: Default::default(),
                ftHandle: std::ptr::null_mut(),
            }
        }
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub(crate) struct FT_PIPE_INFORMATION {
        pub(crate) PipeType: c_int,
        pub(crate) PipeID: c_uchar,
        pub(crate) MaximumPacketSize: c_ushort,
        pub(crate) Interval: c_uchar,
    }

    #[allow(non_camel_case_types)]
    pub(crate) type FT_STATUS = c_ulong;
    #[allow(non_camel_case_types)]
    pub(crate) type FT_HANDLE = *mut c_void;

    /// Convert a C string received via FFI to a Rust `String`.
    pub(crate) fn c_str_to_string(s: &[c_uchar]) -> Result<String> {
        unsafe {
            Ok(CStr::from_ptr(s.as_ptr() as *const _)
                .to_str()
                .or(Err(D3xxError::OtherError))?
                .to_string())
        }
    }
}

/// Constants used by the D3XX library.
///
/// These are defined in `FTD3XX.h`.
#[allow(dead_code)]
pub mod constants {
    use libc::{c_uchar, c_ulong, c_ushort};

    // Standard Descriptor Types
    pub(crate) const FT_DEVICE_DESCRIPTOR_TYPE: c_ushort = 0x01;
    pub(crate) const FT_CONFIGURATION_DESCRIPTOR_TYPE: c_ushort = 0x02;
    pub(crate) const FT_STRING_DESCRIPTOR_TYPE: c_ushort = 0x03;
    pub(crate) const FT_INTERFACE_DESCRIPTOR_TYPE: c_ushort = 0x04;

    // Reserved pipes
    pub(crate) const FT_RESERVED_INTERFACE_INDEX: c_ushort = 0x0;
    pub(crate) const FT_RESERVED_PIPE_INDEX_SESSION: c_ushort = 0x0;
    pub(crate) const FT_RESERVED_PIPE_INDEX_NOTIFICATION: c_ushort = 0x1;
    pub(crate) const FT_RESERVED_PIPE_SESSION: c_ushort = 0x1;
    pub(crate) const FT_RESERVED_PIPE_NOTIFICATION: c_ushort = 0x81;

    // Create flags
    pub(crate) const FT_OPEN_BY_SERIAL_NUMBER: c_ulong = 0x00000001;
    pub(crate) const FT_OPEN_BY_DESCRIPTION: c_ulong = 0x00000002;
    pub(crate) const FT_OPEN_BY_LOCATION: c_ulong = 0x00000004;
    pub(crate) const FT_OPEN_BY_GUID: c_ulong = 0x00000008;
    pub(crate) const FT_OPEN_BY_INDEX: c_ulong = 0x00000010;

    // ListDevices flags
    pub(crate) const FT_LIST_ALL: c_ulong = 0x20000000;
    pub(crate) const FT_LIST_BY_INDEX: c_ulong = 0x40000000;
    pub(crate) const FT_LIST_NUMBER_ONLY: c_ulong = 0x80000000;

    // GPIO direction, value
    pub(crate) const FT_GPIO_DIRECTION_IN: c_uchar = 0;
    pub(crate) const FT_GPIO_DIRECTION_OUT: c_uchar = 1;
    pub(crate) const FT_GPIO_VALUE_LOW: c_uchar = 0;
    pub(crate) const FT_GPIO_VALUE_HIGH: c_uchar = 1;
    pub(crate) const FT_GPIO_0: c_uchar = 0;
    pub(crate) const FT_GPIO_1: c_uchar = 1;
}
