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

    let mut tcp = Framed::new(
        TcpStream::connect("127.0.0.1:25565").await?,
        LengthDelimitedCodec::new(),
    );

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

                        let msg = vtubestudio::Message::new_with_hotkey(hotkey as i32 + 1);

                        let out = serde_json::to_vec(&msg)?;
                        tcp.send(out.into()).await?;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
