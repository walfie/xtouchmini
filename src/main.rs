use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use xtouchmini::*;

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

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);
            match event {
                Event::ButtonPressed { button, is_down } => {
                    let state = if is_down {
                        ButtonLedState::On
                    } else {
                        ButtonLedState::Off
                    };

                    controller.send(Command::SetButtonLedState { button, state })?;

                    if is_down {
                        let hotkey: usize = button.into();
                        let hotkey = hotkey as i32 + 1;

                        if let Err(e) = vtube.toggle_hotkey(hotkey).await {
                            eprintln!("{}", e); // TODO: Logger
                            controller.set_button(Button::Button16, ButtonLedState::Off)?;
                        } else {
                            controller.set_button(Button::Button16, ButtonLedState::On)?;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
