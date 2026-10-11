use std::convert::TryFrom;
fn main() {
    println!("{:#?}", "x".parse::<i32>().unwrap_err());
    println!("{:#?}", "x".parse::<f64>().unwrap_err());
    println!("{:#?}", "x".parse::<bool>().unwrap_err());
    println!("{:#?}", "xy".parse::<char>().unwrap_err());
    println!("{:#?}", u8::try_from(300i32).unwrap_err());
    println!("{:#?}", char::try_from(0xD800u32).unwrap_err());
    println!("{:#?}", String::from_utf8(vec![0xff]).unwrap_err());
    println!("{:?}", String::from_utf8(vec![0x41, 0xff]).unwrap_err());
    println!("{:#?}", std::str::from_utf8(&[0xff]).unwrap_err());
    println!("{:?}", Err::<i32, _>("x".parse::<i32>().unwrap_err()));
    println!("{:#?}", Err::<i32, _>("x".parse::<i32>().unwrap_err()));
}
