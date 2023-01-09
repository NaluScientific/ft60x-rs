pub(crate) mod bindings;
pub mod error;

use std::{ffi::CString, fmt::Debug, ptr::null_mut, time::Duration};

use libc::*;

use bindings::*;
use error::{d3xx_error, D3xxError};

type Result<T> = std::result::Result<T, D3xxError>;

// =============================================================================

/// A D3XX device.
pub struct Device {
    handle: FT_HANDLE,
}

impl Device {
    /// Open a device using the given device information.
    pub fn open(info: &DeviceInfo) -> Result<Device> {
        Self::open_with_serial_number(&info.serial_number())
    }

    /// Open a device using the given serial number.
    pub fn open_with_serial_number(serial_number: &str) -> Result<Device> {
        unsafe {
            let serial = CString::new(serial_number).or(Err(D3xxError::InvalidParameter))?;
            let mut handle: FT_HANDLE = std::ptr::null_mut();
            d3xx_error!(FT_Create(
                serial.as_ptr() as *mut c_void,
                FT_OPEN_BY_SERIAL_NUMBER,
                &mut handle as *mut FT_HANDLE,
            ))?;
            Ok(Self::from_handle(handle))
        }
    }

    /// Create a device wrapper using a raw handle
    pub unsafe fn from_handle(handle: FT_HANDLE) -> Device {
        Self { handle }
    }

    /// Get the raw handle to the D3XX device.
    pub fn raw_handle(&self) -> FT_HANDLE {
        self.handle
    }

    /// Gets information about the device.
    pub fn info(&self) -> Result<DeviceInfo> {
        let index = self.index()?;
        let mut device_info = FT_DEVICE_LIST_INFO_NODE::default();
        unsafe {
            d3xx_error!(FT_GetDeviceInfoDetail(
                index as u32,
                ptr_mut(&mut device_info.Flags),
                ptr_mut(&mut device_info.Type),
                ptr_mut(&mut device_info.ID),
                ptr_mut(&mut device_info.LocId),
                ptr_mut(&mut device_info.SerialNumber),
                ptr_mut(&mut device_info.Description),
                ptr_mut(&mut device_info.ftHandle),
            ))?;
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
            d3xx_error!(FT_GetVIDPID(
                self.handle,
                ptr_mut(&mut vid),
                ptr_mut(&mut pid)
            ))?;
        }
        Ok((vid as usize, pid as usize))
    }

    /// Gets the D3XX kernel driver version.
    pub fn driver_version(&self) -> Result<Version> {
        let mut version: c_ulong = 0;
        unsafe {
            d3xx_error!(FT_GetDriverVersion(self.handle, ptr_mut(&mut version)))?;
        }
        Ok(Version::new(version as u32))
    }

