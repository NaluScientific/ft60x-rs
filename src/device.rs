use std::{error::Error, time::Duration};

use rusb::{
    request_type, Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext,
};

use crate::{config::FT60xConfig, error::Ft60xError};

pub const DEFAULT_VID: u16 = 0x0403;
pub const DEFAULT_PID: u16 = 0x601F;

pub struct Ft60xDevice {
    context: Context,
    handle: DeviceHandle<Context>,
    streaming_mode: bool,
}

impl Ft60xDevice {
    pub fn open_default() -> Result<Ft60xDevice, Box<dyn Error>> {
        Self::open(DEFAULT_VID, DEFAULT_PID)
    }

    pub fn open(vid: u16, pid: u16) -> Result<Ft60xDevice, Box<dyn Error>> {
        let context = Context::new()?;
        let handle = context
            .open_device_with_vid_pid(vid, pid)
            .ok_or(Ft60xError::NoMatchingDevice)?;

        Ok(Ft60xDevice {
            context,
            handle,
            streaming_mode: false,
        })
    }

    pub fn config(&self) -> Result<FT60xConfig, Box<dyn Error>> {
        let mut buf = [0; 152];
        let read = self.handle.read_control(
            request_type(Direction::In, RequestType::Vendor, Recipient::Device),
            0xcf,
            1,
            0,
            &mut buf,
            Duration::new(1, 0),
        )?;

        if read != 152 {
            Err(Ft60xError::Unknown)?;
        }
        FT60xConfig::parse(buf)
    }

    pub fn set_config(&mut self, config: FT60xConfig) -> Result<(), Box<dyn Error>> {
        let buf = config.encode()?;
        let written = self.handle.write_control(
            request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
            0xcf,
            0,
            0,
            &buf,
            Duration::new(1, 0),
        )?;

        if written != 152 {
            Err(Ft60xError::Unknown)?;
        }
        Ok(())
    }

    pub fn set_streaming_mode(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.streaming_mode {
            self.handle.claim_interface(0)?;
            self.handle.claim_interface(1)?;

            let ctrlreq = [
                0x00, 0x00, 0x00, 0x00, 0x82, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            self.handle
                .write_bulk(0x01, &ctrlreq, Duration::new(1, 0))?;
        }
        Ok(())
    }

    /// it is recommended to read multiples of 32Kb
    pub fn read(&mut self, buf: &mut [u8]) -> Result<(), Box<dyn Error>> {
        // self.set_streaming_mode()?;

        let blocksize: usize = 32 * 1024; // 32 Kb seems to be the sweet spot for the ft601
        for chunk in buf.chunks_mut(blocksize) {
            let read_amount = self.handle.read_bulk(0x82, chunk, Duration::from_millis(1000))?;
            if read_amount != chunk.len() {
                Err(Ft60xError::ReadError)?;
            }
        }
        Ok(())
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Box<dyn Error>> {
        // self.set_streaming_mode()?;

        let blocksize: usize = 32 * 1024; // 32 Kb seems to be the sweet spot for the ft601
        for chunk in buf.chunks(blocksize) {
            let write_amount = self.handle.write_bulk(0x80, chunk, Duration::from_millis(1000))?;
            if write_amount != chunk.len() {
                Err(Ft60xError::WriteError)?;
            }
        }
        Ok(())
    }

}
