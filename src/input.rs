use crate::model::Event;
use crate::MIDI_DEVICE_NAME;
use anyhow::{Context as _, Result};
use futures::channel::mpsc;
use futures::task::{Context, Poll};
use futures::Stream;
use midir::{Ignore, MidiInput, MidiInputConnection};
use pin_project_lite::pin_project;
use std::convert::TryFrom;
use std::pin::Pin;
use tracing::error;

pin_project! {
    pub struct EventStream {
        connection: MidiInputConnection<()>,
        #[pin]
        stream: mpsc::UnboundedReceiver<Result<Event>>,
    }
}

impl EventStream {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::unbounded();
        let connection = get_input_port(MIDI_DEVICE_NAME, move |event| {
            if let Err(error) = tx.unbounded_send(event) {
                error!(?error, "Failed to send controller event to stream");
            }
        })?;

        Ok(EventStream {
            connection,
            stream: rx,
        })
    }
}

impl Stream for EventStream {
    type Item = Result<Event>;

    fn poll_next(
        self: Pin<&mut Self>,
        context: &mut Context,
    ) -> Poll<Option<<Self as futures::Stream>::Item>> {
        let this = self.project();
        this.stream.poll_next(context)
    }
}

fn get_input_port<F>(port_name: &str, handler: F) -> Result<MidiInputConnection<()>>
where
    F: Fn(Result<Event>) + Send + 'static,
{
    let mut midi_in = MidiInput::new(port_name)?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();

    let in_port = ports
        .iter()
        .find(|port| midi_in.port_name(port).as_deref() == Ok(MIDI_DEVICE_NAME))
        .with_context(|| format!("could not find device {}", MIDI_DEVICE_NAME))?;

    let connection = midi_in
        .connect(
            in_port,
            port_name,
            move |_timestamp, bytes, ()| {
                handler(Event::try_from(bytes));
            },
            (),
        )
        .map_err(|e| midir::ConnectError::new(e.kind(), ()))?;

    Ok(connection)
}
