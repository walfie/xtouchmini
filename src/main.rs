use anyhow::Result;
use autopilot::key::{Code, KeyCode};
use futures::StreamExt;
use xtouchmini::keyboard;
use xtouchmini::vtubestudio::Param;
use xtouchmini::*;

struct Context {
    controller: Controller,
    vtube: vtubestudio::Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (mut controller, worker) = Controller::new()?;

    tokio::spawn(worker);
    let mut stream = EventStream::new()?;

    let vtube_addr = "127.0.0.1:25565".parse()?;
    let mut vtube = vtubestudio::Client::new(vtube_addr);

    if vtube.connect().await.is_ok() {
        controller.set_button(Button::Button16, ButtonLedState::On)?;
    }

    let mut context = Context { controller, vtube };

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            let is_connected = context.vtube.is_connected();

            println!("{:?}", event); // TODO: Debug log

            let result = match event {
                Event::KnobTurned { knob, delta } => {
                    handle_knob(&mut context, knob, KnobAction::Turned { delta }).await
                }
                Event::KnobPressed { knob, is_down } => {
                    handle_knob(&mut context, knob, KnobAction::Pressed { is_down }).await
                }
                Event::ButtonPressed { button, is_down } => {
                    handle_button(&mut context, button, is_down).await
                }
                Event::FaderMoved { value } => handle_fader(&mut context, value).await,
            };

