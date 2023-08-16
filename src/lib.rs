//! Crate for interfacing with FT600/601 devices through a semi-safe wrapper around the D3XX library.
//!
//! The center point of this crate is the [`Device`] struct. This struct
//! represents a single FT600/601 device. It provides methods for
//! interacting with the device, such as configuring and reading/writing to
//! the device.
//!
//! # ⚠️ Important ⚠️
//!
//! The D3XX library must be loaded before any D3XX functions can be called.
//! This can be done by calling [`load_dylib`] to load an external library from disk
//! or [`load_bundled_dylib`] to use the bundled library for the current platform.

pub(crate) mod assets;
pub mod error;
pub(crate) mod ffi;

use std::{ffi::CString, fmt::Debug, ptr::null_mut, time::Duration};

use ffi::{constants, lib, ptr_mut, types};
use libc::{c_uchar, c_ulong, c_ushort, c_void};

pub use error::D3xxError;
pub use assets::{load_dylib, load_bundled_dylib};

pub type Result<T, E = D3xxError> = std::result::Result<T, E>;

/// A D3XX device.
///
/// The [`Device`] struct represents a single D3XX device. It provides methods
/// for interacting with the device, such as configuring and reading/writing to
/// the device.
pub struct Device {
    /// The raw handle to the D3XX device.
    handle: types::FT_HANDLE,
}

impl Device {
    /// Open a device using the given device information.
    pub fn open(info: &DeviceInfo) -> Result<Device> {
        Self::open_with_serial_number(&info.serial_number()?)
    }

    /// Open a device using the given serial number.
    pub fn open_with_serial_number(serial_number: &str) -> Result<Device> {
        unsafe {
            let serial = CString::new(serial_number).or(Err(D3xxError::InvalidParameter))?;
            let mut handle: types::FT_HANDLE = std::ptr::null_mut();
            lib::FT_Create(
                serial.as_ptr() as *mut c_void,
                constants::FT_OPEN_BY_SERIAL_NUMBER,
                &mut handle as *mut types::FT_HANDLE,
            )?;
            Ok(Self::from_handle(handle))
        }
    }

    /// Create a device wrapper using a raw handle
    pub unsafe fn from_handle(handle: types::FT_HANDLE) -> Device {
        Self { handle }
    }

    /// Get the raw handle to the D3XX device.
    pub fn raw_handle(&self) -> types::FT_HANDLE {
        self.handle
    }

    /// Gets information about the device.
    pub fn info(&self) -> Result<DeviceInfo> {
        let index = self.index()?;
        let mut device_info = types::FT_DEVICE_LIST_INFO_NODE::default();
        unsafe {
            lib::FT_GetDeviceInfoDetail(
                index as c_ulong,
                ptr_mut(&mut device_info.Flags),
                ptr_mut(&mut device_info.Type),
                ptr_mut(&mut device_info.ID),
                ptr_mut(&mut device_info.LocId),
                ptr_mut(&mut device_info.SerialNumber),
                ptr_mut(&mut device_info.Description),
                ptr_mut(&mut device_info.ftHandle),
            )?;
        }
        Ok(DeviceInfo::new(index, device_info))
    }

    /// Get the vendor ID of the device.
    pub fn vendor_id(&self) -> Result<usize> {
        Ok(self.vid_pid()?.0)
    }

    /// Get the product ID of the device.
    pub fn product_id(&self) -> Result<usize> {
        Ok(self.vid_pid()?.1)
    }

    /// Get the vendor ID and product ID of the device.
    fn vid_pid(&self) -> Result<(usize, usize)> {
        let mut vid: c_ushort = 0;
        let mut pid: c_ushort = 0;
        unsafe {
            lib::FT_GetVIDPID(self.handle, ptr_mut(&mut vid), ptr_mut(&mut pid))?;
        }
        Ok((vid as usize, pid as usize))
    }

    /// Check if the device is connected using a USB2 cable.
    pub fn is_usb2(&self) -> Result<bool> {
        Ok(self.device_descriptor()?.is_usb2())
    }

