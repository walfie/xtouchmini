use autopilot::key::{Character, Code, KeyCode};
const INPUT_DELAY: u64 = 0;

pub fn tap_key(code: KeyCode) {
    autopilot::key::tap(&Code(code), &[], INPUT_DELAY, 0);
}

pub fn tap_char(c: char) {
    autopilot::key::tap(&Character(c), &[], INPUT_DELAY, 0);
}

pub fn type_text(text: &str) {
    autopilot::key::type_string(text, &[], 0., 0.);
}
