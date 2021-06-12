mod opt;

use crate::opt::Opt;

use anyhow::Result;
use autopilot::key::{Character, Code, Flag, KeyCode};
use futures::StreamExt;
use structopt::StructOpt;
use xtouchmini::model::{Button, ButtonPressed, CcValue, Knob, State};
use xtouchmini::{Command, Controller, Event, EventStream};

const INPUT_DELAY: u64 = 0;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let (mut controller, worker) = Controller::new()?;

    let mut state = State::default();
    for command in state.to_commands() {
        controller.send(command)?;
    }

    tokio::spawn(worker);

    let client = obws::Client::connect(opt.obs_host, opt.obs_port).await?;
    client.login(opt.obs_password).await?;

    let mut stream = EventStream::new()?;

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);

            use Event::*;
            let layout = state.layer_mut(event.layer);
            let result = match event.event {
                ButtonPress {
                    button,
                    state: ButtonPressed::Down,
                } => handle_button_press(&client, &mut controller, button).await,
                KnobPress {
                    knob,
                    state: ButtonPressed::Down,
                } => handle_knob_press(&client, &mut controller, knob).await,
                KnobChange { knob, value } => {
                    let out = handle_knob_change(
                        &client,
                        &mut controller,
                        knob,
                        &Delta::new(layout.knob(&knob).value, value),
                    )
                    .await;
                    layout.knob_mut(&knob).value = value;
                    out
                }
                FaderChange { value } => {
                    let out =
                        handle_fader_change(&client, &Delta::new(layout.fader().value, value))
                            .await;
                    layout.fader_mut().value = value;
                    out
                }
                _ => Ok(()),
            };

            if let Err(e) = result {
                eprintln!("Failed to handle event: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn handle_knob_press(
    client: &obws::Client,
    controller: &mut Controller,
    knob: Knob,
) -> Result<()> {
    match knob {
        Knob::Knob8 => {
            controller.send(Command::SetKnobValue {
                knob,
                value: 0.into(),
            })?;
            let props = obws::requests::SceneItemProperties {
                item: either::Either::Left("VTubeStudio NDI"),
                rotation: Some(0.),
                ..Default::default()
            };

            client
                .scene_items()
                .set_scene_item_properties(props)
                .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_button_press(
    client: &obws::Client,
    controller: &mut Controller,
    button: Button,
) -> Result<()> {
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
                controller.send(command)?;
            }
        }
        _ => {}
    }

    Ok(())
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
    prev: CcValue,
    current: CcValue,
}

impl Delta {
    fn new(prev: CcValue, current: CcValue) -> Self {
        Self { prev, current }
    }

    fn is_increase(&self) -> bool {
        self.current.0 > self.prev.0
    }

    fn magnitude_or_1(&self) -> u8 {
        if self.current.0 == self.prev.0 {
            1
        } else {
            self.magnitude()
        }
    }

    fn magnitude(&self) -> u8 {
        if self.current.0 > self.prev.0 {
            self.current.0 - self.prev.0
        } else {
            self.prev.0 - self.current.0
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

        for i in delta.prev.0..delta.current.0 {
            let c = string.chars().nth(i as usize).unwrap_or(last);
            tap_char(c);
        }
    } else {
        for _ in 0..delta.magnitude_or_1() {
            tap_key(KeyCode::Backspace);
        }
    }
}

async fn handle_knob_change(
    client: &obws::Client,
    controller: &mut Controller,
    knob: Knob,
    delta: &Delta,
) -> Result<()> {
    match knob {
        Knob::Knob1 => type_string_or_backspace("yo", delta),
        Knob::Knob8 => {
            if delta.magnitude() == 0 {
                if delta.current.is_max() {
                    controller.send(Command::SetKnobValue {
                        knob,
                        value: CcValue::MIN,
                    })?;
                } else if delta.current.is_min() {
                    controller.send(Command::SetKnobValue {
                        knob,
                        value: CcValue::MAX,
                    })?;
                }
            }

            let props = obws::requests::SceneItemProperties {
                item: either::Either::Left("VTubeStudio NDI"),
                rotation: Some(delta.current.percentage() * 360.),
                ..Default::default()
            };

            client
                .scene_items()
                .set_scene_item_properties(props)
                .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_fader_change(client: &obws::Client, delta: &Delta) -> Result<()> {
    type_string_or_backspace("Let's go", delta);
    Ok(())
}
