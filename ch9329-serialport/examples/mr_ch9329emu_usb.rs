fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut device = ch9329_serialport::Device::open_usb(0x1a86, 0x7523)?;
    device.clear()?;

    device.send(ch9329::Command::GetInfo)?;
    device.recv()?;

    device.send(ch9329::Command::GetUsbString {
        type_: ch9329::UsbStringType::Vendor,
    })?;
    device.recv()?;

    device.send(ch9329::Command::GetUsbString {
        type_: ch9329::UsbStringType::Product,
    })?;
    device.recv()?;

    device.send(ch9329::Command::GetUsbString {
        type_: ch9329::UsbStringType::Serial,
    })?;
    device.recv()?;

    device.send(ch9329::Command::GetParaCfg)?;
    device.recv()?;

    Ok(())
}
