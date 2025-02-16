use serialport::{SerialPort, SerialPortType};
use std::io::{self, Read, Write};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ch9329(#[from] ch9329::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    SerialPort(#[from] serialport::Error),

    #[error("no device")]
    NoDevice,
}

pub struct Device<P> {
    port: P,
    buf: [u8; ch9329::MAX_PACKET_SIZE],
    addr: u8,
}

impl<P> Device<P>
where
    P: Read + Write,
{
    #[tracing::instrument(err, ret, skip(self))]
    pub fn clear(&mut self) -> Result<usize, Error> {
        let mut len = 0;
        loop {
            match self.port.read(&mut self.buf) {
                Ok(0) => break Ok(len),
                Ok(n) => len += n,
                Err(e) if e.kind() == io::ErrorKind::TimedOut => break Ok(len),
                Err(e) => Err(e)?,
            };
        }
    }

    #[tracing::instrument(err, ret, skip(self))]
    pub fn send(&mut self, command: ch9329::Command) -> Result<(), Error> {
        let packet = ch9329::encode(&mut self.buf, self.addr, command.cmd(), |buf| {
            command.data(buf)
        });
        tracing::info!(packet = format_args!("{packet:02X?}"));
        self.port.write_all(packet)?;
        self.port.flush()?;
        Ok(())
    }

    #[tracing::instrument(err, ret, skip(self))]
    pub fn recv(&mut self) -> Result<(u8, ch9329::Response<'_>), Error> {
        // https://github.com/tokio-rs/tracing/issues/2796
        let this = self;
        let mut len = 0;
        while let Err(ch9329::Error::Incomplete(total)) = ch9329::decode(&this.buf[..len]) {
            let n = this.port.read(&mut this.buf[len..total])?;
            if n == 0 {
                return Err(Error::Io(io::ErrorKind::UnexpectedEof.into()));
            }
            len += n;
        }
        let packet = &this.buf[..len];
        tracing::info!(packet = format_args!("{packet:02X?}"));
        let (addr, cmd, data) = ch9329::decode(packet)?;
        Ok((addr, ch9329::Response::decode(cmd, data)?))
    }
}

impl Device<Box<dyn SerialPort>> {
    pub fn open_usb(vid: u16, pid: u16) -> Result<Self, Error> {
        let port_info = serialport::available_ports()?
            .into_iter()
            .find(|port_info| {
                if let SerialPortType::UsbPort(port_info) = &port_info.port_type {
                    port_info.vid == vid && port_info.pid == pid
                } else {
                    false
                }
            })
            .ok_or(Error::NoDevice)?;
        let port = serialport::new(port_info.port_name, 9_600)
            .timeout(Duration::from_millis(500))
            .open()?;
        Ok(Self {
            port,
            buf: [0; ch9329::MAX_PACKET_SIZE],
            addr: 0x00,
        })
    }
}
