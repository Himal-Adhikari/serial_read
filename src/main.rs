use serialport::SerialPort;
use std::env;
use std::time::Duration;

enum Err {
    Reserch,
}

fn manage(mut port: Box<dyn SerialPort>, byte_size: usize) -> Err {
    let mut error_time: u32 = 0;
    let mut buf: [u8; 1024] = [0; 1024];

    loop {
        if let Err(_b) = port.read_exact(&mut buf[0..byte_size]) {
            error_time += 1;
        } else {
            buf[0..byte_size]
                .iter()
                .for_each(|&byte| print!("{:02X?} ", byte));
            println!();
        }
        if error_time > 50 {
            return Err::Reserch;
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
        .parse::<usize>()
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
