use anyhow::{Context, Result};
use futures::SinkExt;
use serde::Serialize;
use serde_repr::Serialize_repr;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug)]
pub struct Client {
    addr: SocketAddr,
    tcp: Option<Framed<TcpStream, LengthDelimitedCodec>>,
    params: HashMap<Param, f64>,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            params: HashMap::new(),
            tcp: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.tcp.is_some()
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.tcp = Some(self.connect_inner().await?);
        Ok(())
    }

    pub async fn toggle_hotkey(&mut self, hotkey: i32) -> Result<()> {
        self.send_message(&Message::new_with_hotkey(hotkey)).await
    }

    pub async fn set_param(&mut self, param: Param, value: f64) -> Result<()> {
        let result = self
            .send_message(&Message::new_with_param(param, value))
            .await;

        if result.is_ok() {
            self.params.insert(param, value);
        }

        result
    }

    pub async fn refresh_params(&mut self) -> Result<()> {
        let mut msg = Message::new();

        msg.data = self
            .params
            .iter()
            .map(|(param, value)| MessageData {
                param: *param,
                value: *value,
            })
            .collect::<Vec<_>>();

        self.send_message(&msg).await
    }

    pub fn param(&self, param: Param) -> f64 {
        if let Some(value) = self.params.get(&param) {
            *value
        } else {
            0.0
        }
    }

    async fn connect_inner(&self) -> Result<Framed<TcpStream, LengthDelimitedCodec>> {
        Ok(Framed::new(
            TcpStream::connect(&self.addr)
                .await
                .context("failed to connect to VTubeStudio")?,
            LengthDelimitedCodec::new(),
        ))
    }

    async fn send_message(&mut self, msg: &Message) -> Result<()> {
        let mut tcp = if let Some(tcp) = self.tcp.take() {
            tcp
        } else {
            self.connect_inner().await?
        };

        let json = serde_json::to_vec(&msg)?;
        tcp.send(json.into())
            .await
            .context("failed to send message to VTubeStudio")?;
        self.tcp = Some(tcp);
        Ok(())
    }
}

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
            found: true,
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

#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize_repr)]
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
