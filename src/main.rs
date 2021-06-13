use anyhow::Result;
use futures::StreamExt;
use xtouchmini::EventStream;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = EventStream::new()?;

    while let Some(event_opt) = stream.next().await {
        if let Ok(event) = event_opt {
            println!("{:?}", event);
        }
    }

    Ok(())
}
