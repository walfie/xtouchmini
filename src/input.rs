use crate::model::{Button, ButtonPressed, CcValue, Knob, Layer};
use crate::MIDI_DEVICE_NAME;
use anyhow::{bail, Context as _, Result};
use futures::channel::mpsc;
use futures::task::{Context, Poll};
use futures::Stream;
use midir::{Ignore, MidiInput, MidiInputConnection};
use pin_project_lite::pin_project;
use std::convert::TryFrom;
use std::pin::Pin;

pin_project! {
    pub struct EventStream {
        connection: MidiInputConnection<()>,
        #[pin]
        stream: mpsc::UnboundedReceiver<Result<EventWithLayer>>,
    }
}

impl EventStream {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::unbounded();
        let connection = get_input_port(MIDI_DEVICE_NAME, move |event| {
            if let Err(e) = tx.unbounded_send(event) {
                eprintln!("Failed to send: {}", e);
            }
        })?;

        Ok(EventStream {
            connection,
            stream: rx,
        })
    }
}

impl Stream for EventStream {
    type Item = Result<EventWithLayer>;

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
    F: Fn(Result<EventWithLayer>) + Send + 'static,
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
                handler(EventWithLayer::try_from(bytes));
            },
            (),
        )
        .map_err(|e| midir::ConnectError::new(e.kind(), ()))?;

    Ok(connection)
}

#[derive(Debug, PartialEq, Eq)]
pub struct EventWithLayer {
    pub layer: Layer,
    pub event: Event,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    ButtonPress {
        button: Button,
        state: ButtonPressed,
    },
    KnobChange {
        knob: Knob,
        value: CcValue,
    },
    KnobPress {
        knob: Knob,
        state: ButtonPressed,
    },
    FaderChange {
        value: CcValue,
    },
}

impl TryFrom<&[u8]> for EventWithLayer {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> anyhow::Result<EventWithLayer> {
        Ok(match bytes {
            &[186, channel, value] => {
                let value = CcValue::from(value);
                let (layer, knob) = match channel {
                    1..=8 => (
                        Layer::A,
                        Knob::from_index(channel).context("invalid knob index")?,
                    ),
                    11..=18 => (
                        Layer::B,
                        Knob::from_index(channel - 10).context("invalid knob index")?,
                    ),
                    9 => {
                        return Ok(EventWithLayer {
                            layer: Layer::A,
                            event: Event::FaderChange { value },
                        })
                    }
                    10 => {
                        return Ok(EventWithLayer {
                            layer: Layer::B,
                            event: Event::FaderChange { value },
                        })
                    }
                    _ => bail!("unknown channel {}", channel),
                };

                EventWithLayer {
                    event: Event::KnobChange { knob, value },
                    layer,
                }
            }

            &[action, channel, _value] => {
                let state = match action {
                    154 => ButtonPressed::Down,
                    138 => ButtonPressed::Up,
                    _ => bail!("unknown action {}", action),
                };

                match channel {
                    0..=7 => EventWithLayer {
                        layer: Layer::A,
                        event: Event::KnobPress {
                            knob: Knob::from_index(channel + 1).context("invalid knob index")?,
                            state,
                        },
                    },

                    24..=31 => EventWithLayer {
                        layer: Layer::B,
                        event: Event::KnobPress {
                            knob: Knob::from_index(channel - 23).context("invalid knob index")?,
                            state,
                        },
                    },

                    8..=23 => EventWithLayer {
                        layer: Layer::A,
                        event: Event::ButtonPress {
                            button: Button::from_index(channel - 7)
                                .context("invalid button index")?,
                            state,
                        },
                    },

                    32..=47 => EventWithLayer {
                        layer: Layer::B,
                        event: Event::ButtonPress {
                            button: Button::from_index(channel - 31)
                                .context("invalid button index")?,
                            state,
                        },
                    },

                    _ => bail!("unexpected channel {}", channel),
                }
            }
            _ => bail!("invalid bytes"),
        })
    }
}
