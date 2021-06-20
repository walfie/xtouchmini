mod input;
pub mod keyboard;
mod model;
mod output;
pub mod vtubestudio;

pub use crate::input::EventStream;
pub use crate::model::{
    Button, ButtonLedState, ControllerState, Event, FaderValue, Knob, KnobLedStyle, KnobLedValue,
    KnobState,
};
pub use crate::output::{Command, Controller};

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";
