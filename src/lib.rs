use std::ffi::CStr;

use bindings::*;
use libc::c_void;

use error::{d3xx_error, d3xx_result, D3xxError};

pub mod bindings;
pub mod error;

type Result<T> = std::result::Result<T, D3xxError>;

pub struct Device {
    handle: FT_HANDLE,
}

impl Device {
    /// Open a device using the serial number
    pub fn open_with_serial_number(serial_number: &str) -> Result<Device> {
        unsafe {
            let serial = std::ffi::CString::new(serial_number).unwrap();
            let mut handle: FT_HANDLE = std::ptr::null_mut();
            d3xx_error!(FT_Create(
                serial.as_ptr() as *mut c_void,
                FT_OPEN_BY_SERIAL_NUMBER,
                &mut handle as PFT_HANDLE,
            ))?;
            Ok(Device { handle })
        }
    }

    /// Write to the device
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        let mut bytes_transferred = 0;
        unsafe {
            d3xx_error!(FT_WritePipeEx(
                self.handle,
                0x02,
                buf as *const _ as *const u8,
                buf.len() as u32,
                &mut bytes_transferred,
                std::ptr::null_mut(),
            ))?;
        }
        Ok(bytes_transferred as usize)
    }

    /// Read from the device
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut bytes_transferred = 0;
        unsafe {
            d3xx_error!(FT_ReadPipeEx(
                self.handle,
                0x82,
                buf as *mut _ as *mut u8,
                buf.len() as u32,
                &mut bytes_transferred,
                std::ptr::null_mut(),
            ));
        }
        Ok(bytes_transferred as usize)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            match FT_Close(self.handle) {
                0 => (),
                _ => panic!("Failed to close device"),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    flags: u32,
    type_: u32,
    id: u32,
    loc_id: u32,
    serial_number: String,
    description: String,
    handle: usize,
}

impl DeviceInfo {
    pub fn from_raw(node: &FT_DEVICE_LIST_INFO_NODE) -> DeviceInfo {
        unsafe {
            DeviceInfo {
                flags: node.Flags,
                type_: node.Type,
                id: node.ID,
                loc_id: node.LocId,
                serial_number: CStr::from_ptr(node.SerialNumber.as_ptr() as *const _)
                    .to_str()
                    .unwrap()
                    .to_string(),
                description: CStr::from_ptr(node.Description.as_ptr() as *const _)
                    .to_str()
                    .unwrap()
                    .to_string(),
                handle: node.ftHandle as usize,
            }
        }
    }

    pub fn open(&self) -> Result<Device> {
        Device::open_with_serial_number(&self.serial_number)
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn r#type(&self) -> u32 {
        self.type_
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn loc_id(&self) -> u32 {
        self.loc_id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    pub fn handle(&self) -> usize {
        self.handle
    }
}

pub fn list_device_details() -> Result<Vec<DeviceInfo>> {
    const MAX_DEVICES: usize = 32;
    let mut num_devices = 0;
    unsafe {
        let mut devices: [FT_DEVICE_LIST_INFO_NODE; MAX_DEVICES] = std::mem::zeroed();
        d3xx_error!(FT_CreateDeviceInfoList(
            &mut num_devices as *mut _ as *mut DWORD
        ))?;
        d3xx_error!(FT_GetDeviceInfoList(
            &mut devices as *mut FT_DEVICE_LIST_INFO_NODE,
            &mut num_devices as *mut _ as *mut DWORD
        ))?;
        Ok(devices[..num_devices]
            .iter()
            .map(DeviceInfo::from_raw)
            .collect())
    }
}

pub fn device_count() -> Result<u32> {
    let mut n: DWORD = 0;
    unsafe {
        d3xx_error!(FT_ListDevices(
            &mut n as *mut _ as PVOID,
            std::ptr::null_mut(),
            FT_LIST_NUMBER_ONLY,
        ))?;
    }
    Ok(n)
}
