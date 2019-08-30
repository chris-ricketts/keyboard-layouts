#[macro_use]
extern crate log;

use bytes::{BufMut, Bytes, BytesMut};
use gen_layouts_sys::*;

use std::fmt;

const UNICODE_ENTER: u16 = 10; // \n
const UNICODE_TAB: u16 = 9; // \t
// https://stackoverflow.com/questions/23320417/what-is-this-character-separator
const CONTROL_CHARACTER_OFFSET: u16 = 0x40;
const UNICODE_FIRST_ASCII: u16 = 0x20; // SPACE
const UNICODE_LAST_ASCII: u16 = 0x7F; // BACKSPACE
const UNICODE_DIGIT_OFFSET: usize = 48; // 0
const KEY_MASK: u16 = 0x3F; // Remove SHIFT/ALT/CTRL from keycode
/// The number of bytes in a keyboard HID packet
pub const HID_PACKET_LEN: usize = 8;
const HID_PACKET_SUFFIX: [u8; 5] = [0u8; 5];
const RELEASE_KEYS_HID_PACKET: [u8; 8] = [0u8; 8];

#[derive(Debug)]
pub enum Error {
    InvalidLayoutKey(String),
    InvalidCharacter(char),
}

#[derive(Debug)]
pub enum Release {
    All,
    Keys,
    None,
}

#[derive(Debug)]
pub struct KeyMod {
    pub key: u8,
    pub modifier: u8,
    pub release: Release,
}

enum Keycode {
    ModifierKeySequence(u16, Vec<u16>),
    RegularKey(u16),
    InvalidCharacter,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidLayoutKey(key) => write!(f, "No layout defined for {}", key),
            Error::InvalidCharacter(c) => write!(f, "Invalid character: '{}' or [{:?}]", c, c),
        }
    }
}

/// Get a list of the supported keyboard layouts
pub fn available_layouts() -> Vec<&'static str> {
    LAYOUT_MAP.keys().map(|k| *k).collect()
}

/// Get a list of the key and modifier pairs required to type the given string on a keyboard with
/// the specified layout.
pub fn string_to_keys_and_modifiers(layout_key: &str, string: &str) -> Result<Vec<KeyMod>, Error> {
    let layout = LAYOUT_MAP
        .get(layout_key)
        .ok_or_else(|| Error::InvalidLayoutKey(layout_key.to_string()))?;

    let mut keys_and_modifiers: Vec<KeyMod> = Vec::with_capacity(string.len());

    for c in string.chars() {
        match keycode_for_unicode(layout, c as u16) {
            Keycode::ModifierKeySequence(modifier, sequence) => {
                for keycode in sequence {
                    keys_and_modifiers.push(KeyMod {
                        key: keycode as u8,
                        modifier: modifier as u8,
                        release: Release::Keys,
                    });
                }
                // Manually add release after sequence is finished
                keys_and_modifiers.push(KeyMod {
                    key: 0,
                    modifier: 0,
                    release: Release::None,
                });
            }
            Keycode::RegularKey(keycode) => {
                if let Some(dead_keycode) = deadkey_for_keycode(layout, keycode) {
                    let key = key_for_keycode(layout, dead_keycode);
                    let modifier = modifier_for_keycode(layout, dead_keycode);
                    keys_and_modifiers.push(KeyMod {
                        key,
                        modifier,
                        release: Release::All,
                    });
                }
                let key = key_for_keycode(layout, keycode);
                let modifier = modifier_for_keycode(layout, keycode);
                keys_and_modifiers.push(KeyMod {
                    key,
                    modifier,
                    release: Release::All,
                });
            }
            _ => return Err(Error::InvalidCharacter(c)),
        }
    }

    Ok(keys_and_modifiers)
}

