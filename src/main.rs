use anyhow::Result;
use futures::StreamExt;

use autopilot::key::{Character, Code, Flag, KeyCode};
use xtouchmini::model::{Button, ButtonPressed, State};
use xtouchmini::{Controller, Event, EventStream};

#[tokio::main]
async fn main() -> Result<()> {
    let (mut controller, worker) = Controller::new()?;

    let mut state = State::default();
    for command in state.to_commands() {
        controller.send(command)?;
    }

    tokio::spawn(worker);

    let mut stream = EventStream::new()?;

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);

            use Event::*;
            let layout = state.layer_mut(event.layer);
            match event.event {
                ButtonPress {
                    button,
                    state: ButtonPressed::Down,
                } => handle_button_press(button, &mut controller),
                KnobChange { knob, value } => {
                    handle_knob_change(value, layout.knob(&knob).value);
                    layout.knob_mut(&knob).value = value;
                }
                FaderChange { value } => {
                    handle_fader_change(value, layout.fader().value);
                    layout.fader_mut().value = value;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_button_press(button: Button, controller: &mut Controller) {
    match button {
        Button::Button16 => {
            autopilot::key::type_string(":_lighto::_hmm::_lighto:", &[], 0., 0.);
        }
        Button::Button7 => {
            autopilot::key::tap(&Code(KeyCode::Tab), &[Flag::Shift, Flag::Control], 0, 0);
        }
        Button::Button8 => {
            autopilot::key::tap(&Code(KeyCode::Tab), &[Flag::Control], 0, 0);
        }
        Button::Button15 => {
            // Reset
            let state = State::default();
            for command in state.to_commands() {
                if let Err(e) = controller.send(command) {
                    eprintln!("{}", e);
                }
            }
        }
        _ => {}
    }
}

fn handle_knob_change(value: u8, prev_value: u8) {
    let delay: u64 = 0;
    let string = "yo";
    if value == prev_value && value == 0 {
        autopilot::key::tap(&Code(KeyCode::Backspace), &[], delay, 0);
    } else if value > prev_value {
        for i in prev_value..value {
            let c = string.chars().nth(i as usize).unwrap_or('o');
            autopilot::key::tap(&Character(c), &[], delay, 0);
        }
    } else {
        for _ in 0..(prev_value - value) {
            autopilot::key::tap(&Code(KeyCode::Backspace), &[], delay, 0);
        }
    }
}

fn handle_fader_change(value: u8, prev_value: u8) {
    let delay: u64 = 0;
    let string = "Let's go";
    if value > prev_value {
        for i in prev_value..value {
            let c = string.chars().nth(i as usize).unwrap_or('o');
            autopilot::key::tap(&Character(c), &[], delay, 0);
        }
    } else {
        for _ in 0..(prev_value - value) {
            autopilot::key::tap(&Code(KeyCode::Backspace), &[], delay, 0);
        }
    }
}
