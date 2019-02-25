pub struct DeadKeys {
    pub acute_accent_bits: Option<u16>,
    pub deadkey_accute_accent: Option<u16>,
    pub cedilla_bits: Option<u16>,
    pub deadkey_cedilla: Option<u16>,
    pub diaeresis_bits: Option<u16>,
    pub deadkey_diaeresis: Option<u16>,
    pub grave_accent_bits: Option<u16>,
    pub deadkey_grave_accent: Option<u16>,
    pub circumflex_bits: Option<u16>,
    pub deadkey_circumflex: Option<u16>,
    pub tilde_bits: Option<u16>,
    pub deadkey_tilde: Option<u16>,
}

pub struct Layout {
    pub layout_name: &'static str,
    pub shift_mask: u16,
    pub alt_mask: Option<u16>,
    pub ctrl_mask: Option<u16>,
    pub non_us: Option<u16>,
    pub dead_keys_mask: Option<u16>,
    pub keycode_mask: u16,
    pub keycodes: Box<[u16]>,
    pub deadkeys: DeadKeys,
}

impl DeadKeys {
    pub fn new(
        acute_accent_bits: Option<u16>,
        deadkey_accute_accent: Option<u16>,
        cedilla_bits: Option<u16>,
        deadkey_cedilla: Option<u16>,
        diaeresis_bits: Option<u16>,
        deadkey_diaeresis: Option<u16>,
        grave_accent_bits: Option<u16>,
        deadkey_grave_accent: Option<u16>,
        circumflex_bits: Option<u16>,
        deadkey_circumflex: Option<u16>,
        tilde_bits: Option<u16>,
        deadkey_tilde: Option<u16>,
    ) -> DeadKeys {
        DeadKeys {
            acute_accent_bits,
            deadkey_accute_accent,
            cedilla_bits,
            deadkey_cedilla,
            diaeresis_bits,
            deadkey_diaeresis,
            grave_accent_bits,
            deadkey_grave_accent,
            circumflex_bits,
            deadkey_circumflex,
            tilde_bits,
            deadkey_tilde,
        }
    }
}

impl Layout {
    pub fn new(
        layout_name: &'static str,
        shift_mask: u16,
        alt_mask: Option<u16>,
        ctrl_mask: Option<u16>,
        non_us: Option<u16>,
        dead_keys_mask: Option<u16>,
        keycode_mask: u16,
        keycodes: Vec<u16>,
        deadkeys: DeadKeys,
    ) -> Layout {
        let keycodes = keycodes.into_boxed_slice();
        Layout {
            layout_name,
            shift_mask,
            alt_mask,
            ctrl_mask,
            non_us,
            dead_keys_mask,
            keycode_mask,
            keycodes,
            deadkeys,
        }
    }
}
