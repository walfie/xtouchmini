use crate::model::{Button, ButtonPressed, Knob, Layer};
use anyhow::{bail, Context};
use std::convert::TryFrom;

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
        value: u8,
    },
    KnobPress {
        knob: Knob,
        state: ButtonPressed,
    },
    FaderChange {
        value: u8,
    },
}

impl TryFrom<&[u8]> for EventWithLayer {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> anyhow::Result<EventWithLayer> {
        Ok(match bytes {
            &[186, channel, value] => {
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
