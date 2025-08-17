// https://usb.org/sites/default/files/hut1_21.pdf
// "10 Keyboard/Keypad Page (0x07)"

use core::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct KeyCode(pub(crate) u8);

impl fmt::Debug for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("KeyCode");
        match self.0 {
            0x04..=0x1D => d.field(&format_args!("{}", char::from(b'a' + (self.0 - 0x04)))),
            0x1E..=0x27 => d.field(&format_args!("{}", char::from(b'1' + (self.0 - 0x1E)))),
            0x28 => d.field(&format_args!("Return")),
            0x29 => d.field(&format_args!("ESCAPE")),
            0x2A => d.field(&format_args!("DELETE")),
            0x2B => d.field(&format_args!("Tab")),
            0x2C => d.field(&format_args!("Spacebar")),
            0x2D => d.field(&format_args!("-")),
            0x2E => d.field(&format_args!("=")),
            0x2F => d.field(&format_args!("[")),
            0x30 => d.field(&format_args!("]")),
            0x31 => d.field(&format_args!("\\")),
            0x33 => d.field(&format_args!(";")),
            0x34 => d.field(&format_args!("'")),
            0x35 => d.field(&format_args!("`")),
            0x36 => d.field(&format_args!(",")),
            0x37 => d.field(&format_args!(".")),
            0x38 => d.field(&format_args!("/")),
            _ => d.field(&format_args!("0x{:02X}", self.0)),
        };
        d.finish()
    }
}

impl KeyCode {
    pub const RETURN: Self = Self(0x28);
    pub const ESCAPE: Self = Self(0x29);
    pub const DELETE: Self = Self(0x2A);
    pub const TAB: Self = Self(0x2B);

    pub const fn from_ascii(value: u8) -> Option<(bool, Self)> {
        match value {
            b' ' => Some((false, Self(0x2C))),
            b'!' => Some((true, Self(0x1E))),
            b'"' => Some((true, Self(0x34))),
            b'#' => Some((true, Self(0x20))),
            b'$' => Some((true, Self(0x21))),
            b'%' => Some((true, Self(0x22))),
            b'&' => Some((true, Self(0x24))),
            b'\'' => Some((false, Self(0x34))),
            b'(' => Some((true, Self(0x26))),
            b')' => Some((true, Self(0x27))),
            b'*' => Some((true, Self(0x25))),
            b'+' => Some((true, Self(0x2E))),
            b',' => Some((false, Self(0x36))),
            b'-' => Some((false, Self(0x2D))),
            b'.' => Some((false, Self(0x37))),
            b'/' => Some((false, Self(0x38))),
            b'0' => Some((false, Self(0x27))),
            b'1'..=b'9' => Some((false, Self(0x1E + (value - b'1')))),
            b'=' => Some((false, Self(0x2E))),
            b'A'..=b'Z' => Some((true, Self(0x04 + (value - b'A')))),
            b'[' => Some((false, Self(0x2F))),
            b'\\' => Some((false, Self(0x31))),
            b']' => Some((false, Self(0x30))),
            b'^' => Some((true, Self(0x23))),
            b'_' => Some((true, Self(0x2D))),
            b'`' => Some((false, Self(0x35))),
            b'a'..=b'z' => Some((false, Self(0x04 + (value - b'a')))),
            b'{' => Some((true, Self(0x2F))),
            b'|' => Some((true, Self(0x31))),
            b'}' => Some((true, Self(0x30))),
            b'~' => Some((true, Self(0x35))),
            _ => None,
        }
    }
}
