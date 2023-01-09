use ft60x_rs::*;



fn main() {
    let device = list_device_details().unwrap()[0].open().unwrap();


    // let data = vec![0xAD, 0x00, 0x00, 0x00];
    // assert!(dev.write(&data).unwrap() == 4);

    // let mut data: [u8; 8] = [0; 8];
    // println!("{}", dev.read(&mut data).unwrap());
    // println!("{:?}", data);
}
