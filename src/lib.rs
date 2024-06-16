// This module contains various data types, client and server related code.
pub mod client;
pub mod data_types;
pub mod server;

// Importing necessary modules and libraries.
use crate::server::add_data;
use crate::Data::{RecordData, RecordDataOption};
use data_types::DataType;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use strum_macros::Display;
use whoami::fallible;

/// Enum representing the direction of movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Direction {
    Left,
    Right,
}

/// Enum representing various types of commands.
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

// Implementing Display trait for Command enum.
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

// Enum representing different types of data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
pub enum Data {
    Command(String),
    RecordData(u128, Vec<DataType>),
    RecordDataOption(u128, Vec<Option<DataType>>),
}

// Struct representing recorded data.
#[derive(Debug, Clone)]
pub struct RecData {
    /// first the format of the current data
    pub data: Vec<Data>,
    commands: Vec<Command>,
    start_time: u128,
    right_total_d: f32,
    left_total_d: f32,
    first_time: bool,
}

// Implementing Default trait for RecData struct.
impl Default for RecData {
    fn default() -> Self {
        Self::new()
    }
}

// Implementing methods for RecData struct.
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

// Mutex for RecData and AtomicBool for server.
static REC_DATA: Mutex<RecData> = Mutex::new(RecData::new());
static SERVER: AtomicBool = AtomicBool::new(false);

// Function to set the server status.
pub fn set_server(b: bool) {
    SERVER.store(b, SeqCst);
}

// Macro to lock the mutex.
macro_rules! lm {
    ($mutex:expr) => {
        $mutex.lock().expect("Mutex poisoned")
    };
}

// Various functions to get and manipulate RecData.
pub fn get_rec_data() -> RecData {
    lm!(REC_DATA).clone()
}

pub fn get_rec_len() -> usize {
    lm!(REC_DATA).data.len()
}

pub fn get_rec_index(index: usize) -> Data {
    lm!(REC_DATA).data[index].clone()
}

pub fn get_rec_start_time() -> u128 {
    lm!(REC_DATA).start_time
}

pub fn add_command(command: Command) {
    lm!(REC_DATA).commands.push(command);
    // todo: show also the params
    lm!(REC_DATA).data.push(Data::Command(command.to_string()));
}

pub fn add_comment(comment: String) {
    lm!(REC_DATA).data.push(Data::Command(comment));
}

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

pub fn get_time() -> Option<u128> {
    if lm!(REC_DATA).start_time == 0 {
        None
    } else {
        Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                - lm!(REC_DATA).start_time,
        )
    }
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
    let rec_data = RecordData(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - start_time,
        data.clone(),
    );
    if SERVER.load(SeqCst) {
        add_data(rec_data.clone());
    }
    lm!(REC_DATA).data.push(rec_data);
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
    // todo: write time and date in this format: hh:mm:ss dd.mm.yyyy
    // Get current time, date and the name of the current user and format it in a nice way
    let now = chrono::Local::now();
    let user = whoami::username();
    let machine_name = fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    file.write_all(format!(
        "# Created on {} at {} by {} on {}\n",
        now.format("%d-%m-%Y"),
        now.format("%H:%M:%S"),
        user,
        machine_name
    ).as_bytes()).unwrap();
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
/// This Function deletes the first n entries from the data, but keeps the rest.
pub fn delete_data(n: usize) {
    if n < lm!(REC_DATA).data.len() {
        lm!(REC_DATA).data.drain(0..n);
    }
}
