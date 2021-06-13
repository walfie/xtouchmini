use serde::Serialize;
use serde_repr::Serialize_repr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    pub ver: i32,
    pub time: u64,
    pub r#type: i32,
    pub command: i32,
    #[serde(rename = "found")] // This one is lowercase for some reason
    pub found: bool,
    pub hotkey: i32,
    pub data: Vec<MessageData>,
}

impl Message {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Self {
            ver: 0,
            time,
            r#type: 1,
            command: 0,
            found: false,
            hotkey: -1,
            data: Vec::new(),
        }
    }

    pub fn new_with_param(param: Param, value: f64) -> Self {
        let mut msg = Self::new();
        msg.data.push(MessageData { param, value });
        msg
    }

    pub fn new_with_hotkey(hotkey: i32) -> Self {
        let mut msg = Self::new();
        msg.hotkey = hotkey;
        msg
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MessageData {
    #[serde(rename = "p")]
    pub param: Param,
    #[serde(rename = "v")]
    pub value: f64,
}

impl MessageData {
    fn new(param: Param, value: f64) -> Self {
        Self { param, value }
    }
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr)]
pub enum Param {
    FacePositionX = 1,
    FacePositionY,
    FacePositionZ,
    FaceAngleX,
    FaceAngleY,
    FaceAngleZ,
    MouthSmile,
    MouthOpen,
    Brows,
    TongueOut,    // iOS
    EyeOpenLeft,  // iOS/webcam
    EyeOpenRight, // iOS/webcam
    EyeLeftX,     // iOS/webcam
    EyeLeftY,     // iOS/webcam
    EyeRightX,    // iOS/webcam
    EyeRightY,    // iOS/webcam
    CheekPuff,    // iOS
    FaceAngry,    // iOS
    BrowLeftY,    // iOS/webcam
    BrowRightY,   // iOS/webcam
    MousePositionX,
    MousePositionY,
    VoiceVolume,                  // Desktop
    VoiceFrequency,               // Desktop
    VoiceVolumePlusMouthOpen,     // Desktop
    VoiceFrequencyPlusMouthSmile, // Desktop
    MouthX,                       // iOS
}
