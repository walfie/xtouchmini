use anyhow::Result;
use futures::StreamExt;
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

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);
        }
    }

    Ok(())
}
