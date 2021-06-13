// Heavily based on the Mackie Control values from Jon Skeet's project:
// https://github.com/jskeet/DemoCode/blob/c73e36e45bd01e1327b529b3b7de300ed7f01601/XTouchMini/XTouchMini.Model/XTouchMiniMackieController.cs#L115

use anyhow::{bail, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ControllerState {
    knobs: [KnobState; 8],
    buttons: [ButtonLedState; 18],
    fader: FaderValue,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct KnobState {
    pub style: KnobLedStyle,
    pub value: KnobValue,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum Button {
    // Top row
    Button1 = 0x59,
    Button2 = 0x5a,
    Button3 = 0x28,
    Button4 = 0x29,
    Button5 = 0x2a,
    Button6 = 0x2b,
    Button7 = 0x2c,
    Button8 = 0x2d,

    // Bottom row
    Button9 = 0x57,
    Button10 = 0x58,
    Button11 = 0x5b,
    Button12 = 0x5c,
    Button13 = 0x56,
    Button14 = 0x5d,
    Button15 = 0x5e,
    Button16 = 0x5f,

    // Layer buttons
    LayerA = 0x54,
    LayerB = 0x55,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum Knob {
    // These u8 values are for knob turn messages.
    // For knob press, add 0x10 to the value.
    Knob1 = 0x01,
    Knob2,
    Knob3,
    Knob4,
    Knob5,
    Knob6,
    Knob7,
    Knob8,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive)]
pub enum ButtonLedState {
    Off = 0x00,
    On = 0x7f,
    Blink = 0x01,
}

impl Default for ButtonLedState {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct KnobValue(pub(crate) u8);

impl KnobValue {
    pub const MIN: KnobValue = KnobValue(0);

    // There are 12 LEDs, but the max knob value in MC mode is actually 11
    pub const MAX: KnobValue = KnobValue(12);

    pub fn new(value: u8) -> Self {
        Self(value.min(Self::MAX.0))
    }

    pub fn from_percent(value: f64) -> Self {
        Self::new(((Self::MAX.0 as f64) * value) as u8)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KnobLedStyle {
    /// One LED is lit
    Single,
    /// Doesn't work in MC mode
    Pan,
    /// Light all LEDs from the top until the current value, on both sides
    Fan,
    /// Same as Single in MC mode
    Spread,
    /// Light all LEDs from the top until the current value, on one side
    Trim,
}

impl Default for KnobLedStyle {
    fn default() -> Self {
        Self::Single
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FaderValue(pub u8);

impl FaderValue {
    pub const MIN: FaderValue = FaderValue(0);
    pub const MAX: FaderValue = FaderValue(127);

    pub fn as_percent(&self) -> f64 {
        self.0 as f64 / Self::MAX.0 as f64
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    ButtonPressed { button: Button, is_down: bool },
    KnobPressed { knob: Knob, is_down: bool },
    KnobTurned { knob: Knob, delta: i8 },
    FaderMoved { value: FaderValue },
}

impl TryFrom<&[u8]> for Event {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Event> {
        use Event::*;

        match bytes {
            &[0xb0, channel, value] => {
                let delta = if value >= 64 {
                    -((value - 64) as i8)
                } else {
                    value as i8
                };

                Ok(KnobTurned {
                    knob: Knob::try_from(channel - 0x0f)?,
                    delta,
                })
            }
            &[0xe8, _, value] => Ok(Event::FaderMoved {
                value: FaderValue(value),
            }),
            &[0x90, channel, state] => {
                let is_down = state != 0;

                if channel >= 0x20 && channel <= 0x27 {
                    Ok(Event::KnobPressed {
                        knob: Knob::try_from(channel - 0x1f)?,
                        is_down,
                    })
                } else {
                    Ok(Event::ButtonPressed {
                        button: Button::try_from(channel)?,
                        is_down,
                    })
                }
            }
            _ => bail!("unknown event: {:?}", bytes),
        }
    }
}
