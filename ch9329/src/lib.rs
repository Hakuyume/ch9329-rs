#![no_std]

use core::str::Utf8Error;

pub const MAX_PACKET_SIZE: usize = 5 + 64 + 1;

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error("incomplete (expect {0} bytes)")]
    Incomplete(usize),
    #[error("invalid HEAD")]
    InvalidHead,
    #[error("invalid CMD")]
    InvalidCmd,
    #[error("invalid DATA")]
    InvalidData,
    #[error("invalid SUM")]
    InvalidSum,
}

pub fn encode<F>(buf: &mut [u8], addr: u8, cmd: u8, data: F) -> &[u8]
where
    F: FnOnce(&mut [u8]) -> usize,
{
    buf[..2].copy_from_slice(&HEAD);
    buf[2] = addr;
    buf[3] = cmd;
    let len = data(&mut buf[5..]);
    buf[4] = len.try_into().unwrap();
    buf[5 + len] = sum(&buf[..5 + len]);
    &buf[..5 + len + 1]
}

pub fn decode(buf: &[u8]) -> Result<(u8, u8, &[u8]), Error> {
    if buf.len() < 5 + 1 {
        return Err(Error::Incomplete(5 + 1));
    }
    if buf[..2] != HEAD {
        return Err(Error::InvalidHead);
    }
    let addr = buf[2];
    let cmd = buf[3];
    let len = usize::from(buf[4]);
    if buf.len() < 5 + len + 1 {
        return Err(Error::Incomplete(5 + len + 1));
    }
    let data = &buf[5..5 + len];
    if buf[5 + len] != sum(&buf[..5 + len]) {
        return Err(Error::InvalidSum);
    }

    Ok((addr, cmd, data))
}

const HEAD: [u8; 2] = [0x57, 0xAB];

fn sum(buf: &[u8]) -> u8 {
    buf.iter().fold(0, |a, b| a.overflowing_add(*b).0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command<'a> {
    GetInfo,
    SendMyHidData { data: &'a [u8] },
    GetParaCfg,
    GetUsbString { type_: UsbStringType },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Response<'a> {
    GetInfo {
        version: char,
    },
    GetParaCfg(ParaCfg),
    GetUsbString {
        type_: UsbStringType,
        descriptor: &'a str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParaCfg {
    pub operation_mode: u8,
    pub serial_communication_mode: u8,
    pub addr: u8,
    pub baud_rate: u32,
    todo_0: [u8; 2 + 2],
    pub vid: u16,
    pub pid: u16,
    todo_1: [u8; 2 + 2 + 1 + 8 + 8 + 1 + 1 + 12],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbStringType {
    Vendor,
    Product,
    Serial,
}

impl Command<'_> {
    pub fn cmd(self) -> u8 {
        match self {
            Self::GetInfo => 0x01,
            Self::SendMyHidData { .. } => 0x06,
            Self::GetParaCfg => 0x08,
            Self::GetUsbString { .. } => 0x0A,
        }
    }

    pub fn data(self, buf: &mut [u8]) -> usize {
        match self {
            Self::GetInfo | Self::GetParaCfg => 0,
            Self::SendMyHidData { data } => {
                buf[..data.len()].copy_from_slice(data);
                data.len()
            }
            Self::GetUsbString { type_ } => {
                buf[0] = match type_ {
                    UsbStringType::Vendor => 0x00,
                    UsbStringType::Product => 0x01,
                    UsbStringType::Serial => 0x02,
                };
                1
            }
        }
    }
}

impl<'a> Response<'a> {
    pub fn decode(cmd: u8, data: &'a [u8]) -> Result<Self, Error> {
        match cmd {
            0x81 => {
                if data.len() == 8 {
                    let version = data[0].into();
                    Ok(Self::GetInfo { version })
                } else {
                    Err(Error::InvalidData)
                }
            }
            0x88 => {
                if data.len() == 50 {
                    let operation_mode = data[0];
                    let serial_communication_mode = data[1];
                    let addr = data[2];
                    let baud_rate = u32::from_be_bytes(data[3..7].try_into().unwrap());
                    let todo_0 = data[7..11].try_into().unwrap();
                    let vid = u16::from_be_bytes(data[11..13].try_into().unwrap());
                    let pid = u16::from_be_bytes(data[13..15].try_into().unwrap());
                    let todo_1 = data[15..50].try_into().unwrap();
                    Ok(Self::GetParaCfg(ParaCfg {
                        operation_mode,
                        serial_communication_mode,
                        addr,
                        baud_rate,
                        todo_0,
                        vid,
                        pid,
                        todo_1,
                    }))
                } else {
                    Err(Error::InvalidData)
                }
            }
            0x8A => {
                if data.len() >= 2 {
                    let type_ = match data[0] {
                        0x00 => Ok(UsbStringType::Vendor),
                        0x01 => Ok(UsbStringType::Product),
                        0x02 => Ok(UsbStringType::Serial),
                        _ => Err(Error::InvalidData),
                    }?;
                    let len = usize::from(data[1]);
                    if data.len() == 2 + len {
                        let descriptor = core::str::from_utf8(&data[2..2 + len])?;
                        Ok(Self::GetUsbString { type_, descriptor })
                    } else {
                        Err(Error::InvalidData)
                    }
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidCmd),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_encode() {
        let mut buf = [0; crate::MAX_PACKET_SIZE];

        let data = [0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00];
        let packet = super::encode(&mut buf, 0x00, 0x02, |buf| {
            buf[..data.len()].copy_from_slice(&data);
            data.len()
        });
        assert_eq!(
            packet,
            [
                0x57, 0xAB, 0x00, 0x02, 0x08, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10
            ],
        );
    }

    #[test]
    fn test_decode() {
        let buf = [
            0x57, 0xAB, 0x00, 0x02, 0x08, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
        ];
        assert_eq!(super::decode(&buf[..3]), Err(super::Error::Incomplete(6)));
        assert_eq!(super::decode(&buf[..6]), Err(super::Error::Incomplete(14)));
        assert_eq!(
            super::decode(&buf),
            Ok((
                0x00,
                0x02,
                &[0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00][..]
            ))
        );
    }
}
