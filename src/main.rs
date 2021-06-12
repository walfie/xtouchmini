mod input;
mod model;
mod output;

use anyhow::{Context, Result};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::convert::TryFrom;
use std::io::stdin;

use crate::input::EventWithLayer;
use crate::model::{Button, ButtonLight, Knob, Layer, RingLedBehavior, State};
use crate::output::Command;

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";

fn get_input_port<F>(port_name: &str, handler: F) -> Result<MidiInputConnection<()>>
where
    F: Fn(Result<EventWithLayer>) + Send + 'static,
{
    let mut midi_in = MidiInput::new(port_name)?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();

    let in_port = ports
        .iter()
        .find(|port| midi_in.port_name(port).as_deref() == Ok(MIDI_DEVICE_NAME))
        .with_context(|| format!("could not find device {}", MIDI_DEVICE_NAME))?;

    let connection = midi_in
        .connect(
            in_port,
            port_name,
            move |_timestamp, bytes, ()| {
                handler(EventWithLayer::try_from(bytes));
            },
            (),
        )
        .map_err(|e| midir::ConnectError::new(e.kind(), ()))?;

    Ok(connection)
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

fn main() -> anyhow::Result<()> {
    let _conn_in = get_input_port("input-reader", |event| {
        println!("{:?}", event);
    })?;

    let mut conn_out = get_output_port("output-reader")?;

    let state = State::default();
    for command in state.to_commands() {
        conn_out.send(&command.as_bytes())?;
    }

    /*
    conn_out.send(&Command::ChangeLayer { layer: Layer::A }.as_bytes())?;

    conn_out.send(
        &Command::SetButtonLight {
            button: Button::Button5,
            state: ButtonLight::On,
        }
        .as_bytes(),
    )?;

    conn_out.send(
        &Command::ChangeRingLedBehavior {
            knob: Knob::Knob3,
            behavior: RingLedBehavior::Trim,
        }
        .as_bytes(),
    )?;

    conn_out.send(
        &Command::SetKnobValue {
            knob: Knob::Knob4,
            value: 60,
        }
        .as_bytes(),
    )?;

    conn_out.send(&Command::ChangeLayer { layer: Layer::B }.as_bytes())?;

    conn_out.send(
        &Command::SetButtonLight {
            button: Button::Button2,
            state: ButtonLight::On,
        }
        .as_bytes(),
    )?;
    */

    let mut input = String::new();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
