use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;

#[derive(Debug, Serialize, Deserialize, PartialEq, FromRepr, Clone, Copy)]
// todo: make macros for the write and from_string functions
#[repr(u8)]
pub enum DataType {
    // length of data that should be there
    // todo: fix this, its a giant mess
    None(u8),
    /// right, left
    Color(i16, i16),
    // todo: track the whole distance in the same way as DrivenDistance does, or scrap this data type
    Distance(i16),
    /// right, left
    CalcSpeed(i16, i16),
    /// right, left
    SyncSpeed(i16, i16),
    /// right, left
    RealSpeeds(i16, i16),
    /// right, left
    DrivenDistance(f32, f32),
    SyncError(f32),
    /// right, left
    Correction(f32, f32),
}

impl DataType {
    pub fn write(&self) -> String {
        match self {
            // maybe we should use something like "n" instead of "null" to save space
            DataType::None(u8) => vec!["null".to_string(); Self::get_none(*u8) as usize].join(", "),
            DataType::Color(r, l) => format!("{}, {}", r, l),
            DataType::Distance(d) => format!("{}", d),
            DataType::CalcSpeed(r, l) => format!("{}, {}", r, l),
            DataType::SyncSpeed(r, l) => format!("{}, {}", r, l),
            DataType::RealSpeeds(r, l) => format!("{}, {}", r, l),
            DataType::DrivenDistance(r, l) => format!("{}, {}", r, l),
            DataType::SyncError(e) => format!("{}", e),
            DataType::Correction(r, l) => format!("{}, {}", r, l),
        }
    }

    pub fn get_none(i: u8) -> u8 {
        match i {
            0 => 0,
            1 => 2,
            2 => 1,
            3 => 2,
            4 => 2,
            5 => 2,
            6 => 2,
            7 => 1,
            8 => 2,
            _ => panic!("Unknown data type: {}", i),
        }
    }

    pub fn none(&self) -> u8 {
        match self {
            DataType::None(u8) => *u8,
            DataType::Color(_, _) => 2,
            DataType::Distance(_) => 1,
            DataType::CalcSpeed(_, _) => 2,
            DataType::SyncSpeed(_, _) => 2,
            DataType::RealSpeeds(_, _) => 2,
            DataType::DrivenDistance(_, _) => 2,
            DataType::SyncError(_) => 1,
            DataType::Correction(_, _) => 2,
        }
    }

    pub fn to_u8(&self) -> u8 {
        unsafe { (self as *const DataType as *const u8).read() }
    }

    /// This converts a string in this format "1, 30, 30" to a DataType::Color(30, 30)
    pub fn from_string(s: String) -> DataType {
        let mut parts = s.split(", ");
        let ty = parts.next().unwrap();
        let data = match DataType::from_repr(ty.parse::<u8>().unwrap()) {
            Some(d) => d,
            None => panic!("Unknown data type: {}", ty),
        };
        match data {
            DataType::None(_) => DataType::None(0),
            DataType::Color(_, _) => {
                let r = parts.next().unwrap().parse::<i16>().unwrap();
                let l = parts.next().unwrap().parse::<i16>().unwrap();
                DataType::Color(r, l)
            }
            DataType::Distance(_) => {
                let d = parts.next().unwrap().parse::<i16>().unwrap();
                DataType::Distance(d)
            }
            DataType::CalcSpeed(_, _) => {
                let r = parts.next().unwrap().parse::<i16>().unwrap();
                let l = parts.next().unwrap().parse::<i16>().unwrap();
                DataType::CalcSpeed(r, l)
            }
            DataType::SyncSpeed(_, _) => {
                let r = parts.next().unwrap().parse::<i16>().unwrap();
                let l = parts.next().unwrap().parse::<i16>().unwrap();
                DataType::SyncSpeed(r, l)
            }
            DataType::RealSpeeds(_, _) => {
                let r = parts.next().unwrap().parse::<i16>().unwrap();
                let l = parts.next().unwrap().parse::<i16>().unwrap();
                DataType::RealSpeeds(r, l)
            }
            DataType::DrivenDistance(_, _) => {
                let r = parts.next().unwrap().parse::<f32>().unwrap();
                let l = parts.next().unwrap().parse::<f32>().unwrap();
                DataType::DrivenDistance(r, l)
            }
            DataType::SyncError(_) => {
                let e = parts.next().unwrap().parse::<f32>().unwrap();
                DataType::SyncError(e)
            }
            DataType::Correction(_, _) => {
                let r = parts.next().unwrap().parse::<f32>().unwrap();
                let l = parts.next().unwrap().parse::<f32>().unwrap();
                DataType::Correction(r, l)
            }
        }
    }

    /// Writes the description of the data type like this: for Color: "right color, left color"
    pub fn write_description(&self) -> String {
        match self {
            DataType::None(_) => String::new(),
            DataType::Color(_, _) => "right color, left color".to_string(),
            DataType::Distance(_) => "dist".to_string(),
            DataType::CalcSpeed(_, _) => "right calculated v, left calculated v".to_string(),
            DataType::SyncSpeed(_, _) => "right synced v, left synced v".to_string(),
            DataType::RealSpeeds(_, _) => "right real v, left real v".to_string(),
            DataType::DrivenDistance(_, _) => "right distance, left distance".to_string(),
            DataType::SyncError(_) => "sync error".to_string(),
            DataType::Correction(_, _) => "right correction, left correction".to_string(),
        }
    }
}
