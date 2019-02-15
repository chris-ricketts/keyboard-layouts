#![recursion_limit = "128"]
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::RegexBuilder;
use syn::{Expr, Ident, Item, Lit};

use std::env;
use std::fs;
use std::path::PathBuf;

const KEY_LAYOUTS_HEADER: &'static str = include_str!("keylayouts.h");
const ASCII_CODE_START: u8 = 0x20;
const ASCII_CODE_END: u8 = 0x7F;
const UNICODE_ENTER: u8 = 10;
const UNICODE_TAB: u8 = 11;
const N_ASCII_CHARS_SUPPORTED: usize = (ASCII_CODE_END - ASCII_CODE_START) as usize; 

struct GlobalKeys {
    pub enter: u16,
    pub tab: u16,
    pub shift_modifier: u16,
    pub right_alt_modifier: u16,
    pub right_ctrl_modifier: u16,
}

struct Layout

fn main() {
    let GlobalKeys {
        enter,
        tab,
        shift_modifier,
        right_alt_modifier,
        right_ctrl_modifier,
    } = get_global_keys();

    let layout_defs = find_layout_definitions();

    let layout_display_names: Vec<String> = layout_defs
        .iter()
        .map(|l| l.replace("LAYOUT_", "").replace("_", " "))
        .collect();

    let layout_idents: Vec<Ident> = layout_defs
        .iter()
        .map(|l| Ident::new(l, Span::call_site()))
        .collect();

    // First layout idents is unusable after first interpolation
    let layout_idents_clone = layout_idents.clone();

    let layout_vecs: Vec<TokenStream> = layout_defs
        .iter()
        .map(|l| {
            let keycodes = get_keycodes_for_layout(l, enter, tab);
            quote! {
                vec![#(#keycodes),*]
            }
        })
        .collect();

    let full_output = quote! {
        use std::collections::HashMap;
        use lazy_static::lazy_static;

        const SHIFT_MODIFIER: u16 = #shift_modifier;
        const RIGHT_ALT_MODIFIER: u16 = #right_alt_modifier;
        const RIGHT_CTRL_MODIFIER: u16 = #right_ctrl_modifier;

        lazy_static! {
            #(static ref #layout_idents: Vec<u16> = #layout_vecs;)*
            static ref LAYOUT_MAP: HashMap<&'static str, (&'static str, &'static [u16])> = {
                let mut m = HashMap::new();
                #(m.insert(#layout_defs, (#layout_display_names, #layout_idents_clone.as_slice()));)*
                m
            };
        }

        fn determine_modifier(keycode: u16) -> u8 {
            let mut modifier = 0u8;
            if keycode & SHIFT

        pub fn available_layouts() -> Vec<(&'static str, &'static str)> {
            LAYOUT_MAP.iter().map(|(k, (m, _))| (*k, *m)).collect()
        }

        pub fn unicode_to_key_and_modifier(layout: &str, unicode: u8) -> Option<(u8, u8)> {
            let (_, layout) = LAYOUT_MAP.get(layout)?;
            match unicode {
                #UNICODE_ENTER => Some((*layout.last().unwrap(), 0))
                #UNICODE_TAB => Some((*layout.last().unwrap(), 0))
                u if u >= #ASCII_CODE_START && u <= #ASCII_CODE_END =>  {
                    let keycode = layout.get(u - 0x20).unwrap();
                    let modifier = determine



    };

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not defined"));

    fs::write(out_path.join("generated.rs"), &full_output.to_string())
        .expect("Failed to write generated output");
}

fn get_keycodes_for_layout(layout: &str, enter_key: u16, tab_key: u16) -> Vec<u16> {
    let header_name = format!("{}.h", layout);
    let defined_layout = format!("#define {}\n{}", layout, KEY_LAYOUTS_HEADER);
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header_contents(&header_name, &defined_layout)
        .generate()
        .expect("Unable to generate bindings")
        .to_string();

    let syntax = syn::parse_str::<syn::File>(&bindings).expect("Failed to parse bindings");

    let keycode_mask = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "KEYCODE_MASK"))
        .expect("Failed to find KEYCODE_MASK");

    let mut ascii_keycodes: Vec<u16> = syntax
        .items
        .iter()
        .filter_map(|item| find_const_u16_with_name_containing(item, "ASCII_"))
        .map(|k| k & keycode_mask)
        .collect();

    if ascii_keycodes.len() != N_ASCII_CHARS_SUPPORTED {
        panic!("Failed to find enough keycodes for layout: {}", layout);
    }
    
    // Add tab and enter keycodes to the end of the ascii ones
    // as the code is dependent on the layout's keycode mask
    ascii_keycodes.push(tab_key & keycode_mask);
    ascii_keycodes.push(enter_key & keycode_mask);

    ascii_keycodes
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

fn get_global_keys() -> GlobalKeys {
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .header_contents("base.h", KEY_LAYOUTS_HEADER)
        .generate()
        .expect("Unable to generate bindings")
        .to_string();

    let syntax = syn::parse_str::<syn::File>(&bindings).expect("Failed to parse bindings");

    let enter = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "KEY_ENTER"))
        .expect("Failed to find KEY_ENTER");

    let tab = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "KEY_TAB"))
        .expect("Failed to find KEY_TAB");

    let shift_modifier = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "MODIFIERKEY_SHIFT"))
        .expect("Failed to find MODIFIERKEY_SHIFT");

    let right_alt_modifier = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "MODIFIERKEY_RIGHT_ALT"))
        .expect("Failed to find MODIFIERKEY_RIGHT_ALT");

    let right_ctrl_modifier = syntax
        .items
        .iter()
        .find_map(|item| find_const_u16_with_name_containing(item, "MODIFIERKEY_RIGHT_CTRL"))
        .expect("Failed to find MODIFIERKEY_RIGHT_CTRL");

    GlobalKeys {
        enter,
        tab,
        shift_modifier,
        right_alt_modifier,
        right_ctrl_modifier,
    }
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
