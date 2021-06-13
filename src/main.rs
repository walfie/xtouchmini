use anyhow::Result;
use futures::StreamExt;
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
            println!("{:?}", event); // TODO: Debug log

            let result = match event {
                Event::KnobTurned { knob, delta } => handle_knob(&mut context, knob, delta).await,
                Event::ButtonPressed { button, is_down } => {
                    handle_button(&mut context, button, is_down).await
                }
                _ => Ok(()),
            };

            if let Err(e) = result {
                eprintln!("{}", e);
            }
        }
    }

    Ok(())
}

async fn handle_knob(context: &mut Context, knob: Knob, delta: i8) -> Result<()> {
    use vtubestudio::Param;

    match knob {
        // Raise arms
        Knob::Knob1 | Knob::Knob2 => {
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
        _ => {}
    }

    Ok(())
}

async fn handle_button(context: &mut Context, button: Button, is_down: bool) -> Result<()> {
    let state = if is_down {
        ButtonLedState::On
    } else {
        ButtonLedState::Off
    };

    context
        .controller
        .send(Command::SetButtonLedState { button, state })?;

    if is_down {
        let hotkey: usize = button.into();
        let hotkey = hotkey as i32 + 1;

        if let Err(e) = context.vtube.toggle_hotkey(hotkey).await {
            eprintln!("{}", e); // TODO: Logger
            context
                .controller
                .set_button(Button::Button16, ButtonLedState::Off)?;
        } else {
            context
                .controller
                .set_button(Button::Button16, ButtonLedState::On)?;
        }
    }

    Ok(())
}