    /// Check if the device is connected using a USB3 cable.
    pub fn is_usb3(&self) -> Result<bool> {
        Ok(self.device_descriptor()?.is_usb3())
    }

    /// Gets the D3XX kernel driver version.
    pub fn driver_version(&self) -> Result<Version> {
        let mut version: c_ulong = 0;
        unsafe {
            lib::FT_GetDriverVersion(self.handle, ptr_mut(&mut version))?;
        }
        Ok(Version::new(version as u32))
    }

    /// Get the index of this device in the current device info list.
    pub fn index(&self) -> Result<usize> {
        let devices = list_devices()?;
        let (i, _) = devices
            .iter()
            .enumerate()
            .find(|(_, x)| match x.raw_handle() {
                Some(handle) => handle == self.handle,
                None => false,
            })
            .ok_or(D3xxError::DeviceNotFound)?;
        Ok(i)
    }

    /// Get information about a pipe.
    #[allow(unused, unreachable_code)]
    pub fn pipe_info(&self, pipe: Pipe) -> Result<PipeInfo> {
        todo!("this doesn't work, interface index is wrong");

        let mut info = PipeInfo::default();
        unsafe {
            lib::FT_GetPipeInformation(self.handle, 0, pipe as c_uchar, ptr_mut(&mut info))?;
        }
        Ok(info)
    }

    /// Writes data to the specified pipe. This method will block
    /// until the transfer is complete, or the timeout is reached.
    pub fn write(&self, pipe: Pipe, buf: &[u8]) -> Result<usize> {
        if !pipe.is_write_pipe() {
            Err(D3xxError::InvalidParameter)?;
        }

        let mut bytes_transferred = 0;
        unsafe {
            match lib::FT_WritePipeEx(
                self.handle,
                0x02,
                buf as *const _ as *const u8,
                buf.len() as c_ulong,
                &mut bytes_transferred,
                std::ptr::null_mut(),
            ) {
                Ok(_) => (),
                Err(e) => {
                    self.abort_transfers(pipe)?;
                    return Err(e);
                }
            }
        }
        Ok(bytes_transferred as usize)
    }

    /// Reads data from the specified pipe. This method will block
    /// until the transfer is complete, or the timeout is reached.
    pub fn read(&self, pipe: Pipe, buf: &mut [u8]) -> Result<usize> {
        if !pipe.is_read_pipe() {
            Err(D3xxError::InvalidParameter)?;
        }

        let mut bytes_transferred = 0;
        unsafe {
            match lib::FT_ReadPipe(
                self.handle,
                pipe as c_uchar,
                buf as *mut _ as *mut u8,
                buf.len() as c_ulong,
                &mut bytes_transferred,
                std::ptr::null_mut(),
            ) {
                Ok(_) => (),
                Err(e) => {
                    self.abort_transfers(pipe)?;
                    return Err(e);
                }
            }
        }
        Ok(bytes_transferred as usize)
    }

    /// Discards any data cached in an IN pipe.
    /// If `pipe` is an OUT pipe, an `InvalidParameter` error is returned.
    pub fn flush(&self, pipe: Pipe) -> Result<()> {
        if !pipe.is_read_pipe() {
            Err(D3xxError::InvalidParameter)?;
        }
        unsafe { lib::FT_FlushPipe(self.handle, pipe as c_uchar) }
    }

    /// Configures a timeout for the specified endpoint. Reading and writing will
    /// timeout in the event the operation hangs for the given duration.
    ///
    /// The new value is only valid as long as the device is open; re-opening the device
    /// will reset the timeout to the default of 5 seconds.
    pub fn set_timeout(&self, pipe: Pipe, timeout: Duration) -> Result<()> {
        unsafe {
            lib::FT_SetPipeTimeout(self.handle, pipe as c_uchar, timeout.as_millis() as c_ulong)
        }
    }

    /// Get the timeout configured for the specified pipe.
    pub fn get_timeout(&self, pipe: Pipe) -> Result<Duration> {
        let mut timeout_millis: c_ulong = 0;
        unsafe {
            lib::FT_GetPipeTimeout(self.handle, pipe as c_uchar, ptr_mut(&mut timeout_millis))?;
        }
        Ok(Duration::from_millis(timeout_millis as u64))
    }

