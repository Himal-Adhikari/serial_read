use serialport::SerialPort;
use std::env;
use std::time::Duration;

enum Err {
    Reserch,
}

fn manage(mut port: Box<dyn SerialPort>, byte_size: u32) -> Err {
    let mut error_time: u32 = 0;
    let mut print_count: u32 = 0;

    loop {
        let mut byte = vec![0; 1];
        if let Err(_b) = port.read_exact(&mut byte) {
            error_time += 1;
        } else {
            print!("{:02X?} ", byte.first().unwrap());
            print_count += 1;
        }
        if error_time > 50 {
            return Err::Reserch;
        }
        if print_count >= byte_size {
            println!();
            print_count = 0;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_tty = String::from("/dev/ttyUSB0");
    let arg = args.get(1).unwrap_or(&default_tty);
    let byte_size = args
        .get(2)
        .unwrap_or(&String::from("1"))
        .parse::<u32>()
        .expect("Expected a number as the second argument");
    loop {
        println!("Trying to Open Port {}", arg);
        if let Ok(port) = serialport::new(arg, 115200)
            .timeout(Duration::from_millis(100))
            .open()
        {
            manage(port, byte_size);
        }

        let ten_millis = Duration::from_millis(10);

        std::thread::sleep(ten_millis);
    }
}