    /// Get the index of this device in the current device info list.
    pub fn index(&self) -> Result<usize> {
        let devices = list_device()?;
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

    pub fn pipe_info(&self, pipe: Pipe) -> Result<PipeInfo> {
        let mut info = PipeInfo::default();
        unsafe {
            d3xx_error!(FT_GetPipeInformation(
                self.handle,
                1,
                pipe as c_uchar,
                ptr_mut(&mut info)
            ))?;
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
            match d3xx_error!(FT_WritePipeEx(
                self.handle,
                0x02,
                buf as *const _ as *const u8,
                buf.len() as u32,
                &mut bytes_transferred,
                std::ptr::null_mut(),
            )) {
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
            match d3xx_error!(FT_ReadPipeEx(
                self.handle,
                pipe as c_uchar,
                buf as *mut _ as *mut u8,
                buf.len() as u32,
                &mut bytes_transferred,
                std::ptr::null_mut(),
            )) {
                Ok(_) => (),
                Err(e) => {
                    self.abort_transfers(pipe)?;
                    return Err(e);
                }
            }
        }
        Ok(bytes_transferred as usize)
    }

    /// Configures a timeout for the specified endpoint. Reading and writing will
    /// timeout in the event the operation hangs for the given duration.
    ///
    /// The new value is only valid as long as the device is open; re-opening the device
    /// will reset the timeout to the default of 5 seconds.
    pub fn set_timeout(&self, pipe: Pipe, timeout: Duration) -> Result<()> {
        unsafe {
            d3xx_error!(FT_SetPipeTimeout(
                self.handle,
                pipe as c_uchar,
                timeout.as_millis() as c_ulong
            ))
        }
    }

    /// Get the timeout configured for the specified pipe.
    pub fn get_timeout(&self, pipe: Pipe) -> Result<Duration> {
        let mut timeout_millis: c_ulong = 0;
        unsafe {
            d3xx_error!(FT_GetPipeTimeout(
                self.handle,
                pipe as c_uchar,
                ptr_mut(&mut timeout_millis),
            ))?;
        }
        Ok(Duration::from_millis(timeout_millis as u64))
    }

    /// Sets streaming protocol transfer for the specified pipe. This is for
    /// applications that read or write a fixed size of data to or from the device.
    pub fn set_stream_size(&self, pipe: Pipe, stream_size: Option<u32>) -> Result<()> {
        unsafe {
            match stream_size {
                Some(size) => d3xx_error!(FT_SetStreamPipe(
                    self.handle,
                    false as c_uchar,
                    false as c_uchar,
                    pipe as c_uchar,
                    size as c_ulong,
                )),
                None => d3xx_error!(FT_ClearStreamPipe(
                    self.handle,
                    false as c_uchar,
                    false as c_uchar,
                    pipe as c_uchar
                )),
            }
        }
    }

    /// Aborts all pending transfers for the given pipe.
    pub fn abort_transfers(&self, pipe: Pipe) -> Result<()> {
        unsafe { d3xx_error!(FT_AbortPipe(self.handle, pipe as c_uchar)) }
    }

    /// Get the USB device descriptor.
    pub fn device_descriptor(&self) -> Result<DeviceDescriptor> {
        let mut device_descriptor = DeviceDescriptor::default();
        unsafe {
            d3xx_error!(FT_GetDeviceDescriptor(
                self.handle,
                ptr_mut(&mut device_descriptor.inner)
            ))?;
        }
        Ok(device_descriptor)
    }

    /// Power cycles the device port. This causes the device to be re-enumermated by the host.
    /// Consumes the object, meaning the device must be re-opened.
    pub fn power_cycle_port(self) -> Result<()> {
        // TODO: determine if device needs to be reopened.
        unsafe { d3xx_error!(FT_CycleDevicePort(self.handle)) }
    }
}

impl Drop for Device {
    /// Closes the device.
    ///
    /// # Panics
    /// Panics if the device could not be closed.
    fn drop(&mut self) {
        unsafe {
            match FT_Close(self.handle) {
                0 => (),
                _ => panic!("Failed to close device"),
            }
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
    inner: FT_DEVICE_LIST_INFO_NODE,
}

impl DeviceInfo {
    /// Create a new DeviceInfo object from a raw value. The index is the index in the D3XX
    /// device info list.
    fn new(index: usize, node: FT_DEVICE_LIST_INFO_NODE) -> DeviceInfo {
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
        self.inner.Flags
    }

    /// Device type.
    pub fn type_(&self) -> u32 {
        self.inner.Type
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
        self.inner.LocId
    }

    /// Device description.
    pub fn description(&self) -> String {
        c_str_to_string(&self.inner.Description)
    }

    /// Device serial number.
    pub fn serial_number(&self) -> String {
        c_str_to_string(&self.inner.SerialNumber)
    }

    /// Raw handle to the device.
    /// Returns `None` if the device is not opened, or `Some(handle)` if the device
    /// is currently open.
    pub fn raw_handle(&self) -> Option<FT_HANDLE> {
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
    inner: FT_DEVICE_DESCRIPTOR,
}

impl DeviceDescriptor {

    /// The USB specification number the device complies to.
    pub fn usb_specification_number(&self) -> usize {
        self.inner.bcdUSB as usize
    }

    /// The device class code assigned by the USB organization.
    pub fn class_code(&self) -> usize {
        self.inner.bDeviceClass as usize
    }

    /// The device subclass code assigned by the USB organization.
    pub fn subclass_code(&self) -> usize {
        self.inner.bDeviceSubClass as usize
    }

    /// The device protocol code assigned by the USB organization.
    pub fn protocol_code(&self) -> usize {
        self.inner.bDeviceProtocol as usize
    }

    /// The maximum packet size for Zero Endpoint
    pub fn max_packet_size(&self) -> usize {
        self.inner.bMaxPacketSize0 as usize
    }

    /// The device vendor ID assigned by the USB organization.
    pub fn vendor_id(&self) -> usize {
        self.inner.idVendor as usize
    }

    /// The device product ID assigned by the manufacturer.
    pub fn product_id(&self) -> usize {
        self.inner.idProduct as usize
    }

    /// The device release number.
    pub fn release_number(&self) -> usize {
        self.inner.bcdDevice as usize
    }

    /// The number of possible configurations for the device.
    pub fn num_configurations(&self) -> usize {
        self.inner.bNumConfigurations as usize
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
    inner: FT_PIPE_INFORMATION,
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
        d3xx_error!(FT_ListDevices(
            ptr_mut(&mut n),
            null_mut(),
            FT_LIST_NUMBER_ONLY,
        ))?;
    }
    Ok(n as u32)
}

/// Get information about all D3XX devices connected to the system.
pub fn list_device() -> Result<Vec<DeviceInfo>> {
    const MAX_DEVICES: usize = 32;
    let mut num_devices = 0;
    unsafe {
        let mut devices: [FT_DEVICE_LIST_INFO_NODE; MAX_DEVICES] = std::mem::zeroed();
        d3xx_error!(FT_CreateDeviceInfoList(ptr_mut(&mut num_devices)))?;
        d3xx_error!(FT_GetDeviceInfoList(
            ptr_mut(&mut devices),
            ptr_mut(&mut num_devices),
        ))?;
        Ok(devices[..num_devices]
            .iter()
            .enumerate()
            .map(|(i, e)| DeviceInfo::new(i, e.clone()))
            .collect())
    }
}

/// Get the D3XX library version.
pub fn d3xx_version() -> Version {
    let mut version: c_ulong = 0;
    unsafe {
        d3xx_error!(FT_GetLibraryVersion(ptr_mut(&mut version)))
            .expect("failed to get d3xx library version");
    }
    Version::new(version as u32)
}

/// Check if D3XX drivers are available on this system.
pub fn d3xx_available() -> bool {
    device_count().is_ok()
}