/// Create the sequence of HID packets required to type the given string. Impersonating a keyboard
/// with the specified layout. These packets can be written directly to a HID device file.
pub fn string_to_hid_packets(layout_key: &str, string: &str) -> Result<Bytes, Error> {
    let keys_and_modifiers = string_to_keys_and_modifiers(layout_key, string)?;

    debug!("Keys and Modifiers for {}:{:?}", string, keys_and_modifiers);
    let mut packet_bytes = BytesMut::with_capacity(HID_PACKET_LEN * keys_and_modifiers.len() * 2);

    for KeyMod {
        key,
        modifier,
        release,
    } in keys_and_modifiers.iter()
    {
        packet_bytes.put_u8(*modifier);
        packet_bytes.put_u8(0u8);
        packet_bytes.put_u8(*key);
        packet_bytes.put_slice(&HID_PACKET_SUFFIX);
        match *release {
            Release::All => packet_bytes.put_slice(&RELEASE_KEYS_HID_PACKET),
            Release::Keys => {
                packet_bytes.put_u8(*modifier);
                packet_bytes.put_u8(0u8);
                packet_bytes.put_u8(0u8);
                packet_bytes.put_slice(&HID_PACKET_SUFFIX);
            }
            Release::None => {}
        }
    }

    Ok(packet_bytes.freeze())
}

fn keycode_for_unicode(layout: &Layout, unicode: u16) -> Keycode {
    match unicode {
        u if u == UNICODE_ENTER => Keycode::RegularKey(ENTER_KEYCODE & layout.keycode_mask),
        u if u == UNICODE_TAB => Keycode::RegularKey(TAB_KEYCODE & layout.keycode_mask),
        u if u < UNICODE_FIRST_ASCII =>  {
            let idx = ((u + CONTROL_CHARACTER_OFFSET) - UNICODE_FIRST_ASCII) as usize;
            let keycodes = vec![layout.keycodes[idx]];
            Keycode::ModifierKeySequence(RIGHT_CTRL_MODIFIER, keycodes)
        }
        u if u >= UNICODE_FIRST_ASCII && u <= UNICODE_LAST_ASCII => {
            let idx = (u - UNICODE_FIRST_ASCII) as usize;
            Keycode::RegularKey(layout.keycodes[idx])
        }
        _ => Keycode::InvalidCharacter,
    }
}

// https://github.com/PaulStoffregen/cores/blob/master/teensy3/usb_keyboard.c
fn deadkey_for_keycode(layout: &Layout, keycode: u16) -> Option<u16> {
    layout.dead_keys_mask.and_then(|dkm| {
        let keycode = keycode & dkm;
        if let Some(acute_accent_bits) = layout.deadkeys.acute_accent_bits {
            if keycode == acute_accent_bits {
                return layout.deadkeys.deadkey_accute_accent;
            }
        }
        if let Some(cedilla_bits) = layout.deadkeys.cedilla_bits {
            if keycode == cedilla_bits {
                return layout.deadkeys.deadkey_cedilla;
            }
        }
        if let Some(diaeresis_bits) = layout.deadkeys.diaeresis_bits {
            if keycode == diaeresis_bits {
                return layout.deadkeys.deadkey_diaeresis;
            }
        }
        if let Some(grave_accent_bits) = layout.deadkeys.grave_accent_bits {
            if keycode == grave_accent_bits {
                return layout.deadkeys.deadkey_grave_accent;
            }
        }
        if let Some(circumflex_bits) = layout.deadkeys.circumflex_bits {
            if keycode == circumflex_bits {
                return layout.deadkeys.deadkey_circumflex;
            }
        }
        if let Some(tilde_bits) = layout.deadkeys.tilde_bits {
            if keycode == tilde_bits {
                return layout.deadkeys.deadkey_tilde;
            }
        }
        None
    })
}

// https://github.com/PaulStoffregen/cores/blob/master/usb_hid/usb_api.cpp#L196
fn modifier_for_keycode(layout: &Layout, keycode: u16) -> u8 {
    let mut modifier = 0u16;

    if keycode & layout.shift_mask > 0 {
        modifier |= SHIFT_MODIFIER;
    }

    if let Some(alt_mask) = layout.alt_mask {
        if keycode & alt_mask > 0 {
            modifier |= RIGHT_ALT_MODIFIER;
        }
    }

    if let Some(ctrl_mask) = layout.ctrl_mask {
        if keycode & ctrl_mask > 0 {
            modifier |= RIGHT_CTRL_MODIFIER;
        }
    }

    modifier as u8
}

// https://github.com/PaulStoffregen/cores/blob/master/usb_hid/usb_api.cpp#L212
fn key_for_keycode(layout: &Layout, keycode: u16) -> u8 {
    let key = keycode & KEY_MASK;
    match layout.non_us {
        Some(non_us) => {
            if key == non_us {
                100u8
            } else {
                key as u8
            }
        }
        None => key as u8,
    }
}
