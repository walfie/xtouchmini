use anyhow::Result;
use futures::StreamExt;

use xtouchmini::model::{Button, ButtonPressed, State};
use xtouchmini::{Controller, Event, EventStream};

#[tokio::main]
async fn main() -> Result<()> {
    let (mut controller, worker) = Controller::new()?;

    let state = State::default();
    for command in state.to_commands() {
        controller.send(command)?;
    }

    tokio::spawn(worker);

    let mut stream = EventStream::new()?;

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);

            if let Event::ButtonPress {
                button: Button::Button16,
                state: ButtonPressed::Down,
            } = event.event
            {
                // Reset
                let state = State::default();
                for command in state.to_commands() {
                    if let Err(e) = controller.send(command) {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
