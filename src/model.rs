use crate::output::Command;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct State {
    layer_a: Layout,
    layer_b: Layout,
}

impl State {
    pub fn to_commands(&self) -> Vec<Command> {
        let mut out = vec![Command::ChangeLayer { layer: Layer::B }];
        out.append(&mut self.layer_b.to_commands());
        out.push(Command::ChangeLayer { layer: Layer::A });
        out.append(&mut self.layer_a.to_commands());
        out
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Layout {
    knobs: [KnobState; 8],
    buttons: [ButtonLight; 16],
    fader: FaderState,
}

impl Layout {
    pub fn to_commands(&self) -> Vec<Command> {
        let mut out = Vec::new();

        for (index, state) in self.knobs.iter().enumerate() {
            if let Some(knob) = Knob::from_index(index as u8 + 1) {
                out.push(Command::SetKnobValue {
                    knob,
                    value: state.value,
                });
                out.push(Command::SetRingLighting {
                    knob,
                    lighting: state.lighting,
                });
                out.push(Command::ChangeRingLedBehavior {
                    knob,
                    behavior: state.behavior,
                });
            }
        }

        for (index, state) in self.buttons.iter().enumerate() {
            if let Some(button) = Button::from_index(index as u8 + 1) {
                out.push(Command::SetButtonLight {
                    button,
                    state: *state,
                });
            }
        }

        out
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct FaderState {
    value: u8,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct KnobState {
    value: u8,
    lighting: RingLighting,
    behavior: RingLedBehavior,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Layer {
    A,
    B,
}

impl Default for Layer {
    fn default() -> Self {
        Self::A
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Button {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
    Button9,
    Button10,
    Button11,
    Button12,
    Button13,
    Button14,
    Button15,
    Button16,
}

impl Button {
    pub fn to_index(&self) -> u8 {
        use Button::*;
        match self {
            Button1 => 1,
            Button2 => 2,
            Button3 => 3,
            Button4 => 4,
            Button5 => 5,
            Button6 => 6,
            Button7 => 7,
            Button8 => 8,
            Button9 => 9,
            Button10 => 10,
            Button11 => 11,
            Button12 => 12,
            Button13 => 13,
            Button14 => 14,
            Button15 => 15,
            Button16 => 16,
        }
    }

    pub fn from_index(index: u8) -> Option<Button> {
        use Button::*;
        Some(match index {
            1 => Button1,
            2 => Button2,
            3 => Button3,
            4 => Button4,
            5 => Button5,
            6 => Button6,
            7 => Button7,
            8 => Button8,
            9 => Button9,
            10 => Button10,
            11 => Button11,
            12 => Button12,
            13 => Button13,
            14 => Button14,
            15 => Button15,
            16 => Button16,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Knob {
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
    pub fn to_index(&self) -> u8 {
        use Knob::*;
        match self {
            Knob1 => 1,
            Knob2 => 2,
            Knob3 => 3,
            Knob4 => 4,
            Knob5 => 5,
            Knob6 => 6,
            Knob7 => 7,
            Knob8 => 8,
        }
    }

    pub fn from_index(index: u8) -> Option<Knob> {
        use Knob::*;
        Some(match index {
            1 => Knob1,
            2 => Knob2,
            3 => Knob3,
            4 => Knob4,
            5 => Knob5,
            6 => Knob6,
            7 => Knob7,
            8 => Knob8,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RingLighting {
    Off,
    On(RingIndex),
    Blink(RingIndex),
    AllOn,
    AllBlink,
}

impl Default for RingLighting {
    fn default() -> Self {
        Self::Off
    }
}

impl RingLighting {
    pub fn as_u8(&self) -> u8 {
        use RingLighting::*;
        match self {
            Off => 0,
            On(index) => index.0,
            Blink(index) => index.0 + 13,
            AllOn => 26,
            AllBlink => 27,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct RingIndex(u8);

impl RingIndex {
    pub fn new(value: u8) -> Result<RingIndex, ()> {
        if value < 13 {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonLight {
    Off,
    On,
    Blink,
}

impl Default for ButtonLight {
    fn default() -> Self {
        Self::Off
    }
}

impl ButtonLight {
    pub fn as_u8(&self) -> u8 {
        use ButtonLight::*;
        match self {
            Off => 0,
            On => 1,
            Blink => 2,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RingLedBehavior {
    Single,
    Pan,
    Fan,
    Spread,
    Trim,
}

impl Default for RingLedBehavior {
    fn default() -> Self {
        Self::Single
    }
}

impl RingLedBehavior {
    pub fn as_u8(&self) -> u8 {
        use RingLedBehavior::*;
        match self {
            Single => 0,
            Pan => 1,
            Fan => 2,
            Spread => 3,
            Trim => 4,
        }
    }
}
