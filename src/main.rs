use anyhow::Result;
use futures::StreamExt;
use xtouchmini::vtubestudio::Param;
use xtouchmini::*;

struct Context {
    controller: Controller,
    vtube: vtubestudio::Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (mut controller, worker) = Controller::new()?;

    // Reset controller state
    let state = ControllerState::default();
    for command in state.to_commands() {
        controller.send(command)?;
    }

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
                _ => Ok(()),
            };

            // On error, disable the last button to indicate that VTubeStudio failed
            // TODO: Add specific error variant
            if let Err(e) = result {
                eprintln!("{}", e); // TODO: Logger
                context
                    .controller
                    .set_button(Button::Button16, ButtonLedState::Off)?;
            } else if !is_connected {
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

async fn handle_knob(context: &mut Context, knob: Knob, action: KnobAction) -> Result<()> {
    use KnobAction::*;

    match (knob, action) {
        // Raise arms
        (knob, Turned { delta }) if matches!(knob, Knob::Knob1 | Knob::Knob2) => {
            let (param, multiplier) = if let Knob::Knob1 = knob {
                (Param::CheekPuff, 0.01)
            } else {
                (Param::FaceAngry, -0.01)
            };

            let value = (context.vtube.param(param) + (delta as f64 * multiplier))
                .max(0.0)
                .min(1.0);
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
            let multiplier = 1.0;
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

    match button {
        Button::Button1 => set_expression(context, button, 1.0).await?,
        Button::Button2 => set_expression(context, button, 2.0).await?,
        Button::Button3 => set_expression(context, button, 3.0).await?,
        Button::Button4 => set_expression(context, button, 4.0).await?,
        Button::Button5 => set_expression(context, button, 5.0).await?,
        Button::Button6 => set_expression(context, button, 6.0).await?,
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
        _ => {}
    }

    Ok(())
}
