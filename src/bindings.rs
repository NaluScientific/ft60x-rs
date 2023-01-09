pub const FT_DEVICE_DESCRIPTOR_TYPE: u16 = 0x01;
pub const FT_CONFIGURATION_DESCRIPTOR_TYPE: u16 = 0x02;
pub const FT_STRING_DESCRIPTOR_TYPE: u16 = 0x03;
pub const FT_INTERFACE_DESCRIPTOR_TYPE: u16 = 0x04;

// Reserved pipes
pub const FT_RESERVED_INTERFACE_INDEX: u16 = 0x0;
pub const FT_RESERVED_PIPE_INDEX_SESSION: u16 = 0x0;
pub const FT_RESERVED_PIPE_INDEX_NOTIFICATION: u16 = 0x1;
pub const FT_RESERVED_PIPE_SESSION: u16 = 0x1;
pub const FT_RESERVED_PIPE_NOTIFICATION: u16 = 0x81;

//
// Create flags
//
pub const FT_OPEN_BY_SERIAL_NUMBER: u32 = 0x00000001;
pub const FT_OPEN_BY_DESCRIPTION: u32 = 0x00000002;
pub const FT_OPEN_BY_LOCATION: u32 = 0x00000004;
pub const FT_OPEN_BY_GUID: u32 = 0x00000008;
pub const FT_OPEN_BY_INDEX: u32 = 0x00000010;

//
// ListDevices flags
//
pub const FT_LIST_ALL: u32 = 0x20000000;
pub const FT_LIST_BY_INDEX: u32 = 0x40000000;
pub const FT_LIST_NUMBER_ONLY: u32 = 0x80000000;

//
// GPIO direction, value
//
pub const FT_GPIO_DIRECTION_IN: u8 = 0;
pub const FT_GPIO_DIRECTION_OUT: u8 = 1;
pub const FT_GPIO_VALUE_LOW: u8 = 0;
pub const FT_GPIO_VALUE_HIGH: u8 = 1;
pub const FT_GPIO_0: u8 = 0;
pub const FT_GPIO_1: u8 = 1;


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum FtStatus {
    FT_OK = 0,
    FT_INVALID_HANDLE = 1,
    FT_DEVICE_NOT_FOUND = 2,
    FT_DEVICE_NOT_OPENED = 3,
    FT_IO_ERROR = 4,
    FT_INSUFFICIENT_RESOURCES = 5,
    FT_INVALID_PARAMETER = 6,
    FT_INVALID_BAUD_RATE = 7,
    FT_DEVICE_NOT_OPENED_FOR_ERASE = 8,
    FT_DEVICE_NOT_OPENED_FOR_WRITE = 9,
    FT_FAILED_TO_WRITE_DEVICE = 10,
    FT_EEPROM_READ_FAILED = 11,
    FT_EEPROM_WRITE_FAILED = 12,
    FT_EEPROM_ERASE_FAILED = 13,
    FT_EEPROM_NOT_PRESENT = 14,
    FT_EEPROM_NOT_PROGRAMMED = 15,
    FT_INVALID_ARGS = 16,
    FT_NOT_SUPPORTED = 17,

    FT_NO_MORE_ITEMS = 18,
    FT_TIMEOUT = 19,
    FT_OPERATION_ABORTED = 20,
    FT_RESERVED_PIPE = 21,
    FT_INVALID_CONTROL_REQUEST_DIRECTION = 22,
    FT_INVALID_CONTROL_REQUEST_TYPE = 23,
    FT_IO_PENDING = 24,
    FT_IO_INCOMPLETE = 25,
    FT_HANDLE_EOF = 26,
    FT_BUSY = 27,
    FT_NO_SYSTEM_RESOURCES = 28,
    FT_DEVICE_LIST_NOT_READY = 29,
    FT_DEVICE_NOT_CONNECTED = 30,
	FT_INCORRECT_DEVICE_PATH = 31,

    FT_OTHER_ERROR = 32,
}

pub enum FtPipeType {
    FTPipeTypeControl=0,
    FTPipeTypeIsochronous,
    FTPipeTypeBulk,
    FTPipeTypeInterrupt
}

#[repr(C)]
struct CommonDescriptor {
    bLength: u8,
    bDescriptorType: u8,
}

#[allow(non_snake_case)]
#[repr(C)]
struct DeviceDescriptor {
    bLength: u8,
    bDescriptorType: u8,
    bcdUSB: u16,
    bDeviceClass: u8,
    bDeviceSubClass: u8,
    bDeviceProtocol: u8,
    bMaxPacketSize0: u8,
    idVendor: u16,
    idProduct: u16,
    bcdDevice: u16,
    iManufacturer: u8,
    iProduct: u8,
    iSerialNumber: u8,
    bNumConfigurations: u8,
}

#[allow(non_snake_case)]
#[repr(C)]
struct ConfigurationDescriptor {
    bLength: u8,
    bDescriptorType: u8,
    wTotalLength: u16,
    bNumInterfaces: u8,
    bConfigurationValue: u8,
    iConfiguration: u8,
    bmAttributes: u8,
    MaxPower: u8,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct FT_DEVICE_LIST_INFO_NODE {
	pub Flags: ULONG,
	pub Type: ULONG,
	pub ID: ULONG,
	pub LocId: DWORD,
	pub SerialNumber: [u8; 16],
	pub Description: [u8; 32],
	pub ftHandle: FT_HANDLE,
}


pub type DWORD = libc::c_ulong;
pub type PVOID = *mut libc::c_void;
pub type FT_HANDLE = PVOID;
pub type PFT_HANDLE = *mut FT_HANDLE;
pub type UCHAR = u8;
pub type PUCHAR = *mut u8;
pub type ULONG = u32;
pub type PULONG = *mut u32;

#[link(name = "FTD3XX", kind="static")]
extern "C" {
    pub fn FT_ListDevices(pArg1: PVOID, pArg2: PVOID, flags: DWORD) -> DWORD;
    pub fn FT_CreateDeviceInfoList(lpdwNumDevs: *mut DWORD) -> DWORD;
    pub fn FT_GetDeviceInfoList(ptDest: *mut FT_DEVICE_LIST_INFO_NODE, lpdwNumDevs: *mut DWORD) -> DWORD;

    pub fn FT_Create(pvArg: PVOID, dwFlags: DWORD, pftHandle: PFT_HANDLE) -> DWORD;
    pub fn FT_Close(handle: FT_HANDLE) -> DWORD;
    pub fn FT_WritePipeEx(handle: FT_HANDLE, ucPipeId: u8, pucBuffer: *const u8, ulBufferLength: ULONG, pulBytesTransferred: PULONG, pOverlapped: PVOID) -> DWORD;
    pub fn FT_ReadPipeEx(handle: FT_HANDLE, ucPipeId: u8, pucBuffer: *mut u8, ulBufferLength: ULONG, pulBytesTransferred: PULONG, pOverlapped: PVOID) -> DWORD;
}

pub fn ft_success(status: FtStatus) -> bool {
    status == FtStatus::FT_OK
}

pub fn ft_failed(status: FtStatus) -> bool {
    status != FtStatus::FT_OK
}

pub fn is_read_pipe(pipe_id: u8) -> bool {
    (pipe_id & 0x80) == 1
}

pub fn is_write_pipe(pipe_id: u8) -> bool {
    (pipe_id & 0x80) == 0
}

pub fn is_bulk_pipe(pipe_type: u8) -> bool {
    pipe_type == 2
}
