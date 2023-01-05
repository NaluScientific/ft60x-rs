use ft60x_rs::device::Ft60xDevice;

fn main() {
    let mut device = Ft60xDevice::open_default().unwrap();
    println!("Config: {:?}", device.config());


    let data: Vec<u8> = vec![0xAD, 0x00, 0x00, 0x00];
    println!("Sending: {:?}", data);
    device.write(&data).unwrap();

    let mut data: [u8; 8] = [0; 8];
    device.read(&mut data).unwrap();
    println!("Got: {:?}", data);
}
