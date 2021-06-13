// Heavily based on the Mackie Control values from Jon Skeet's project:
// https://github.com/jskeet/DemoCode/blob/c73e36e45bd01e1327b529b3b7de300ed7f01601/XTouchMini/XTouchMini.Model/XTouchMiniMackieController.cs#L115

use crate::output::Command;
use anyhow::{bail, Context, Result};
use num_enum::IntoPrimitive;
use std::convert::TryFrom;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ControllerState {
    knobs: [KnobState; 8],
    buttons: [ButtonLedState; 18],
    fader: FaderValue,
}

impl ControllerState {
    pub fn knob(&self, knob: Knob) -> &KnobState {
        &self.knobs[knob.to_index()]
    }

    pub fn knob_mut(&mut self, knob: Knob) -> &mut KnobState {
        &mut self.knobs[knob.to_index()]
    }

    pub fn button(&self, button: Button) -> &ButtonLedState {
        &self.buttons[button.to_index()]
    }

    pub fn button_mut(&mut self, button: Button) -> &mut ButtonLedState {
        &mut self.buttons[button.to_index()]
    }

    pub fn fader(&self) -> &FaderValue {
        &self.fader
    }

    pub fn fader_mut(&mut self) -> &mut FaderValue {
        &mut self.fader
    }

    pub fn to_commands<'a>(&'a self) -> impl Iterator<Item = Command> + 'a {
        let knobs = Knob::iter()
            .enumerate()
            .map(move |(i, knob)| Command::SetKnobLedState {
                knob,
                state: self.knobs[i].clone(),
            });

        let buttons =
            Button::iter()
                .enumerate()
                .map(move |(i, button)| Command::SetButtonLedState {
                    button,
                    state: self.buttons[i].clone(),
                });

        knobs.chain(buttons)
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct KnobState {
    pub style: KnobLedStyle,
    pub value: KnobValue,
}

macro_rules! impl_midi {
    ($($variant:ident => $value:expr),+ $(,)?) => (
        pub fn to_midi(&self) -> u8 {
            match self {
                $(Self::$variant => $value,)*
            }
        }

        pub fn from_midi(value: u8) -> Option<Self> {
            Some(match value {
                $($value => Self::$variant,)*
                _ => return None,
            })
        }

        pub fn to_index(&self) -> usize {
            (*self).into()
        }

        pub fn from_index(index: usize) -> Option<Self> {
            Self::iter().find(|item| item.to_index() == index)
        }
    )
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, EnumIter)]
pub enum Button {
    // Top row
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,

    // Bottom row
    Button9,
    Button10,
    Button11,
    Button12,
    Button13,
    Button14,
    Button15,
    Button16,

    // Layer buttons
    LayerA,
    LayerB,
}

impl Button {
    impl_midi! {
        Button1 => 0x59,
        Button2 => 0x5a,
        Button3 => 0x28,
        Button4 => 0x29,
        Button5 => 0x2a,
        Button6 => 0x2b,
        Button7 => 0x2c,
        Button8 => 0x2d,
        Button9 => 0x57,
        Button10 => 0x58,
        Button11 => 0x5b,
        Button12 => 0x5c,
        Button13 => 0x56,
        Button14 => 0x5d,
        Button15 => 0x5e,
        Button16 => 0x5f,
        LayerA => 0x54,
        LayerB => 0x55,
    }
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, EnumIter)]
pub enum Knob {
    // These u8 values are for knob turn messages.
    // For knob press, add 0x10 to the value.
    Knob1,
    Knob2,
    Knob3,
    Knob4,
    Knob5,
    Knob6,
    Knob7,
    Knob8,
}

impl Knob {
    impl_midi! {
        Knob1 => 0x01,
        Knob2 => 0x02,
        Knob3 => 0x03,
        Knob4 => 0x04,
        Knob5 => 0x05,
        Knob6 => 0x06,
        Knob7 => 0x07,
        Knob8 => 0x08,
    }
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, EnumIter)]
pub enum ButtonLedState {
    Off,
    On,
    Blink,
}

impl ButtonLedState {
    impl_midi! {
        Off => 0x00,
        On => 0x7f,
        Blink => 0x01,
    }
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

    pub fn from_percent_nonzero(value: f64) -> Self {
        Self::new((((Self::MAX.0 - 1) as f64) * value) as u8 + 1)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KnobLedStyle {
    /// One LED is lit
    Single,
    /// Doesn't work in MC mode
    Pan,
    /// Light all LEDs from the left until the current value
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
            &[0xb0, controller_num, value] => {
                let delta = if value >= 64 {
                    -((value - 64) as i8)
                } else {
                    value as i8
                };

                Ok(KnobTurned {
                    knob: Knob::from_midi(controller_num - 0x0f)
                        .context("unknown controller number for knob")?,
                    delta,
                })
            }
            &[0xe8, _, value] => Ok(Event::FaderMoved {
                value: FaderValue(value),
            }),
            &[0x90, note, state] => {
                let is_down = state != 0;

                if note >= 0x20 && note <= 0x27 {
                    Ok(Event::KnobPressed {
                        knob: Knob::from_midi(note - 0x1f).context("unknown note for knob")?,
                        is_down,
                    })
                } else {
                    Ok(Event::ButtonPressed {
                        button: Button::from_midi(note).context("unknown note for button")?,
                        is_down,
                    })
                }
            }
            _ => bail!("unknown event: {:?}", bytes),
        }
    }
}
