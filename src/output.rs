use crate::model::*;
use crate::MIDI_DEVICE_NAME;
use anyhow::{Context as _, Result};
use futures::channel::mpsc;
use futures::StreamExt;
use midir::{MidiOutput, MidiOutputConnection};
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Controller {
    sender: mpsc::UnboundedSender<Command>,
    state: ControllerState,
}

impl Controller {
    pub fn new() -> Result<(Self, impl Future<Output = ()>)> {
        let (tx, mut rx) = mpsc::unbounded::<Command>();
        let mut connection = get_output_port(MIDI_DEVICE_NAME)?;

        let worker = async move {
            while let Some(command) = rx.next().await {
                if let Err(e) = connection.send(&command.as_bytes()) {
                    // TODO
                    eprintln!("Failed to send command to controller: {}", e);
                }
            }
        };

        // Reset controller state
        let state = ControllerState::default();
        for command in state.to_commands() {
            tx.unbounded_send(command)?;
        }

        let mut controller = Self { sender: tx, state };

        controller.send(Command::SetOperationMode {
            mode: OperationMode::MackieControl,
        })?;

        Ok((controller, worker))
    }

    fn send(&mut self, command: Command) -> Result<()> {
        Ok(self.sender.unbounded_send(command)?)
    }

    pub fn set_button(&mut self, button: Button, state: ButtonLedState) -> Result<()> {
        *self.state.button_mut(button) = state;
        self.send(Command::SetButtonLedState { button, state })
    }

    pub fn set_knob(&mut self, knob: Knob, style: KnobLedStyle, value: KnobValue) -> Result<()> {
        let state = {
            let state = self.state.knob_mut(knob);
            state.style = style;
            state.value = value;
            state.clone()
        };

        self.send(Command::SetKnobLedState { knob, state })
    }

    pub fn set_fader(&mut self, value: FaderValue) {
        self.state.fader_mut().0 = value.0;
    }

    pub fn state(&self) -> &ControllerState {
        &self.state
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
    SetButtonLedState {
        button: Button,
        state: ButtonLedState,
    },
    SetKnobLedState {
        knob: Knob,
        state: KnobState,
    },
    SetOperationMode {
        mode: OperationMode,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationMode {
    Standard,
    MackieControl,
}

impl Command {
    pub fn as_bytes(&self) -> [u8; 3] {
        use Command::*;
        match self {
            SetButtonLedState { button, state } => [0x90, button.to_midi(), state.to_midi()],
            SetKnobLedState { knob, state } => {
                use KnobLedStyle::*;
                let value = state.value.0;
                let midi_value = match state.style {
                    Single => value,
                    Trim => value + 0x10,
                    Fan => value + 0x20,
                    Spread => value + 0x40,
                    Pan => value + 0x50, // This doesn't actually do anything in MC mode
                };

                [0xb0, 0x2f + knob.to_midi(), midi_value]
            }
            SetOperationMode { mode } => {
                let data = match mode {
                    OperationMode::Standard => 0,
                    OperationMode::MackieControl => 1,
                };
                [0xb0, 0x7f, data]
            }
        }
    }
}