    /// Sets streaming protocol transfer for the specified pipe. This is for
    /// applications that read or write a fixed size of data to or from the device.
    pub fn set_stream_size(&self, pipe: Pipe, stream_size: Option<u32>) -> Result<()> {
        unsafe {
            match stream_size {
                Some(size) => lib::FT_SetStreamPipe(
                    self.handle,
                    false as c_uchar,
                    false as c_uchar,
                    pipe as c_uchar,
                    size as c_ulong,
                ),
                None => lib::FT_ClearStreamPipe(
                    self.handle,
                    false as c_uchar,
                    false as c_uchar,
                    pipe as c_uchar,
                ),
            }
        }
    }

    /// Aborts all pending transfers for the given pipe.
    pub fn abort_transfers(&self, pipe: Pipe) -> Result<()> {
        unsafe { lib::FT_AbortPipe(self.handle, pipe as c_uchar) }
    }

    /// Get the USB device descriptor.
    pub fn device_descriptor(&self) -> Result<DeviceDescriptor> {
        let mut device_descriptor = DeviceDescriptor::default();
        unsafe {
            lib::FT_GetDeviceDescriptor(self.handle, ptr_mut(&mut device_descriptor.inner))?;
        }
        Ok(device_descriptor)
    }

    /// Power cycles the device port. This causes the device to be re-enumermated by the host.
    /// Consumes the object, meaning the device must be re-opened.
    pub fn power_cycle_port(self) -> Result<()> {
        // TODO: determine if device needs to be reopened.
        unsafe { lib::FT_CycleDevicePort(self.handle) }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            let _ = lib::FT_Close(self.handle);
        }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle)
            .finish()
    }
}

// =============================================================================
/// Holds device information regarding a D3XX device attached to the system.
#[derive(Clone, Debug, Default)]
pub struct DeviceInfo {
    /// Index in the D3XX device list. This value changes when the list is rebuilt!
    index: usize,
    inner: types::FT_DEVICE_LIST_INFO_NODE,
}

impl DeviceInfo {
    /// Create a new DeviceInfo object from a raw value. The index is the index in the D3XX
    /// device info list.
    fn new(index: usize, node: types::FT_DEVICE_LIST_INFO_NODE) -> DeviceInfo {
        DeviceInfo { index, inner: node }
    }

    /// Attempts to open the device represented by this struct.
    pub fn open(&self) -> Result<Device> {
        Device::open(&self)
    }

    /// Gets the index of this device in the current D3XX device list.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Bit flags for USB3 or USB2 connection, etc.
    pub fn flags(&self) -> u32 {
        self.inner.Flags as _
    }

    /// Device type.
    pub fn type_(&self) -> u32 {
        self.inner.Type as _
    }

    /// Vendor ID.
    pub fn vendor_id(&self) -> u16 {
        ((self.inner.ID >> 16) & 0xFFFF) as _
    }

    /// Product ID.
    pub fn product_id(&self) -> u16 {
        (self.inner.ID & 0xFFFF) as _
    }

    /// Location identifier.
    pub fn location_identifier(&self) -> u32 {
        self.inner.LocId as _
    }

    /// Device description.
    pub fn description(&self) -> Result<String> {
        types::c_str_to_string(&self.inner.Description)
    }

    /// Device serial number.
    pub fn serial_number(&self) -> Result<String> {
        types::c_str_to_string(&self.inner.SerialNumber)
    }

    /// Raw handle to the device.
    /// Returns `None` if the device is not opened, or `Some(handle)` if the device
    /// is currently open.
    pub fn raw_handle(&self) -> Option<types::FT_HANDLE> {
        if self.inner.ftHandle as usize == 0 {
            None
        } else {
            Some(self.inner.ftHandle)
        }
    }

    /// Checks if the device is currently in use.
    pub fn is_open(&self) -> bool {
        self.raw_handle().is_some()
    }
}

// =============================================================================

/// Holds information regarding a USB device.
#[derive(Default, Clone)]
pub struct DeviceDescriptor {
    inner: types::FT_DEVICE_DESCRIPTOR,
}