            // On error, disable the last button to indicate that VTubeStudio failed
            // TODO: Add specific error variant
            if let Err(e) = result {
                eprintln!("{}", e); // TODO: Logger
                context
                    .controller
                    .set_button(Button::Button16, ButtonLedState::Off)?;
            } else if !is_connected && context.vtube.is_connected() {
                context
                    .controller
                    .set_button(Button::Button16, ButtonLedState::On)?;
            }
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum KnobAction {
    Turned { delta: i8 },
    Pressed { is_down: bool },
}

async fn handle_fader(context: &mut Context, value: FaderValue) -> Result<()> {
    fn type_string_or_backspace(string: &str, prev: FaderValue, current: FaderValue, friction: u8) {
        let friction = friction.max(1);
        let current = current.0 / friction;
        let prev = prev.0 / friction;

        if current > prev {
            let last = if let Some(c) = string.chars().last() {
                c
            } else {
                return;
            };

            for i in prev..current {
                let c = string.chars().nth(i as usize).unwrap_or(last);
                keyboard::tap_char(c);
            }
        } else {
            for _ in 0..(prev - current) {
                keyboard::tap_key(KeyCode::Backspace);
            }
        }
    }

    if !context.controller.state().button(Button::LayerA).is_on() {
        let prev = context.controller.state().fader();
        type_string_or_backspace("Let's go", *prev, value, 3);
    }

    context.controller.set_fader(value);
    Ok(())
}

async fn handle_knob(context: &mut Context, knob: Knob, action: KnobAction) -> Result<()> {
    use KnobAction::*;

    match (knob, action) {
        // Raise arms
        (knob, action) if matches!(knob, Knob::Knob1 | Knob::Knob2) => {
            let multiplier = context.controller.state().fader().as_percent() * 0.1 + 0.01;
            let (param, multiplier) = if let Knob::Knob1 = knob {
                (Param::CheekPuff, multiplier)
            } else {
                (Param::FaceAngry, -multiplier)
            };

            let value = match action {
                Turned { delta } => (context.vtube.param(param) + (delta as f64 * multiplier))
                    .max(0.0)
                    .min(1.0),
                Pressed { is_down: true } => 0.0,
                _ => return Ok(()),
            };

            context.vtube.set_param(param, value).await?;

            let knob_value = if let Knob::Knob1 = knob {
                value / 3.0
            } else {
                1.0 - value / 3.0
            };

            context.controller.set_knob(
                knob,
                KnobLedStyle::Single,
                KnobValue::from_percent_nonzero(knob_value),
            )?;
        }
        // Spin
        (Knob::Knob8, action) => {
            let param = Param::VoiceFrequency;
            let multiplier = context.controller.state().fader().as_percent() * 10.0 + 1.0;
            let max = 360.0;

            let value = match action {
                Turned { delta } => {
                    let value = context.vtube.param(param) + (delta as f64 * multiplier);

                    if value < 0.0 {
                        max - value
                    } else if value >= max {
                        value - max
                    } else {
                        value
                    }
                }
                Pressed { is_down: true } => 0.0,
                _ => return Ok(()),
            };

            context.vtube.set_param(param, value).await?;

            context.controller.set_knob(
                knob,
                KnobLedStyle::Single,
                KnobValue::from_percent_nonzero(value / max),
            )?;
        }
        _ => {}
    }

    context.vtube.refresh_params().await?;

    Ok(())
}

async fn handle_button(context: &mut Context, button: Button, is_down: bool) -> Result<()> {
    if !is_down {
        return Ok(());
    }

    async fn set_expression(context: &mut Context, button: Button, value: f64) -> Result<()> {
        use Button::*;

        let new_value = if context.vtube.param(Param::MouthX) == value {
            0.0
        } else {
            value
        };

        context.vtube.set_param(Param::MouthX, new_value).await?;
        for b in &[Button1, Button2, Button3, Button4, Button5, Button6] {
            let state = if *b == button && new_value != 0.0 {
                ButtonLedState::On
            } else {
                ButtonLedState::Off
            };

            context.controller.set_button(*b, state)?;
        }

        Ok(())
    }

    if matches!(button, Button::LayerA | Button::LayerB) {
        context.controller.negate_button(button)?;
    } else if context.controller.state().button(Button::LayerA).is_on() {
        match button {
            Button::Button1 => set_expression(context, button, 1.0).await?, // Sad
            Button::Button2 => set_expression(context, button, 2.0).await?, // Angry
            Button::Button3 => set_expression(context, button, 3.0).await?, // Shock
            Button::Button4 => set_expression(context, button, 4.0).await?, // Smug
            Button::Button5 => set_expression(context, button, 5.0).await?, // Excited
            Button::Button6 => set_expression(context, button, 6.0).await?, // Crying
            Button::Button7 => context.vtube.toggle_hotkey(2).await?,       // Dance
            Button::Button8 => context.vtube.toggle_hotkey(3).await?,       // Dab
            Button::Button9 => {
                let was_frowning = context.vtube.param(Param::TongueOut) == 1.0;

                let (value, state) = if was_frowning {
                    (0.0, ButtonLedState::Off)
                } else {
                    (1.0, ButtonLedState::On)
                };

                context.vtube.set_param(Param::TongueOut, value).await?;
                context.controller.set_button(button, state)?;
            }
            Button::Button10 => {
                // Toggle sunglasses
                context.vtube.toggle_hotkey(1).await?;
                context.controller.negate_button(button)?;
            }
            Button::Button16 => {
                // Reset expressions
                context.vtube.toggle_hotkey(8).await?;
            }
            _ => {}
        }

        context.vtube.refresh_params().await?;
    } else {
        match button {
            Button::Button1 => keyboard::type_text("👏"),
            Button::Button2 => keyboard::type_text("🔜"),
            Button::Button3 => keyboard::type_text("👀"),
            Button::Button4 => keyboard::type_text("🙇"),
            Button::Button16 => keyboard::tap_key(KeyCode::Return),
            Button::Button8 => {
                // Find YouTube tab in Chrome, and focus on the chat input field
                osascript::JavaScript::new(include_str!("focus-youtube.js")).execute()?;
            }
            Button::Button9 => {
                use autopilot::key::Flag::{Control, Meta};
                autopilot::key::tap(&Code(KeyCode::Space), &[Meta, Control], 0, 0);
            }

            _ => {}
        }
    }

    Ok(())
}
