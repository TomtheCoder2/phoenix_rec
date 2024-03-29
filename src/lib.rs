pub mod data_types;
pub mod client;
pub mod server;

use data_types::DataType;
use std::fmt::{Debug, Display};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::Data::{RecordData, RecordDataOption};

/// Directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Direction {
    Left,
    Right,
}

/// `Command` public enum representing various types of commands. Includes Turn, TurnRadius, DriveDist, DriveLine, AlignDist, AlignLine, and TurnOneWheel.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    Turn(i16),
    // radius, angle
    TurnRadius(i16, i16),
    DriveDist(i16),
    DriveLine(i16),
    AlignDist(i16),
    AlignLine(i16),
    TurnOneWheel(i16, Direction),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Turn(a) => write!(f, "Turn({})", a),
            Command::TurnRadius(r, a) => write!(f, "TurnRadius({}, {})", r, a),
            Command::DriveDist(d) => write!(f, "DriveDist({})", d),
            Command::DriveLine(d) => write!(f, "DriveLine({})", d),
            Command::AlignDist(d) => write!(f, "AlignDist({})", d),
            Command::AlignLine(d) => write!(f, "AlignLine({})", d),
            Command::TurnOneWheel(a, d) => write!(f, "TurnOneWheel({}, {})", a, d),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
pub enum Data {
    Command(String),
    RecordData(u128, Vec<DataType>),
    RecordDataOption(u128, Vec<Option<DataType>>),
}

struct RecData {
    /// first the format of the current data
    data: Vec<Data>,
    commands: Vec<Command>,
    start_time: u128,
    right_total_d: f32,
    left_total_d: f32,
    first_time: bool,
}

impl Default for RecData {
    fn default() -> Self {
        Self::new()
    }
}

impl RecData {
    const fn new() -> Self {
        Self {
            data: vec![],
            commands: vec![],
            start_time: 0,
            right_total_d: 0.0,
            left_total_d: 0.0,
            first_time: true,
        }
    }
}

static REC_DATA: Mutex<RecData> = Mutex::new(RecData::new());

/// stands for lock_mutex
macro_rules! lm {
    ($mutex:expr) => {
        $mutex.lock().expect("Mutex poisoned")
    };
}

pub fn add_command(command: Command) {
    lm!(REC_DATA).commands.push(command);
    // todo: show also the params
    lm!(REC_DATA).data.push(Data::Command(command.to_string()));
}

/// Connects all the elements from COMMANDS to one string in the following format:<br>
/// data_command1(data1,data2,data3)_command2(data1,data2,data3)_...
pub fn get_data_name() -> String {
    println!("COMMANDS: {:?}", lm!(REC_DATA).commands);
    "data_".to_string()
        + &*lm!(REC_DATA)
            .commands
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("_")
            .replace(' ', "")
            .to_string()
}

pub fn save_data(mut data: Vec<DataType>) {
    if lm!(REC_DATA).start_time == 0 {
        lm!(REC_DATA).start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
    }
    // check if the data contains the same DataType multiple times
    let mut had = vec![];
    for d in &data {
        if had.contains(d) {
            panic!("data contains the same DataType multiple times");
        }
        had.push(*d);
    }
    // cache the total distance, because i cant look the mutex twice
    let right_total_d = lm!(REC_DATA).right_total_d;
    let left_total_d = lm!(REC_DATA).left_total_d;
    // add RIGHT_TOTAL_D and LEFT_TOTAL_D to the DataType::DrivenDistance element
    for d in data.iter_mut() {
        if let DataType::DrivenDistance(r, l) = d {
            *d = DataType::DrivenDistance(right_total_d + *r, left_total_d + *l);
        }
    }
    let start_time = lm!(REC_DATA).start_time;
    lm!(REC_DATA).data.push(RecordData(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - start_time,
        data.clone(),
    ));
}

pub fn save_record_data(data: Data) {
    lm!(REC_DATA).data.push(data);
}

pub fn update_total_distance(right: f32, left: f32) {
    if !lm!(REC_DATA).first_time {
        lm!(REC_DATA).right_total_d += right;
        lm!(REC_DATA).left_total_d += left;
    } else {
        lm!(REC_DATA).first_time = false;
    }
}

pub fn get_right_total_distance() -> f32 {
    lm!(REC_DATA).right_total_d
}

pub fn get_left_total_distance() -> f32 {
    lm!(REC_DATA).left_total_d
}

pub fn write_data(file_name: String) {
    get_data_name();
    println!(
        "Writing data to {}: len: {}",
        file_name,
        lm!(REC_DATA).data.len()
    );
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_name)
        .unwrap();
    file.set_len(0).unwrap();
    // println!("Data: {:?}", DATA);
    let mut data = vec![];
    let mut used = vec![];
    for dat in &*lm!(REC_DATA).data {
        match dat {
            RecordData(t, d) => {
                // todo: maybe index compression
                let mut temp = vec![];
                for i in d {
                    // convert to u8
                    let index = i.to_u8();
                    while temp.len() <= index as usize {
                        temp.push(None);
                    }
                    if !used.contains(&index) {
                        used.push(index);
                    }
                    temp[index as usize] = Some(*i);
                }
                data.push(RecordDataOption(*t, temp));
            }
            Data::Command(s) => {
                data.push(Data::Command(s.clone()));
            }
            _ => {
                panic!("Data is not RecordData or Command");
            }
        }
    }
    used.sort();
    file.write_all(
        format!(
            "time, {}\n",
            used.iter()
                .map(|i| DataType::from_repr(*i)
                    .expect("DataType not found")
                    .write_description())
                .filter(|x| x != &"".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .as_bytes(),
    )
    .unwrap();
    file.write_all(b"# Phoenix data\n").unwrap();
    file.write_all(format!("# {}\n", get_data_name()).as_bytes())
        .unwrap();
    // todo reimplement this
    // file.write_all(
    //     format!(
    //         "# k_p_drive: {}, k_i_drive: {}, k_d_drive: {}\n",
    //         KPDRIVE.load(SeqCst),
    //         KIDRIVE.load(SeqCst),
    //         KDDRIVE.load(SeqCst)
    //     )
    //     .as_bytes(),
    // )
    // .unwrap();
    // println!("Data: {:?}", data);
    for data in data {
        match data {
            RecordDataOption(t, d) => {
                file.write_all(
                    format!(
                        "{}, {}\n",
                        t,
                        d.iter()
                            .enumerate()
                            .map(|(i, x)| {
                                if let Some(x) = x {
                                    x.write()
                                } else if used.contains(&(i as u8)) {
                                    DataType::None(i as u8).write()
                                } else {
                                    // dont print null, because its not needed and we want to save disc space
                                    "".to_string()
                                }
                            })
                            .filter(|x| x != &"".to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .as_bytes(),
                )
                .unwrap();
            }
            Data::Command(s) => {
                file.write_all(format!("# {}\n", s).as_bytes()).unwrap();
            }
            _ => {
                panic!("Data::RecordDataOption or Data::Command expected");
            }
        }
    }
}

pub fn clear_data() {
    *lm!(REC_DATA) = RecData::default();
}
