use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use serial_read::lib::crc::*;
use serialport::SerialPort;
use std::fs::File;
use std::io::Read;
use std::{fmt::Debug, str::FromStr, time::Duration};
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

#[derive(FromBytes)]
#[repr(C, packed)]
struct StmRxMsg {
    omega1: f32,
    omega2: f32,
    omega3: f32,
    dis1: u32,
    dis2: u32,
    dis3: u32,
    dis4: u32,
    is_true: u8,
    crc: u8,
}

#[derive(Debug)]
struct DistanceData {
    distance1: f64,
    distance2: f64,
    distance3: f64,
    distance4: f64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct RawData {
    data1: u64,
    data2: u64,
    data3: u64,
    data4: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Converter {
    m1: f64,
    m2: f64,
    m3: f64,
    m4: f64,
    c1: f64,
    c2: f64,
    c3: f64,
    c4: f64,
}

const BYTE_SIZE: usize = std::mem::size_of::<StmRxMsg>();
const SAMPLE_SIZE: usize = 200;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let default_tty = String::from("/dev/ttyUSB0");
    let arg = args.get(1).unwrap_or(&default_tty);
    if arg == "test" {
        let arg = args.get(2).unwrap_or(&default_tty);
        test_sensors(arg);
        println!("Exiting");
        return;
    }
    let mut sensor_datas = Vec::new();
    let mut distance_datas = Vec::new();
    let mut converted_data = Converter::new();

    println!("Enter number of data points: ");
    let num_points = get_input_from_user::<usize>();

    for _ in 0..num_points {
        distance_datas.push(DistanceData::new());
        sensor_datas.push(RawData::new());
    }

    if let Ok(port) = serialport::new(arg, 115200)
        .timeout(Duration::from_millis(100))
        .open()
    {
        for i in 0..4 {
            println!("Provide data for sensor number {0}: ", i + 1);
            for index in 0..num_points {
                println!("Enter data: ");
                let distance = get_input_from_user::<f64>();

                let sensor_data =
                    RawData::get_mode_raw_from_serial(port.try_clone().unwrap(), SAMPLE_SIZE);
                match i {
                    0 => {
                        distance_datas[index].distance1 = distance;
                        sensor_datas[index].data1 = sensor_data.data1;
                    }
                    1 => {
                        distance_datas[index].distance2 = distance;
                        sensor_datas[index].data2 = sensor_data.data2;
                    }
                    2 => {
                        distance_datas[index].distance3 = distance;
                        sensor_datas[index].data3 = sensor_data.data3;
                    }
                    3 => {
                        distance_datas[index].distance4 = distance;
                        sensor_datas[index].data4 = sensor_data.data4;
                    }
                    _ => unreachable!(),
                }
            }
        }

        converted_data =
            Converter::from_raw_data_and_distance_data(&sensor_datas, &distance_datas).unwrap();

        dbg!(&converted_data);

        loop {
            println!("Do you want to test your model: (Y/N)");
            let reply = get_input_from_user::<String>().to_lowercase();
            if reply == "y" || reply == "yes" {
                let sensor_data =
                    RawData::get_mode_raw_from_serial(port.try_clone().unwrap(), SAMPLE_SIZE);
                let distance = converted_data.convert(sensor_data);
                dbg!(distance);
            } else {
                println!("Do you want to redo the model: (Y/N)");
                let reply = get_input_from_user::<String>().to_lowercase();
                if reply != "y" && reply != "yes" {
                    break;
                }
                println!("Which sensor do you want to redo: ");
                let sensor_num = get_input_from_user::<usize>() - 1;
                for index in 0..num_points {
                    println!("Enter data: ");
                    let distance = get_input_from_user::<f64>();

                    let sensor_data =
                        RawData::get_mode_raw_from_serial(port.try_clone().unwrap(), SAMPLE_SIZE);
                    match sensor_num {
                        0 => {
                            distance_datas[index].distance1 = distance;
                            sensor_datas[index].data1 = sensor_data.data1;
                        }
                        1 => {
                            distance_datas[index].distance2 = distance;
                            sensor_datas[index].data2 = sensor_data.data2;
                        }
                        2 => {
                            distance_datas[index].distance3 = distance;
                            sensor_datas[index].data3 = sensor_data.data3;
                        }
                        3 => {
                            distance_datas[index].distance4 = distance;
                            sensor_datas[index].data4 = sensor_data.data4;
                        }
                        _ => unreachable!(),
                    }
                }
                converted_data =
                    Converter::from_raw_data_and_distance_data(&sensor_datas, &distance_datas)
                        .unwrap();
                dbg!(&converted_data);
            }
        }
    }
    let file = File::create("sick_parameters.yaml").unwrap();
    serde_yaml::to_writer(file, &converted_data).unwrap();
}

fn test_sensors(arg: &str) -> () {
    let mut file =
        File::open("sick_parameters.yaml").expect("No file named sick_parameters.yaml found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let converted_data: Converter = from_reader(contents.as_bytes()).unwrap();
    if let Ok(port) = serialport::new(arg, 115200)
        .timeout(Duration::from_millis(100))
        .open()
    {
        loop {
            println!("Do you want to test your model: (Y/N)");
            let reply = get_input_from_user::<String>().to_lowercase();
            if reply == "n" || reply == "no" {
                return;
            }
            let sensor_data =
                RawData::get_mode_raw_from_serial(port.try_clone().unwrap(), SAMPLE_SIZE);
            let distance = converted_data.convert(sensor_data);
            dbg!(distance);
        }
    }
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
}

impl RawData {
    fn new() -> Self {
        RawData {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: 0,
        }
    }
    fn from(data1: u64, data2: u64, data3: u64, data4: u64) -> Self {
        RawData {
            data1,
            data2,
            data3,
            data4,
        }
    }
    fn from_raw_vec(raws: Vec<RawData>) -> Self {
        let len = raws.len() as f64;

        let sum = raws.iter().fold(RawData::new(), |acc, raw| RawData {
            data1: acc.data1 + raw.data1,
            data2: acc.data2 + raw.data2,
            data3: acc.data3 + raw.data3,
            data4: acc.data4 + raw.data4,
        });

        let (mean_1, mean_2, mean_3, mean_4) = (
            sum.data1 as f64 / len,
            sum.data2 as f64 / len,
            sum.data3 as f64 / len,
            sum.data4 as f64 / len,
        );

        let variance_data = raws.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, input| {
            (
                acc.0 + (input.data1 as f64 - mean_1).powi(2),
                acc.1 + (input.data2 as f64 - mean_2).powi(2),
                acc.2 + (input.data3 as f64 - mean_3).powi(2),
                acc.3 + (input.data4 as f64 - mean_4).powi(2),
            )
        });

        let (min, max) = raws.iter().fold(
            (
                (u64::MAX, u64::MAX, u64::MAX, u64::MAX),
                (u64::MIN, u64::MIN, u64::MIN, u64::MIN),
            ),
            |(min, max), raw| {
                (
                    (
                        min.0.min(raw.data1),
                        min.1.min(raw.data2),
                        min.2.min(raw.data3),
                        min.3.min(raw.data4),
                    ),
                    (
                        max.0.max(raw.data1),
                        max.1.max(raw.data2),
                        max.2.max(raw.data3),
                        max.3.max(raw.data4),
                    ),
                )
            },
        );
        println!(
            "Mean ± σ: ({0} ± {1}), ({2} ± {3}), ({4} ± {5}), ({6} ± {7})",
            mean_1,
            (variance_data.0 / len).sqrt(),
            mean_2,
            (variance_data.1 / len).sqrt(),
            mean_3,
            (variance_data.2 / len).sqrt(),
            mean_4,
            (variance_data.3 / len).sqrt()
        );

        println!(
            "Min...Max: ({0}...{1}), ({2}...{3}), ({4}...{5}), ({6}...{7})",
            min.0, max.0, min.1, max.1, min.2, max.2, min.3, max.3
        );

        // Return Mean
        RawData {
            data1: mean_1 as u64,
            data2: mean_2 as u64,
            data3: mean_3 as u64,
            data4: mean_4 as u64,
        }
    }
    fn get_mode_raw_from_serial(mut port: Box<dyn SerialPort>, num: usize) -> Self {
        let mut raw_vec: Vec<RawData> = Vec::new();
        let mut serial_state = SerialState::StartByte(None);
        let mut buf: [u8; BYTE_SIZE] = [0; BYTE_SIZE];
        port.clear(serialport::ClearBuffer::Input).unwrap();
        while raw_vec.len() < num {
            match serial_state {
                SerialState::StartByte(start_in) => {
                    if let Some(()) = start_in {
                        let msg = StmRxMsg::read_from_bytes(&buf).unwrap();
                        raw_vec.push(RawData::from(
                            msg.dis1 as u64,
                            msg.dis2 as u64,
                            msg.dis3 as u64,
                            msg.dis4 as u64,
                        ));
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
        let new_vec = raw_vec.split_off(20);
        RawData::from_raw_vec(new_vec)
    }
}

impl Converter {
    pub fn from_raw_data_and_distance_data(
        raw_samples: &Vec<RawData>,
        distance_samples: &Vec<DistanceData>,
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

    fn convert(&self, raw: RawData) -> DistanceData {
        DistanceData {
            distance1: self.m1 * raw.data1 as f64 + self.c1,
            distance2: self.m2 * raw.data2 as f64 + self.c2,
            distance3: self.m3 * raw.data3 as f64 + self.c3,
            distance4: self.m4 * raw.data4 as f64 + self.c4,
        }
    }

    fn new() -> Self {
        Self {
            m1: 0.0,
            m2: 0.0,
            m3: 0.0,
            m4: 0.0,
            c1: 0.0,
            c2: 0.0,
            c3: 0.0,
            c4: 0.0,
        }
    }
}

fn fit_linear_model(xs: &[u64], ys: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<u64>() as f64 / n;
    let mean_y = ys.iter().sum::<f64>() / n;

    let (mut cov_xy, mut var_x) = (0.0f64, 0.0f64);
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let dx = x as f64 - mean_x;
        let dy = y as f64 - mean_y;
        cov_xy += dx * dy;
        var_x += dx * dx;
    }

    let slope = cov_xy / var_x;
    let intercept = mean_y - slope * mean_x as f64;
    return (slope, intercept);
}

fn get_input_from_user<T>() -> T
where
    T: FromStr,
    T::Err: Debug,
{
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    return input.trim().parse::<T>().expect("Couldn't parse");
}
