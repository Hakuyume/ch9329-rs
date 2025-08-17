use clap::Parser;
use std::io::{self, Read};

#[derive(Parser)]
struct Args {
    #[clap(long = "return")]
    return_: bool,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // CH340 (MR-CH9329EMU-USB)
    let mut device = ch9329_serialport::Device::open_usb(0x1a86, 0x7523)?;
    device.clear()?;

    let keys = io::stdin()
        .lock()
        .bytes()
        .filter_map(|b| match b {
            Ok(b) => {
                if let Some((shift, code)) = ch9329::KeyCode::from_ascii(b) {
                    Some(Ok((shift, code)))
                } else {
                    tracing::warn!("skip '{}'", b.escape_ascii());
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .chain(args.return_.then_some(Ok((false, ch9329::KeyCode::RETURN))));

    for key in keys {
        let (shift, code) = key?;

        device.send(ch9329::Command::SendKbGeneralData {
            modifiers: if shift {
                ch9329::KeyModifiers::LEFT_SHIFT
            } else {
                ch9329::KeyModifiers::empty()
            },
            codes: &[code],
        })?;
        device.recv()?;

        device.send(ch9329::Command::SendKbGeneralData {
            modifiers: ch9329::KeyModifiers::empty(),
            codes: &[],
        })?;
        device.recv()?;
    }

    Ok(())
}