impl DeviceDescriptor {
    /// The USB specification number the device complies to.
    pub fn usb_specification_number(&self) -> usize {
        self.inner.bcdUSB as _
    }

    /// Check if the device is connected using a USB2 cable.
    pub fn is_usb2(&self) -> bool {
        (self.usb_specification_number() >> 8) & 0xFF == 2
    }

    /// Check if the device is connected using a USB3 cable.
    pub fn is_usb3(&self) -> bool {
        (self.usb_specification_number() >> 8) & 0xFF == 3
    }

    /// The device class code assigned by the USB organization.
    pub fn class_code(&self) -> usize {
        self.inner.bDeviceClass as _
    }

    /// The device subclass code assigned by the USB organization.
    pub fn subclass_code(&self) -> usize {
        self.inner.bDeviceSubClass as _
    }

    /// The device protocol code assigned by the USB organization.
    pub fn protocol_code(&self) -> usize {
        self.inner.bDeviceProtocol as _
    }

    /// The maximum packet size for Zero Endpoint
    pub fn max_packet_size(&self) -> usize {
        self.inner.bMaxPacketSize0 as _
    }

    /// The device vendor ID assigned by the USB organization.
    pub fn vendor_id(&self) -> usize {
        self.inner.idVendor as _
    }

    /// The device product ID assigned by the manufacturer.
    pub fn product_id(&self) -> usize {
        self.inner.idProduct as _
    }

    /// The device release number.
    pub fn release_number(&self) -> usize {
        self.inner.bcdDevice as _
    }

    /// The number of possible configurations for the device.
    pub fn num_configurations(&self) -> usize {
        self.inner.bNumConfigurations as _
    }
}

impl Debug for DeviceDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

// =============================================================================
/// Represents a pipe used for communication with a D3XX device.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pipe {
    /// Input pipe 0 (0x82).
    In0 = 0x82,
    /// Input pipe 1 (0x83).
    In1 = 0x83,
    /// Input pipe 2 (0x84).
    In2 = 0x84,
    /// Input pipe 3 (0x85).
    In3 = 0x85,
    /// Output pipe 0 (0x02).
    Out0 = 0x02,
    /// Output pipe 1 (0x03).
    Out1 = 0x03,
    /// Output pipe 2 (0x04).
    Out2 = 0x04,
    /// Output pipe 3 (0x05).
    Out3 = 0x05,
}

impl Pipe {
    /// Check if the pipe is a read pipe.
    pub fn is_read_pipe(&self) -> bool {
        match self {
            Pipe::In0 | Pipe::In1 | Pipe::In2 | Pipe::In3 => true,
            Pipe::Out0 | Pipe::Out1 | Pipe::Out2 | Pipe::Out3 => false,
        }
    }

    /// Check if the pipe is a write pipe.
    pub fn is_write_pipe(&self) -> bool {
        match self {
            Pipe::In0 | Pipe::In1 | Pipe::In2 | Pipe::In3 => false,
            Pipe::Out0 | Pipe::Out1 | Pipe::Out2 | Pipe::Out3 => true,
        }
    }
}

impl Debug for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = *self as u8;
        match self {
            Self::In0 => write!(f, "In0 ({})", value),
            Self::In1 => write!(f, "In1 ({})", value),
            Self::In2 => write!(f, "In2 ({})", value),
            Self::In3 => write!(f, "In3 ({})", value),
            Self::Out0 => write!(f, "Out0 ({})", value),
            Self::Out1 => write!(f, "Out1 ({})", value),
            Self::Out2 => write!(f, "Out2 ({})", value),
            Self::Out3 => write!(f, "Out3 ({})", value),
        }
    }
}

