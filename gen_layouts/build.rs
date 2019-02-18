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
    pub keycode_mask: u16,
}

fn main() {
    let GlobalKeys {
        enter,
        tab,
        shift_modifier,
        right_alt_modifier,
        right_ctrl_modifier,
    } = get_global_keys();

    let layouts = find_layout_definitions()
        .iter()
        .map(|def| {
            dbg!(def);
            let layout = generate_layout(def);
            let LayoutMasks {
                shift_mask,
                alt_mask,
                ctrl_mask,
                non_us,
                keycode_mask,
            } = extract_layout_masks(&layout, def);
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
            quote! {
                m.insert(
                    #layout_key,
                    Layout::new(
                        #layout_name,
                        #shift_mask,
                        #quote_alt_mask,
                        #quote_ctrl_mask,
                        #keycode_mask,
                        vec![#(#keycodes),*]
                    ),
                );
            }
        })
        .collect::<Vec<TokenStream>>();

    let full_output = quote! {
        use std::collections::HashMap;
        use lazy_static::lazy_static;

        pub const ENTER_KEYCODE: u16 = #enter;
        pub const TAB_KEYCODE: u16 = #tab;
        pub const SHIFT_MODIFIER: u16 = #shift_modifier;
        pub const RIGHT_ALT_MODIFIER: u16 = #right_alt_modifier;
        pub const RIGHT_CTRL_MODIFIER: u16 = #right_ctrl_modifier;

        pub struct Layout {
            pub layout_name: &'static str,
            pub shift_mask: u16,
            pub alt_mask: Option<u16>,
            pub ctrl_mask: Option<u16>,
            pub non_us: Option<u16>,
            pub keycode_mask: u16,
            pub keycodes: Box<[u16]>
        }

        impl Layout {
            pub fn new(
                layout_name: &'static str,
                shift_mask: u16,
                alt_mask: Option<u16>,
                ctrl_mask: Option<u16>,
                non_us: Option<u16>,
                keycode_mask: u16,
                keycodes: Vec<u16>,
            ) -> Layout {
                let keycodes = keycodes.into_boxed_slice();
                Layout {
                    layout_name,
                    shift_mask,
                    alt_mask,
                    ctrl_mask,
                    non_us,
                    keycode_mask,
                    keycodes,
                }
            }
        }

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
        .expect("Unable to generate bindings")
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
    let items = &definitions.items;

    let shift_mask = items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, "SHIFT_MASK"))
        .expect("Failed to find SHIFT_MASK");

    let ctrl_mask = items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, "RCTRL_MASK"));

    let alt_mask = items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, "ALTGR_MASK"));

    let non_us = items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, "KEY_NON_US_100"));

    let keycode_mask = items
        .iter()
        .find_map(|i| find_const_u16_with_name_containing(i, "KEYCODE_MASK"))
        .expect(&format!("Failed to find KEYCODE_MASK for {}", layout));

    LayoutMasks {
        shift_mask,
        alt_mask,
        ctrl_mask,
        non_us,
        keycode_mask,
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
