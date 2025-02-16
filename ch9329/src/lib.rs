#![no_std]

pub const MAX_PACKET_SIZE: usize = 5 + 64 + 1;

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("incomplete (expect {0} bytes)")]
    Incomplete(usize),
    #[error("invalid HEAD")]
    InvalidHead,
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
            [0x57, 0xAB, 0x00, 0x02, 0x08, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10],
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
