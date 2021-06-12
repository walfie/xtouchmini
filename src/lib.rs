mod input;
pub mod model;
pub mod output;

pub use crate::input::{Event, EventStream, EventWithLayer};
pub use crate::output::{Command, Controller};

const MIDI_DEVICE_NAME: &'static str = "X-TOUCH MINI";
