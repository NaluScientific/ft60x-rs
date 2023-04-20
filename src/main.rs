use ft60x_rs::*;

fn main() {
    println!("Devices: {:?}", list_devices().unwrap());

    let device = list_devices().unwrap()[0].open().unwrap();
    println!("{:?}", device.is_usb3().unwrap());
    println!("{:?}", device.device_descriptor());

    let data = vec![0xAD, 0x00, 0x00, 0x00];
    assert!(device.write(Pipe::Out0, &data).unwrap() == 4);

    let mut data: [u8; 8] = [0; 8];
    println!("{}", device.read(Pipe::In0, &mut data).unwrap());
    println!("{:?}", data);
}
