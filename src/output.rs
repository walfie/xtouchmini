use crate::model::{Button, ButtonLight, Knob, Layer, RingLedBehavior, RingLighting};
use crate::MIDI_DEVICE_NAME;
use anyhow::{Context as _, Result};
use futures::channel::mpsc;
use futures::StreamExt;
use midir::{MidiOutput, MidiOutputConnection};
use std::future::Future;

pub struct Controller {
    sender: mpsc::UnboundedSender<Command>,
}

impl Controller {
    pub fn new() -> Result<(Self, impl Future<Output = ()>)> {
        let (tx, mut rx) = mpsc::unbounded::<Command>();
        let mut connection = get_output_port(MIDI_DEVICE_NAME)?;

        let worker = async move {
            while let Some(command) = rx.next().await {
                println!("Received command: {:?}", command);
                if let Err(e) = connection.send(&command.as_bytes()) {
                    // TODO
                    eprintln!("Failed to send: {}", e);
                }
            }
        };

        let controller = Self { sender: tx };

        Ok((controller, worker))
    }

    pub fn send(&mut self, command: Command) -> Result<()> {
        Ok(self.sender.unbounded_send(command)?)
    }
}

fn get_output_port(port_name: &str) -> Result<MidiOutputConnection> {
    let midi_out = MidiOutput::new(port_name)?;

    let ports = midi_out.ports();

    let out_port = ports
        .iter()
        .find(|port| midi_out.port_name(port).as_deref() == Ok(MIDI_DEVICE_NAME))
        .with_context(|| format!("could not find device {}", MIDI_DEVICE_NAME))?;

    let conn_out = midi_out
        .connect(out_port, port_name)
        .map_err(|e| midir::ConnectError::new(e.kind(), ()))?;

    Ok(conn_out)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    SetButtonLight {
        button: Button,
        state: ButtonLight,
    },
    SetLayer {
        layer: Layer,
    },
    SetMode {
        mc: bool,
    },
    SetRingLedBehavior {
        knob: Knob,
        behavior: RingLedBehavior,
    },
    SetRingLighting {
        knob: Knob,
        lighting: RingLighting,
    },
    SetKnobValue {
        knob: Knob,
        value: u8,
    },
}

impl Command {
    // Commands via this Amazon review:
    // https://amazon.com/gp/customer-reviews/R3PVLSSOLJO50D
    pub fn as_bytes(&self) -> Vec<u8> {
        use Command::*;
        match self {
            SetButtonLight { button, state } => {
                // 0x90 [0..15] n Set button LED 0=off, 1=on, 2=blink
                vec![0x90, button.to_index() - 1, state.as_u8()]
            }
            SetLayer { layer } => {
                // 0xC0 n Select layer 0=Layer A (default), 1=Layer B (ONLY IF NOT IN MC MODE)
                let layer_value = match layer {
                    Layer::A => 0,
                    Layer::B => 1,
                };

                vec![0xc0, layer_value]
            }
            SetMode { mc } => {
                // 0xB0 127 n Set mode 0=standard (default), 1=MC
                vec![0xb0, 127, if *mc { 1 } else { 0 }]
            }
            SetRingLedBehavior { knob, behavior } => {
                // 0xB0 [1..8] n Set LED ring mode 0=single, 1=pan, 2=fan, 3=spread, 4=trim
                vec![0xb0, knob.to_index(), behavior.as_u8()]
            }
            SetKnobValue { knob, value } => {
                // 0xBA [1..8] n Set knob position to n
                vec![0xba, knob.to_index(), *value]
            }
            SetRingLighting { knob, lighting } => {
                // 0xB0 [9..16] n Set LED ring illumination 0=off [1..13]=on, [14..26]=blink, 26=all on, 27=all blink
                vec![0xb0, knob.to_index() + 8, lighting.as_u8()]
            }
        }
    }
}
