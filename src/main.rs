use anyhow::{Context, Result};
use futures::StreamExt;
use midir::{MidiOutput, MidiOutputConnection};
use std::io::stdin;

use xtouchmini::model::State;
use xtouchmini::EventStream;

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = EventStream::new()?;

    tokio::spawn(async move {
        while let Some(event_opt) = stream.next().await {
            if let Ok(event) = event_opt {
                println!("{:?}", event);
            }
        }
    });

    let mut conn_out = get_output_port("output-reader")?;

    let state = State::default();
    for command in state.to_commands() {
        conn_out.send(&command.as_bytes())?;
    }

    /*
    conn_out.send(&Command::SetLayer { layer: Layer::A }.as_bytes())?;

    conn_out.send(
        &Command::SetButtonLight {
            button: Button::Button5,
            state: ButtonLight::On,
        }
        .as_bytes(),
    )?;

    conn_out.send(
        &Command::SetRingLedBehavior {
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

    conn_out.send(&Command::SetLayer { layer: Layer::B }.as_bytes())?;

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
