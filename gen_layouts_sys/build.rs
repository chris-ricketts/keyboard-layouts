#![recursion_limit = "128"]
#[cfg(feature = "generate")]
use proc_macro2::TokenStream;
#[cfg(feature = "generate")]
use quote::{quote, ToTokens};
#[cfg(feature = "generate")]
use regex::RegexBuilder;
#[cfg(feature = "generate")]
use syn::{Expr, Item, Lit};

#[cfg(feature = "generate")]
use std::{env, fs, path::PathBuf};

#[cfg(feature = "generate")]
const KEY_LAYOUTS_HEADER: &'static str = include_str!("keylayouts.h");
#[cfg(feature = "generate")]
const N_ASCII_CHARS_SUPPORTED: usize = 96;
#[cfg(feature = "generate")]
const N_NUMPAD_KEYS: usize = 10;

#[cfg(feature = "generate")]
struct GlobalKeys {
    pub enter: u16,
    pub tab: u16,
    pub shift_modifier: u16,
    pub right_alt_modifier: u16,
    pub left_alt_modifier: u16,
    pub right_ctrl_modifier: u16,
    pub numpad_keys: [u16; N_NUMPAD_KEYS],
    pub numlock: u16,
}

#[cfg(feature = "generate")]
struct LayoutMasks {
    pub shift_mask: u16,
    pub alt_mask: Option<u16>,
    pub ctrl_mask: Option<u16>,
    pub non_us: Option<u16>,
    pub dead_keys_mask: Option<u16>,
    pub keycode_mask: u16,
}

