use anyhow::Result;
use autopilot::key::{Character, Code, Flag, KeyCode};
use futures::StreamExt;
use xtouchmini::model::{Button, ButtonPressed, Knob, State};
use xtouchmini::{Controller, Event, EventStream};

const INPUT_DELAY: u64 = 0;

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
                    handle_knob_change(&knob, &Delta::new(layout.knob(&knob).value, value));
                    layout.knob_mut(&knob).value = value;
                }
                FaderChange { value } => {
                    handle_fader_change(&Delta::new(layout.fader().value, value));
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
        Button::Button1 => {
            type_text(":_lighto::_hmm::_lighto:");
        }
        Button::Button7 => {
            autopilot::key::tap(
                &Code(KeyCode::Tab),
                &[Flag::Shift, Flag::Control],
                INPUT_DELAY,
                0,
            );
        }
        Button::Button8 => {
            autopilot::key::tap(&Code(KeyCode::Tab), &[Flag::Control], INPUT_DELAY, 0);
        }
        Button::Button16 => {
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

fn tap_key(code: KeyCode) {
    autopilot::key::tap(&Code(code), &[], INPUT_DELAY, 0);
}

fn tap_char(c: char) {
    autopilot::key::tap(&Character(c), &[], INPUT_DELAY, 0);
}

fn type_text(text: &str) {
    autopilot::key::type_string(text, &[], 0., 0.);
}

struct Delta {
    prev: u8,
    current: u8,
}

impl Delta {
    fn new(prev: u8, current: u8) -> Self {
        Self { prev, current }
    }

    fn is_increase(&self) -> bool {
        self.current > self.prev
    }

    fn magnitude_or_1(&self) -> u8 {
        if self.current == self.prev {
            1
        } else {
            self.magnitude()
        }
    }

    fn magnitude(&self) -> u8 {
        if self.current > self.prev {
            self.current - self.prev
        } else {
            self.prev - self.current
        }
    }
}

fn type_string_or_backspace(string: &str, delta: &Delta) {
    if delta.is_increase() {
        let last = if let Some(c) = string.chars().last() {
            c
        } else {
            return;
        };

        for i in delta.prev..delta.current {
            let c = string.chars().nth(i as usize).unwrap_or(last);
            tap_char(c);
        }
    } else {
        for _ in 0..delta.magnitude_or_1() {
            tap_key(KeyCode::Backspace);
        }
    }
}

fn handle_knob_change(knob: &Knob, delta: &Delta) {
    match knob {
        Knob::Knob1 => type_string_or_backspace("yo", delta),
        _ => {}
    }
}

fn handle_fader_change(delta: &Delta) {
    type_string_or_backspace("Let's go", delta);
}
