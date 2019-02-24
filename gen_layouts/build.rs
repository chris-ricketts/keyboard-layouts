#![recursion_limit = "128"]
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use regex::RegexBuilder;
use syn::{Expr, Item, Lit};

use std::env;
use std::fs;
use std::path::PathBuf;

const KEY_LAYOUTS_HEADER: &'static str = include_str!("keylayouts.h");
const N_ASCII_CHARS_SUPPORTED: usize = 96;

struct GlobalKeys {
    pub enter: u16,
    pub tab: u16,
    pub shift_modifier: u16,
    pub right_alt_modifier: u16,
    pub right_ctrl_modifier: u16,
}

struct LayoutMasks {
    pub shift_mask: u16,
    pub alt_mask: Option<u16>,
    pub ctrl_mask: Option<u16>,
    pub non_us: Option<u16>,
    pub dead_keys_mask: Option<u16>,
    pub keycode_mask: u16,
}

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
}

fn main() {
    let GlobalKeys {
        enter,
        tab,
        shift_modifier,
        right_alt_modifier,
        right_ctrl_modifier,
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
            let layout_name = layout_key.replace("LAYOUT_", "").replace("_", " ");
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

            quote! {
                m.insert(
                    #layout_key,
                    Layout::new(
                        #layout_name,
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
                        )
                    ),
                );
            }
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
        pub const RIGHT_CTRL_MODIFIER: u16 = #right_ctrl_modifier;

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
fn quote_option<T: ToTokens>(o: Option<T>) -> TokenStream {
    match o {
        Some(ref t) => quote! { Some(#t) },
        None => quote! { None },
    }
}

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

fn extract_ascii_keycodes(definitions: &syn::File) -> Vec<u16> {
    definitions
        .items
        .iter()
        .filter_map(|item| find_const_u16_with_name_containing(item, "ASCII_"))
        .collect()
}

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
    }
}

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
        right_ctrl_modifier: find_key_definition(&definitions, "MODIFIERKEY_RIGHT_CTRL")
            .expect("Failed to find global key: MODIFIERKEY_RIGHT_CTRL"),
    }
}

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

fn find_key_definition(definitions: &syn::File, label: &str) -> Option<u16> {
    definitions
        .items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, label))
}

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
