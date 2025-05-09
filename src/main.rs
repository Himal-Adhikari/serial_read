use serial_read::lib::crc::*;
use serialport::SerialPort;
use std::time::Duration;
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

#[derive(FromBytes)]
#[repr(C, packed)]
struct StmRxMsg {
    omega1: f32,
    omega2: f32,
    omega3: f32,
    dis1: f32,
    dis2: f32,
    dis3: f32,
    dis4: f32,
    crc: u8,
}

struct DistanceData {
    distance1: f32,
    distance2: f32,
    distance3: f32,
    distance4: f32,
}

struct RawData {
    data1: f32,
    data2: f32,
    data3: f32,
    data4: f32,
}

#[derive(Debug)]
struct Converter {
    m1: f32,
    m2: f32,
    m3: f32,
    m4: f32,
    c1: f32,
    c2: f32,
    c3: f32,
    c4: f32,
}

const BYTE_SIZE: usize = std::mem::size_of::<StmRxMsg>();

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let default_tty = String::from("/dev/ttyUSB0");
    let arg = args.get(1).unwrap_or(&default_tty);
    let mut sensor_datas = Vec::new();
    let mut distance_datas = Vec::new();
    loop {
        println!("Trying to Open Port {}", arg);
        if let Ok(port) = serialport::new(arg, 115200)
            .timeout(Duration::from_millis(100))
            .open()
        {
            let mut input = String::new();
            println!("Do you want to add more datas: Y(YES), N(NO): ");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Couldn't read line");
            let trimmed_input = input.trim().to_lowercase();
            if trimmed_input != "yes" && trimmed_input != "y" {
                break;
            }

            distance_datas.push(DistanceData::get_distance_from_user());
            sensor_datas.push(RawData::get_mean_raw_from_serial(port, 500));
        } else {
            eprint!("Couldn't open serial port {arg}");
        }
    }
    let converted_data =
        Converter::from_raw_data_and_distance_data(sensor_datas, distance_datas).unwrap();

    dbg!(converted_data);
}

impl DistanceData {
    pub fn new() -> Self {
        DistanceData {
            distance1: 0.0,
            distance2: 0.0,
            distance3: 0.0,
            distance4: 0.0,
        }
    }
    pub fn from(distance1: f32, distance2: f32, distance3: f32, distance4: f32) -> Self {
        DistanceData {
            distance1,
            distance2,
            distance3,
            distance4,
        }
    }
    pub fn get_distance_from_user() -> Self {
        let mut distance = String::new();
        let mut distances = [0.0 as f32; 4];
        for i in 0..4 {
            std::io::stdin()
                .read_line(&mut distance)
                .expect("Failed to read line");
            let distance = distance
                .trim()
                .parse::<f32>()
                .expect("Expected a float but did not find one");
            distances[i] = distance;
        }
        DistanceData::from(distances[0], distances[1], distances[2], distances[3])
    }
}

impl RawData {
    fn new() -> Self {
        RawData {
            data1: 0.0,
            data2: 0.0,
            data3: 0.0,
            data4: 0.0,
        }
    }
    fn from(data1: f32, data2: f32, data3: f32, data4: f32) -> Self {
        RawData {
            data1,
            data2,
            data3,
            data4,
        }
    }
    fn from_raw_vec(raws: Vec<RawData>) -> Self {
        let len = raws.len() as f32;

        let sum = raws.iter().fold(RawData::new(), |acc, raw| RawData {
            data1: acc.data1 + raw.data1,
            data2: acc.data2 + raw.data2,
            data3: acc.data3 + raw.data3,
            data4: acc.data4 + raw.data4,
        });

        RawData {
            data1: sum.data1 / len,
            data2: sum.data2 / len,
            data3: sum.data3 / len,
            data4: sum.data4 / len,
        }
    }
    fn get_mean_raw_from_serial(mut port: Box<dyn SerialPort>, num: usize) -> Self {
        let mut raw_vec: Vec<RawData> = Vec::new();
        let mut serial_state = SerialState::StartByte(None);
        let mut buf: [u8; BYTE_SIZE] = [0; BYTE_SIZE];
        for _i in 0..num {
            match serial_state {
                SerialState::StartByte(start_in) => {
                    if let Some(()) = start_in {
                        let msg = StmRxMsg::read_from_bytes(&buf).unwrap();
                        raw_vec.push(RawData::from(msg.dis1, msg.dis2, msg.dis3, msg.dis4));
                        serial_state = receive_data(&mut port, &mut buf, serial_state);
                    } else {
                        serial_state = receive_data(&mut port, &mut buf, serial_state);
                    }
                }
                SerialState::Working => {
                    serial_state = receive_data(&mut port, &mut buf, serial_state);
                }
            }
        }
        RawData::from_raw_vec(raw_vec)
    }
}

impl Converter {
    pub fn from_raw_data_and_distance_data(
        raw_samples: Vec<RawData>,
        distance_samples: Vec<DistanceData>,
    ) -> Option<Self> {
        let sample_count = raw_samples.len();
        if sample_count < 2 || distance_samples.len() != sample_count {
            return None;
        }

        let mut raw_ch1 = Vec::with_capacity(sample_count);
        let mut dist_ch1 = Vec::with_capacity(sample_count);

        let mut raw_ch2 = Vec::with_capacity(sample_count);
        let mut dist_ch2 = Vec::with_capacity(sample_count);

        let mut raw_ch3 = Vec::with_capacity(sample_count);
        let mut dist_ch3 = Vec::with_capacity(sample_count);

        let mut raw_ch4 = Vec::with_capacity(sample_count);
        let mut dist_ch4 = Vec::with_capacity(sample_count);

        for (raw, dist) in raw_samples.iter().zip(distance_samples.iter()) {
            raw_ch1.push(raw.data1);
            dist_ch1.push(dist.distance1);

            raw_ch2.push(raw.data2);
            dist_ch2.push(dist.distance2);

            raw_ch3.push(raw.data3);
            dist_ch3.push(dist.distance3);

            raw_ch4.push(raw.data4);
            dist_ch4.push(dist.distance4);
        }

        let (slope1, intercept1) = fit_linear_model(&raw_ch1, &dist_ch1);
        let (slope2, intercept2) = fit_linear_model(&raw_ch2, &dist_ch2);
        let (slope3, intercept3) = fit_linear_model(&raw_ch3, &dist_ch3);
        let (slope4, intercept4) = fit_linear_model(&raw_ch4, &dist_ch4);

        Some(Converter {
            m1: slope1,
            c1: intercept1,
            m2: slope2,
            c2: intercept2,
            m3: slope3,
            c3: intercept3,
            m4: slope4,
            c4: intercept4,
        })
    }

    pub fn convert(&self, raw: RawData) -> DistanceData {
        DistanceData {
            distance1: self.m1 * raw.data1 + self.c1,
            distance2: self.m2 * raw.data2 + self.c2,
            distance3: self.m3 * raw.data3 + self.c3,
            distance4: self.m4 * raw.data4 + self.c4,
        }
    }
}

fn fit_linear_model(xs: &[f32], ys: &[f32]) -> (f32, f32) {
    let n = xs.len() as f32;
    let mean_x = xs.iter().sum::<f32>() / n;
    let mean_y = ys.iter().sum::<f32>() / n;

    let (mut cov_xy, mut var_x) = (0.0f32, 0.0f32);
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov_xy += dx * dy;
        var_x += dx * dx;
    }

    let slope = cov_xy / var_x;
    let intercept = mean_y - slope * mean_x;
    (slope, intercept)
}
