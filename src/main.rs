use anyhow::Result;
use futures::StreamExt;
use xtouchmini::*;

#[tokio::main]
async fn main() -> Result<()> {
    let (mut controller, worker) = Controller::new()?;

    controller.send(Command::SetButtonLedState {
        button: Button::Button1,
        state: ButtonLedState::On,
    })?;

    controller.send(Command::SetKnobLedState {
        knob: Knob::Knob5,
        state: KnobState {
            style: KnobLedStyle::Spread,
            value: KnobValue::from_percent(0.6),
        },
    })?;

    tokio::spawn(worker);
    let mut stream = EventStream::new()?;

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);
        }
    }

    Ok(())
}
