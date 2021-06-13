mod input;
mod model;

pub use crate::input::EventStream;
pub use crate::model::{Button, Event, FaderValue, Knob};

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";