#[cfg(feature = "generate")]
struct LayoutDeadKeys {
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

fn main() {
    #[cfg(feature = "generate")]
    generate()
}

#[cfg(feature = "generate")]
fn generate() {
    let GlobalKeys {
        enter,
        tab,
        shift_modifier,
        right_alt_modifier,
        left_alt_modifier,
        right_ctrl_modifier,
        numpad_keys,
        numlock,
    } = get_global_keys();

    // Layout and DeadKeys come from src/types.rs
    let layouts = find_layout_definitions()
        .iter()
        .map(|def| {
            let layout = generate_layout(def);

            let LayoutMasks {
                shift_mask,
                alt_mask,
                ctrl_mask,
                non_us,
                dead_keys_mask,
                keycode_mask,
            } = extract_layout_masks(&layout, def);

            let LayoutDeadKeys {
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
            } = extract_layout_deadkeys(&layout);

            let keycodes = extract_ascii_keycodes(&layout)
                .iter()
                .map(|k| k & keycode_mask)
                .collect::<Vec<u16>>();

            assert_eq!(
                N_ASCII_CHARS_SUPPORTED,
                keycodes.len(),
                "Not enough ASCII keycodes extracted from {}: {}/{}",
                def,
                keycodes.len(),
                N_ASCII_CHARS_SUPPORTED
            );

            let layout_key = def.to_string();
            let quote_alt_mask = quote_option(alt_mask);
            let quote_ctrl_mask = quote_option(ctrl_mask);
            let quote_non_us = quote_option(non_us);
            let quote_dead_keys_mask = quote_option(dead_keys_mask);
            let quote_acute_accent_bits = quote_option(acute_accent_bits);
            let quote_deadkey_accute_accent = quote_option(deadkey_accute_accent);
            let quote_cedilla_bits = quote_option(cedilla_bits);
            let quote_deadkey_cedilla = quote_option(deadkey_cedilla);
            let quote_diaeresis_bits = quote_option(diaeresis_bits);
            let quote_deadkey_diaeresis = quote_option(deadkey_diaeresis);
            let quote_grave_accent_bits = quote_option(grave_accent_bits);
            let quote_deadkey_grave_accent = quote_option(deadkey_grave_accent);
            let quote_circumflex_bits = quote_option(circumflex_bits);
            let quote_deadkey_circumflex = quote_option(deadkey_circumflex);
            let quote_tilde_bits = quote_option(tilde_bits);
            let quote_deadkey_tilde = quote_option(deadkey_tilde);

            quote! {
                m.insert(
                    #layout_key,
                    Layout::new(
                        #shift_mask,
                        #quote_alt_mask,
                        #quote_ctrl_mask,
                        #quote_non_us,
                        #quote_dead_keys_mask,
                        #keycode_mask,
                        vec![#(#keycodes),*],
                        DeadKeys::new(
                            #quote_acute_accent_bits,
                            #quote_deadkey_accute_accent,
                            #quote_cedilla_bits,
                            #quote_deadkey_cedilla,
                            #quote_diaeresis_bits,
                            #quote_deadkey_diaeresis,
                            #quote_grave_accent_bits,
                            #quote_deadkey_grave_accent,
                            #quote_circumflex_bits,
                            #quote_deadkey_circumflex,
                            #quote_tilde_bits,
                            #quote_deadkey_tilde,
                        )
                    ),
                );
            }
        })
        .collect::<Vec<TokenStream>>();

    let quote_numpad_keys = (0..N_NUMPAD_KEYS)
        .map(|idx| {
            let keycode = numpad_keys[idx];
            quote! { #keycode , }
        })
        .collect::<Vec<TokenStream>>();

    // Layout comes from src/types.rs
    let full_output = quote! {
        use std::collections::HashMap;
        use lazy_static::lazy_static;

        pub const ENTER_KEYCODE: u16 = #enter;
        pub const TAB_KEYCODE: u16 = #tab;
        pub const SHIFT_MODIFIER: u16 = #shift_modifier;
        pub const RIGHT_ALT_MODIFIER: u16 = #right_alt_modifier;
        pub const LEFT_ALT_MODIFIER: u16 = #left_alt_modifier;
        pub const RIGHT_CTRL_MODIFIER: u16 = #right_ctrl_modifier;
        pub const NUMLOCK: u16 = #numlock;
        pub const NUMPAD_KEYS: [u16; #N_NUMPAD_KEYS] = [
            #(#quote_numpad_keys)*
        ];

        lazy_static! {
            pub static ref LAYOUT_MAP: HashMap<&'static str, Layout> = {
                let mut m = HashMap::new();
                #(#layouts)*
                m
            };
        }
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not defined"));

    fs::write(out_path.join("generated.rs"), &full_output.to_string())
        .expect("Failed to write generated output");
}

// https://github.com/dtolnay/quote/issues/20
#[cfg(feature = "generate")]
fn quote_option<T: ToTokens>(o: Option<T>) -> TokenStream {
    match o {
        Some(ref t) => quote! { Some(#t) },
        None => quote! { None },
    }
}

#[cfg(feature = "generate")]
fn generate_layout(layout: &str) -> syn::File {
    let header_name = format!("{}.h", layout);
    let defined_layout = format!("#define {}\n{}", layout, KEY_LAYOUTS_HEADER);
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header_contents(&header_name, &defined_layout)
        .generate()
        .expect(&format!("Unable to generate bindings for {}", layout))
        .to_string();

    syn::parse_str::<syn::File>(&bindings).expect("Failed to parse bindings")
}

#[cfg(feature = "generate")]
fn extract_ascii_keycodes(definitions: &syn::File) -> Vec<u16> {
    definitions
        .items
        .iter()
        .filter_map(|item| find_const_u16_with_name_containing(item, "ASCII_"))
        .collect()
}

#[cfg(feature = "generate")]
fn extract_layout_masks(definitions: &syn::File, layout: &str) -> LayoutMasks {
    LayoutMasks {
        shift_mask: find_key_definition(definitions, "SHIFT_MASK")
            .expect(&format!("Failed to find SHIFT_MASK for {}", layout)),
        alt_mask: find_key_definition(definitions, "ALTGR_MASK"),
        ctrl_mask: find_key_definition(definitions, "RCTRL_MASK"),
        non_us: find_key_definition(definitions, "KEY_NON_US_100"),
        dead_keys_mask: find_key_definition(definitions, "DEADKEYS_MASK"),
        keycode_mask: find_key_definition(definitions, "KEYCODE_MASK")
            .expect(&format!("Failed to find KEYCODE_MASK for {}", layout)),
    }
}

#[cfg(feature = "generate")]
fn extract_layout_deadkeys(definitions: &syn::File) -> LayoutDeadKeys {
    LayoutDeadKeys {
        acute_accent_bits: find_key_definition(definitions, "ACUTE_ACCENT_BITS"),
        deadkey_accute_accent: find_key_definition(definitions, "DEADKEY_ACCUTE_ACCENT"),
        cedilla_bits: find_key_definition(definitions, "CEDILLA_BITS"),
        deadkey_cedilla: find_key_definition(definitions, "DEADKEY_CEDILLA"),
        diaeresis_bits: find_key_definition(definitions, "DIAERESIS_BITS"),
        deadkey_diaeresis: find_key_definition(definitions, "DEADKEY_DIAERESIS"),
        grave_accent_bits: find_key_definition(definitions, "GRAVE_ACCENT_BITS"),
        deadkey_grave_accent: find_key_definition(definitions, "DEADKEY_GRAVE_ACCENT"),
        circumflex_bits: find_key_definition(definitions, "CIRCUMFLEX_BITS"),
        deadkey_circumflex: find_key_definition(definitions, "DEADKEY_CIRCUMFLEX"),
        tilde_bits: find_key_definition(definitions, "TILDE_BITS"),
        deadkey_tilde: find_key_definition(definitions, "DEADKEY_TILDE"),
    }
}

#[cfg(feature = "generate")]
fn get_global_keys() -> GlobalKeys {
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header_contents("base.h", KEY_LAYOUTS_HEADER)
        .generate()
        .expect("Unable to generate base bindings")
        .to_string();

    let definitions = syn::parse_str::<syn::File>(&bindings).expect("Failed to parse bindings");

    GlobalKeys {
        enter: find_key_definition(&definitions, "KEY_ENTER")
            .expect("Failed to find global key: KEY_ENTER"),
        tab: find_key_definition(&definitions, "KEY_TAB")
            .expect("Failed to find global key: KEY_TAB"),
        shift_modifier: find_key_definition(&definitions, "MODIFIERKEY_SHIFT")
            .expect("Failed to find global key: MODIFIERKEY_SHIFT"),
        right_alt_modifier: find_key_definition(&definitions, "MODIFIERKEY_RIGHT_ALT")
            .expect("Failed to find global key: MODIFIERKEY_RIGHT_ALT"),
        left_alt_modifier: find_key_definition(&definitions, "MODIFIERKEY_LEFT_ALT")
            .expect("Failed to find global key: MODIFIERKEY_LEFT_ALT"),
        right_ctrl_modifier: find_key_definition(&definitions, "MODIFIERKEY_RIGHT_CTRL")
            .expect("Failed to find global key: MODIFIERKEY_RIGHT_CTRL"),
        numlock: find_key_definition(&definitions, "KEY_NUM_LOCK")
            .expect("Failed to find global key: KEY_NUM_LOCK"),
        numpad_keys: [
            find_key_definition(&definitions, "KEYPAD_0")
                .expect("Failed to find global key: KEYPAD_0"),
            find_key_definition(&definitions, "KEYPAD_1")
                .expect("Failed to find global key: KEYPAD_1"),
            find_key_definition(&definitions, "KEYPAD_2")
                .expect("Failed to find global key: KEYPAD_2"),
            find_key_definition(&definitions, "KEYPAD_3")
                .expect("Failed to find global key: KEYPAD_3"),
            find_key_definition(&definitions, "KEYPAD_4")
                .expect("Failed to find global key: KEYPAD_4"),
            find_key_definition(&definitions, "KEYPAD_5")
                .expect("Failed to find global key: KEYPAD_5"),
            find_key_definition(&definitions, "KEYPAD_6")
                .expect("Failed to find global key: KEYPAD_6"),
            find_key_definition(&definitions, "KEYPAD_7")
                .expect("Failed to find global key: KEYPAD_7"),
            find_key_definition(&definitions, "KEYPAD_8")
                .expect("Failed to find global key: KEYPAD_8"),
            find_key_definition(&definitions, "KEYPAD_9")
                .expect("Failed to find global key: KEYPAD_9"),
        ],
    }
}

#[cfg(feature = "generate")]
fn find_layout_definitions() -> Vec<&'static str> {
    let layout_def_regex = RegexBuilder::new(r"//\#define\s+?(LAYOUT_.*)")
        .multi_line(true)
        .build()
        .expect("Failed to build regex");

    layout_def_regex
        .captures_iter(KEY_LAYOUTS_HEADER)
        .map(|cap| cap.get(1).unwrap().as_str())
        .map(|def| def.trim())
        .collect()
}

#[cfg(feature = "generate")]
fn find_key_definition(definitions: &syn::File, label: &str) -> Option<u16> {
    definitions
        .items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, label))
}

#[cfg(feature = "generate")]
fn find_const_u16_with_name_containing(item: &Item, name: &str) -> Option<u16> {
    match item {
        Item::Const(c) => {
            let ident = c.ident.to_string();
            if ident.contains(name) {
                if let Expr::Lit(ref lit) = *c.expr {
                    if let Lit::Int(ref i) = lit.lit {
                        Some(i.value() as u16)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
