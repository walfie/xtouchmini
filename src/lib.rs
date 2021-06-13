mod input;
mod model;
mod output;

pub use crate::input::EventStream;
pub use crate::model::{Button, ButtonLedState, Event, FaderValue, Knob, KnobLedStyle, KnobValue};
pub use crate::output::{Command, Controller};

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";
