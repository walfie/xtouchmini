mod input;
mod model;
mod output;

pub use crate::input::EventStream;
pub use crate::model::{
    Button, ButtonLedState, ControllerState, Event, FaderValue, Knob, KnobLedStyle, KnobState,
    KnobValue,
};
pub use crate::output::{Command, Controller};

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";