impl From<u8> for Pipe {
    /// Convert from a raw pipe ID to a `Pipe` enum.
    ///
    /// # Panics
    /// Panics if the given value is an invalid pipe ID.
    fn from(pipe_id: u8) -> Self {
        match pipe_id {
            0x82 => Pipe::In0,
            0x83 => Pipe::In1,
            0x84 => Pipe::In2,
            0x85 => Pipe::In3,
            0x02 => Pipe::Out0,
            0x03 => Pipe::Out1,
            0x04 => Pipe::Out2,
            0x05 => Pipe::Out3,
            _ => panic!("Unknown value: {}", pipe_id),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum PipeType {
    /// USB control pipe
    Control = 0,
    /// USB isochronous pipe
    Isochronous = 1,
    /// USB bulk pipe
    Bulk = 2,
    /// USB interrupt pipe
    Interrupt = 3,
}

impl From<u8> for PipeType {
    /// Convert from a raw pipe type to a `PipeType` enum.
    ///
    /// # Panics
    /// Panics if the given value is an invalid pipe type.
    fn from(pipe_id: u8) -> Self {
        match pipe_id {
            0 => PipeType::Control,
            1 => PipeType::Isochronous,
            2 => PipeType::Bulk,
            3 => PipeType::Interrupt,
            _ => panic!("Unknown value: {}", pipe_id),
        }
    }
}

/// Stores information about a pipe.
#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub struct PipeInfo {
    inner: types::FT_PIPE_INFORMATION,
}

impl PipeInfo {
    /// Get the type of pipe.
    pub fn type_(&self) -> PipeType {
        PipeType::from(self.inner.PipeType as u8)
    }

    /// Get the pipe.
    pub fn pipe(&self) -> Pipe {
        Pipe::from(self.inner.PipeID as u8)
    }

    /// Get the maximum transfer size for this pipe.
    pub fn maximum_packet_size(&self) -> usize {
        self.inner.MaximumPacketSize as _
    }

    /// Get the polling interval. Used for interrupt pipes only.
    pub fn interval(&self) -> u8 {
        self.inner.Interval as _
    }
}

impl Debug for PipeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

// =============================================================================

/// Represents a D3XX driver or library version number.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u8,
    minor: u8,
    svn: u8,
    build: u8,
}

impl Version {
    /// Create a new version from a raw version number
    pub fn new(version: u32) -> Version {
        Self {
            major: ((version >> 24) & 0xFF) as u8,
            minor: ((version >> 16) & 0xFF) as u8,
            svn: ((version >> 8) & 0xFF) as u8,
            build: (version & 0xFF) as u8,
        }
    }

    /// Major version number.
    pub fn major(&self) -> u8 {
        self.major
    }

    /// Minor version number.
    pub fn minor(&self) -> u8 {
        self.minor
    }

    /// Subversion number.
    pub fn svn(&self) -> u8 {
        self.svn
    }

    /// Build number.
    pub fn build(&self) -> u8 {
        self.build
    }
}

// =============================================================================
/// Get the number of D3XX devices connected to the system.
pub fn device_count() -> Result<u32> {
    let mut n: c_ulong = 0;
    unsafe {
        lib::FT_ListDevices(ptr_mut(&mut n), null_mut(), constants::FT_LIST_NUMBER_ONLY)?;
    }
    Ok(n as u32)
}

/// Get information about all D3XX devices connected to the system.
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    const MAX_DEVICES: usize = 32;
    let mut num_devices = 0;
    let devices = unsafe {
        let mut devices: [types::FT_DEVICE_LIST_INFO_NODE; MAX_DEVICES] = std::mem::zeroed();
        lib::FT_CreateDeviceInfoList(ptr_mut(&mut num_devices))?;
        lib::FT_GetDeviceInfoList(ptr_mut(&mut devices), ptr_mut(&mut num_devices))?;
        devices
    };
    Ok(devices[..num_devices]
        .iter()
        .enumerate()
        .map(|(i, e)| DeviceInfo::new(i, e.clone()))
        .collect())
}

/// Get the D3XX library version.
pub fn d3xx_version() -> Version {
    let mut version: c_ulong = 0;
    unsafe {
        lib::FT_GetLibraryVersion(ptr_mut(&mut version))
            .expect("failed to get d3xx library version");
    }
    Version::new(version as u32)
}

/// Check if D3XX drivers are available on this system.
pub fn d3xx_available() -> bool {
    device_count().is_ok()
}

unsafe impl Send for Device {}
